use std::{rc::Rc, thread};

use crate::{
    atom_type::AtomType,
    atomic,
    compute::{self, ComputeTrait},
    output::{self, Value},
    parallel::{comm, message as msg, Domain, Worker},
    utils::KeyedVec,
    Atoms, Axis, Container, Integrator, NeighborList, Output, OutputSpec, Verlet, BC,
};
type ComputeVec = KeyedVec<String, compute::Compute>;

struct NLUpdateSettings {
    pub last_update_step: usize,
    pub every: usize,
    pub delay: usize,
    pub check: bool,
}

/// The main simulation class in JMD, with one copy held by each process.
pub struct Simulation<'a, T, A>
where
    T: AtomType,
    A: atomic::AtomicPotentialTrait<T>,
{
    pub atoms: Atoms<T>,
    container: Rc<Container>,
    atomic_potential: A,
    pub neighbor_list: NeighborList,
    domain: Domain<'a, T, A>,
    output: output::Output,
    pos_at_prev_nl_build: Vec<[f64; 3]>,
    computes: ComputeVec,
    timestep: f64,
    forces: Vec<[f64; 3]>,
    nl_update_settings: NLUpdateSettings,
}
impl<'a, T, A> Simulation<'a, T, A>
where
    T: AtomType,
    A: atomic::AtomicPotentialTrait<T>,
{
    /// Create a new simulation
    pub fn new() -> Self {
        let timestep = 1.0;
        let container = Rc::new(Container::new(
            0.0,
            1.0,
            0.0,
            1.0,
            0.0,
            1.0,
            BC::PP,
            BC::PP,
            BC::PP,
        ));
        let atomic_potential = A::new();
        let neighbor_list =
            NeighborList::new(container.clone(), atomic_potential.cutoff_distance(), 1.0);
        Self {
            atoms: Atoms::new(),
            container,
            atomic_potential,
            neighbor_list,
            domain: Domain::new(),
            output: output::Output::new(),
            pos_at_prev_nl_build: Vec::new(),
            computes: KeyedVec::new(),
            timestep,
            forces: Vec::new(),
            nl_update_settings: NLUpdateSettings {
                last_update_step: 0,
                every: 1,
                delay: 0,
                check: true,
            },
        }
    }

    /// Initializes the simulation from a worker thread
    pub fn connect(&mut self, worker: Box<&'a Worker<T, A>>) {
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
    pub(crate) fn domain(&self) -> &Domain<T, A> {
        &self.domain
    }
    pub(crate) fn nlocal(&self) -> usize {
        self.atoms.nlocal
    }
    pub fn computes(&self) -> &ComputeVec {
        &self.computes
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
        self.domain.reset_subdomain(&self.container.rect());
        self.neighbor_list = NeighborList::new(
            self.container.clone(),
            self.atomic_potential.cutoff_distance(),
            self.neighbor_list.skin_distance(),
        );
    }
    pub fn set_output(&mut self, every: usize, output_keys: Vec<&str>) {
        let output_specs: Vec<OutputSpec> = output_keys
            .iter()
            .map(|&key| {
                if key == "step" {
                    OutputSpec::Step
                } else {
                    let c = self
                        .computes
                        .get(&String::from(key))
                        .expect("Invalid compute id");
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
    pub fn set_nl_update(&mut self, every: usize, delay: usize, check: bool) {
        self.nl_update_settings = NLUpdateSettings {
            every,
            delay,
            check,
            last_update_step: 0,
        };
    }

    // Other public functions
    pub fn run(&mut self, num_steps: usize) {
        self.pre_check();

        self.initial_output();

        for step in 0..=num_steps {
            // Forward communication
            self.pre_forward_comm();
            self.forward_comm();
            self.post_forward_comm();

            // Build neighbor list if applicable
            self.check_build_neighbor_list(step);

            // Compute forces
            self.pre_force();
            self.forces = self.compute_forces();

            // Reverse communication
            self.pre_reverse_comm();
            self.reverse_comm();
            self.post_reverse_comm();

            // Output
            self.check_do_output(step);
        }
    }

    // Run methods
    /// Check that all settings are appropriate and agreeable between all parts
    /// of the simulation
    fn pre_check(&self) {
        assert!(
            self.atomic_potential.all_set(),
            "All atomic potential coefficients should be set before running"
        );
    }
    fn pre_forward_comm(&mut self) {
        Verlet::pre_forward_comm(self);
    }
    /// Forward communication: communicating the details of owned atoms to neighboring
    /// processes to use as ghost atoms.
    fn forward_comm(&mut self) {
        comm::forward_comm(self);
    }
    fn post_forward_comm(&mut self) {}
    fn pre_force(&mut self) {}
    /// Compute the atomic potential, etc. forces acting on the atoms
    fn compute_forces(&self) -> Vec<[f64; 3]> {
        self.atomic_potential
            .compute_forces(&self.atoms, &self.neighbor_list)
    }
    fn pre_reverse_comm(&mut self) {}
    /// Reverse communication: communicating the forces of ghost atoms back to the owning
    /// processes
    fn reverse_comm(&mut self) {
        comm::reverse_comm(self);
    }
    fn post_reverse_comm(&mut self) {}

    // Neighbor list methods
    /// Whether the neighbor list should update on a given step.
    ///
    /// If number of steps since last update is not a multiple of nevery, then false.
    /// Else if number of steps since last update < delay, then false.
    /// Else if check is false, then true.
    /// Else if atoms have moved too far, then true.
    /// Else, false.
    fn nl_should_update(&self, step: usize) -> bool {
        let steps_since_last = step - self.nl_update_settings.last_update_step;
        (steps_since_last % self.nl_update_settings.every == 0)  // Step is a multiple of every
            && (steps_since_last >= self.nl_update_settings.delay)  // It has been longer than delay since last update
            && (!self.nl_update_settings.check || self.atoms_moved_too_far()) // if check and atoms moved too far, or if check is false
    }
    /// If the neighbor list has not been built or should be rebuilt, then build it
    fn check_build_neighbor_list(&mut self, step: usize) {
        if !self.neighbor_list.is_built() || self.nl_should_update(step) {
            self.build_neighbor_list();
        }
    }
    /// If this is the first build, then build first.
    /// Wrap the atoms across periodic boundaries, communicate the new atom ownerships,
    /// update the neighbor list, and save the positions to compare against in the future.
    fn build_neighbor_list(&mut self) {
        if !self.neighbor_list.is_built() {
            self.neighbor_list.update(self.atoms.positions());
        }
        self.wrap_pbs();
        comm::comm_atom_ownership(self);
        self.neighbor_list.update(self.atoms.positions());
        self.pos_at_prev_nl_build = self.atoms.positions.clone();
    }
    /// Whether any atom has moved further than half the skin distance
    fn atoms_moved_too_far(&self) -> bool {
        let half_skin_dist = self.neighbor_list.skin_distance() * 0.5;
        let opt = self
            .pos_at_prev_nl_build
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
    /// Wrap atoms across periodic boundary conditions
    /// TODO: increment periodic image flags
    fn wrap_pbs(&mut self) {
        let rect = self.container().rect().clone();

        vec![Axis::X, Axis::Y, Axis::Z]
            .iter()
            .enumerate()
            .for_each(|(i, &axis)| {
                if self.container().is_periodic(axis) {
                    let [lo, hi] = rect.get_bounds(axis);
                    self.atoms.positions.iter_mut().for_each(|p| {
                        if p[i] < lo {
                            p[i] += hi - lo;
                        } else if p[i] > hi {
                            p[i] -= hi - lo;
                        }
                    });
                }
            });
    }

    // Output methods
    // TODO: Move to output
    fn check_do_output(&self, step: usize) {
        if step % self.output.every != 0 {
            return;
        }

        for v in &self.output.values {
            let value = match v {
                output::OutputSpec::Step => Value::Usize(step),
                output::OutputSpec::Compute(c) => c.compute(&self),
            };
            self.domain
                .send_to_main(msg::W2M::Output(thread::current().id(), value));
        }
    }
    fn initial_output(&self) {
        self.domain().send_to_main_once(msg::W2M::InitialOutput);
    }
}
