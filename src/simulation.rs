use std::{rc::Rc, thread};

use crate::{
    atom_type::AtomType,
    atomic,
    compute::{self, ComputeTrait},
    output::{self, Value},
    parallel::{comm, message as msg, Domain, Worker},
    utils::{KeyError, KeyedVec},
    Atoms, Axis, Container, Integrator, NeighborList, Output, OutputSpec, Verlet,
};
type ComputeVec = KeyedVec<String, compute::Compute>;

pub struct Simulation<'a, T, A>
where
    T: AtomType,
    A: atomic::AtomicPotentialTrait<T>,
{
    pub atoms: Atoms<T>,
    container: Rc<Container>,
    atomic_potential: A,
    pub neighbor_list: NeighborList,
    domain: Domain<'a, T>,
    output: output::Output,
    pos_at_prev_neigh_build: Vec<[f64; 3]>,
    computes: ComputeVec,
    timestep: f64,
    forces: Vec<[f64; 3]>,
}
impl<'a, T, A> Simulation<'a, T, A>
where
    T: AtomType,
    A: atomic::AtomicPotentialTrait<T>,
{
    pub fn new(timestep: f64, atoms: Atoms<T>, atomic_potential: A, container: Container) -> Self {
        assert!(
            timestep > 0.0,
            "Timestep should be positive, found {}",
            timestep,
        );
        let container = Rc::new(container);
        let neighbor_list = NeighborList::new(
            container.clone(),
            atomic_potential.cutoff_distance(),
            1.0,
            1,
            0,
            true,
        );
        Self {
            atoms,
            container,
            atomic_potential,
            neighbor_list,
            domain: Domain::new(),
            output: output::Output::new(),
            pos_at_prev_neigh_build: Vec::new(),
            computes: KeyedVec::new(),
            timestep,
            forces: Vec::new(),
        }
    }

    /// Initializes the simulation from a worker thread
    pub fn connect(&mut self, worker: Box<&'a Worker<T>>) {
        self.domain.init(&self.container, worker)
    }

    // Getters
    pub fn container(&self) -> &Container {
        &self.container
    }
    pub fn atomic_potential(&self) -> &A {
        &self.atomic_potential
    }
    pub fn mut_atomic_potential(&mut self) -> &mut A {
        &mut self.atomic_potential
    }
    pub(crate) fn domain(&self) -> &Domain<T> {
        &self.domain
    }
    pub(crate) fn nlocal(&self) -> usize {
        self.atoms.nlocal
    }
    pub fn computes(&self) -> &ComputeVec {
        &self.computes
    }
    pub fn get_compute(&self, id: &str) -> Result<&compute::Compute, KeyError> {
        self.computes.get(&String::from(id))
    }
    pub fn timestep(&self) -> f64 {
        self.timestep
    }
    pub(crate) fn forces(&self) -> &Vec<[f64; 3]> {
        &self.forces
    }
    pub(crate) fn mut_forces(&mut self) -> &mut Vec<[f64; 3]> {
        &mut self.forces
    }

    // Setters
    pub fn set_container(&mut self, container: Container) {
        self.container = Rc::new(container);
        self.domain.reset_subdomain(&self.container);
    }
    pub fn set_domain(&mut self, domain: Domain<'a, T>) {
        self.domain = domain;
    }
    pub fn set_output(&mut self, every: usize, output_keys: Vec<&str>) {
        let output_specs: Vec<OutputSpec> = output_keys
            .iter()
            .map(|&key| {
                if key == "step" {
                    OutputSpec::Step
                } else {
                    let c = self.get_compute(key).expect("Invalid compute id");
                    OutputSpec::Compute(c.clone())
                }
            })
            .collect();

        self.domain
            .send_to_main(msg::W2M::SetupOutput(self.output.values.clone()));
        self.output = Output {
            every,
            values: output_specs,
        };
    }
    pub(crate) fn increment_nlocal(&mut self) {
        self.atoms.nlocal += 1;
    }
    pub fn add_compute(&mut self, id: &str, compute: compute::Compute) {
        self.computes.add(String::from(id), compute)
    }
    pub fn set_atom_types(&mut self, atom_types: Vec<T>) {
        let num_types = atom_types.len();
        self.atoms.set_atom_types(atom_types);
        self.atomic_potential.set_num_types(num_types);
    }
    pub fn set_atomic_potential(&mut self, atomic_potential: A) {
        if self.atomic_potential.cutoff_distance() != atomic_potential.cutoff_distance() {
            self.neighbor_list
                .set_force_distance(atomic_potential.cutoff_distance());
        }
        self.atomic_potential = atomic_potential;
    }
    pub fn set_timestep(&mut self, timestep: f64) {
        assert!(
            timestep > 0.0,
            "Timestep should be positive, found {}",
            timestep,
        );
        self.timestep = timestep;
    }

    // Other public functions
    pub fn run(&mut self, num_steps: usize) {
        self.pre_check();

        self.initial_output();

        self.pre_forward_comm();
        self.forward_comm();
        self.post_forward_comm();
        self.build_neighbor_list();
        dbg!(self.atoms.positions.len());
        dbg!(self.neighbor_list.neighbors().len());
        self.forces = self.compute_forces();
        self.reverse_comm();
        self.check_do_output(&0);

        for step in 1..=num_steps {
            self.pre_forward_comm();
            self.forward_comm();
            self.post_forward_comm();

            self.check_build_neighbor_list(step);
            dbg!(self.atoms.positions.len());
            dbg!(self.neighbor_list.neighbors().len());

            self.pre_force();
            self.forces = self.compute_forces();
            self.pre_reverse_comm();
            self.reverse_comm();
            self.post_reverse_comm();

            self.check_do_output(&step);
        }
    }

    // Communication functions
    fn forward_comm(&mut self) {
        comm::forward_comm(self);
    }
    fn reverse_comm(&mut self) {
        comm::reverse_comm(self);
    }

    // Run methods
    fn pre_check(&self) {
        assert!(
            self.atomic_potential.all_set(),
            "All atomic potential coefficients should be set before running"
        );
    }
    fn pre_forward_comm(&mut self) {
        Verlet::pre_forward_comm(self);
    }
    fn post_forward_comm(&mut self) {}
    fn pre_force(&mut self) {}
    fn pre_reverse_comm(&mut self) {}
    fn post_reverse_comm(&mut self) {}
    fn compute_forces(&self) -> Vec<[f64; 3]> {
        self.atomic_potential
            .compute_forces(&self.atoms, &self.neighbor_list)
    }

    fn check_build_neighbor_list(&mut self, step: usize) {
        if !self.neighbor_list.should_update(step) {
            return;
        }
        if self.neighbor_list.check() && !self.atoms_moved_too_far() {
            return;
        }
        self.build_neighbor_list();
    }

    fn build_neighbor_list(&mut self) {
        if !self.neighbor_list.is_built() {
            self.neighbor_list.update(self.atoms.positions());
        }
        self.wrap_pbs();
        comm::comm_atom_ownership(self);
        self.neighbor_list.update(self.atoms.positions());
        self.pos_at_prev_neigh_build = self.atoms.positions.clone();
    }

    // TODO: Move to output
    fn check_do_output(&self, step: &usize) {
        if step % self.output.every != 0 {
            return;
        }

        for v in &self.output.values {
            let value = match v {
                output::OutputSpec::Step => Value::Usize(*step),
                output::OutputSpec::Compute(c) => c.compute(&self),
            };
            self.domain
                .send_to_main(msg::W2M::Output(thread::current().id(), value));
        }
    }

    // Private functions
    fn atoms_moved_too_far(&self) -> bool {
        let half_skin_dist = self.neighbor_list.skin_distance() * 0.5;
        let opt = self
            .pos_at_prev_neigh_build
            .iter()
            .zip(self.atoms.positions().iter())
            .map(|(old, new)| {
                let [dx, dy, dz] = [new[0] - old[0], new[1] - old[1], new[2] - old[2]];
                dx * dx + dy * dy + dz * dz
            })
            .reduce(f64::max);

        match opt {
            Some(max_dist_sq) => max_dist_sq > half_skin_dist * half_skin_dist,
            None => false,
        }
    }

    fn wrap_pbs(&mut self) {
        if self.container().is_periodic(Axis::X) {
            let [lo, hi] = [self.container().xlo(), self.container().xhi()];
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[0] < lo {
                    p[0] += hi - lo;
                } else if p[0] > hi {
                    p[0] -= hi - lo;
                }
            });
        }
        if self.container().is_periodic(Axis::Y) {
            let [lo, hi] = [self.container().ylo(), self.container().yhi()];
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[1] < lo {
                    p[1] += hi - lo;
                } else if p[1] > hi {
                    p[1] -= hi - lo;
                }
            });
        }
        if self.container().is_periodic(Axis::Z) {
            let [lo, hi] = [self.container().zlo(), self.container().zhi()];
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[2] < lo {
                    p[2] += hi - lo;
                } else if p[2] > hi {
                    p[2] -= hi - lo;
                }
            });
        }
    }

    fn initial_output(&self) {
        self.domain().send_to_main_once(msg::W2M::InitialOutput);
    }
}
