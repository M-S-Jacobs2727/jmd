use std::{rc::Rc, thread};

use rand_distr::Distribution;

use crate::{
    atom_type::AtomType,
    atomic::AtomicPotentialTrait,
    atoms::Atoms,
    compute::{Compute, ComputeTrait},
    container::{Container, BC},
    integrators::{Integrator, Verlet},
    neighbor::NeighborList,
    output::{Output, OutputSpec, Value},
    parallel::{comm, message as msg, Domain, Worker},
    region::{Rect, Region},
    utils::{Axis, KeyedVec},
};
type ComputeVec = KeyedVec<String, Compute>;

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
    A: AtomicPotentialTrait<T>,
{
    pub(crate) atoms: Atoms<T>,
    container: Rc<Container>,
    atomic_potential: A,
    neighbor_list: NeighborList,
    domain: Domain<'a, T, A>,
    output: Output,
    pos_at_prev_nl_build: Vec<[f64; 3]>,
    computes: ComputeVec,
    timestep: f64,
    forces: Vec<[f64; 3]>,
    nl_update_settings: NLUpdateSettings,
}
impl<'a, T, A> Simulation<'a, T, A>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
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
            output: Output::new(),
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
    pub fn neighbor_list(&self) -> &NeighborList {
        &self.neighbor_list
    }
    /// Alias for `Simulation::neighbor_list()`
    pub fn nl(&self) -> &NeighborList {
        &self.neighbor_list
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
    pub fn add_compute(&mut self, id: &str, compute: Compute) {
        self.computes.add(String::from(id), compute)
    }
    /// Set the list of atom types
    /// TODO: Check if this needs to include side effects
    pub fn set_atom_types(&mut self, atom_types: Vec<T>) {
        let num_types = atom_types.len();
        self.atoms.atom_types = atom_types;
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

    pub fn set_atomic_coeff(&mut self, typei: usize, typej: usize, coeff: &A::Coeff) {
        self.atomic_potential.set_coeff(typei, typej, coeff);
    }

    // Neighbor list methods

    pub fn set_nl_update(&mut self, every: usize, delay: usize, check: bool) {
        self.nl_update_settings = NLUpdateSettings {
            every,
            delay,
            check,
            last_update_step: 0,
        };
    }
    pub fn set_nl_skin_distance(&mut self, skin_distance: f64) {
        self.neighbor_list.set_skin_distance(skin_distance);
    }

    // Atoms methods

    /// Add a given number of atoms of the given type with the given region
    pub fn add_random_atoms(&mut self, rect: &Rect, num_atoms: usize, atom_type: usize) {
        let sub_region = rect.intersect(self.domain.subdomain());
        let mut my_natoms =
            (sub_region.volume() / rect.volume() * num_atoms as f64).floor() as usize;
        self.domain.send_to_main(msg::W2M::Sum(my_natoms));
        let message = self.domain.recv_from_main();
        let added_natoms = match message {
            msg::M2W::SumResult(sum) => sum,
            _ => panic!("Invalid message"),
        };
        if self.domain.proc_index() < num_atoms - added_natoms {
            my_natoms += 1;
        }

        let atoms = &mut self.atoms;
        let atom_id = match atoms.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        atoms.ids.extend(atom_id..atom_id + my_natoms);
        atoms.types.reserve(my_natoms);
        atoms.positions.reserve(my_natoms);
        atoms.velocities.reserve(my_natoms);
        atoms.nlocal += my_natoms;

        for _i in 0..my_natoms {
            atoms.types.push(atom_type);
            atoms.velocities.push([0.0, 0.0, 0.0]);
            atoms.positions.push(sub_region.get_random_coord())
        }
    }
    /// Add atoms of the given type at the given coordinates
    pub fn add_atoms(&mut self, atom_type: usize, coords: Vec<[f64; 3]>) {
        let atoms = &mut self.atoms;
        let num_atoms = coords.len();
        let atom_id = match atoms.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };

        atoms.ids.reserve(num_atoms);
        atoms.types.reserve(num_atoms);
        atoms.positions.reserve(num_atoms);
        atoms.velocities.reserve(num_atoms);

        let mut atoms_added = 0;
        coords
            .iter()
            .enumerate()
            .filter(|(_i, coord)| self.domain.subdomain().contains(coord))
            .for_each(|(i, coord)| {
                atoms_added += 1;
                atoms.ids.push(atom_id + i);
                atoms.types.push(atom_type);
                atoms.positions.push(*coord);
                atoms.velocities.push([0.0, 0.0, 0.0]);
            });
    }
    /// Set the temperature
    /// TODO: move to simulation, make work across multiple processes
    pub fn set_temperature(&mut self, temperature: f64) {
        let atoms = &mut self.atoms;
        let mut rng = rand::thread_rng();
        let dist = rand_distr::Normal::new(0.0, temperature.sqrt()).expect("Invalid temperature");
        let sqrt_ke: Vec<f64> = dist.sample_iter(&mut rng).take(atoms.nlocal * 3).collect();
        for i in 0..atoms.nlocal {
            atoms.velocities[i] = [
                sqrt_ke[3 * i + 0] / atoms.atom_types[atoms.types[i]].mass().sqrt(),
                sqrt_ke[3 * i + 1] / atoms.atom_types[atoms.types[i]].mass().sqrt(),
                sqrt_ke[3 * i + 2] / atoms.atom_types[atoms.types[i]].mass().sqrt(),
            ];
        }
    }
    /// Remove atoms at the given indices
    /// TODO: change to IDs instead, add convenience functions for regions
    pub(crate) fn remove_idxs(&mut self, atom_idxs: Vec<usize>) {
        let atoms = &mut self.atoms;
        let num_local = atom_idxs.iter().filter(|&i| *i < atoms.nlocal).count();
        atoms.nlocal -= num_local;
        fn filter_by_idx<T: Copy>(atom_idxs: &Vec<usize>, vec: &Vec<T>) -> Vec<T> {
            vec.iter()
                .enumerate()
                .filter_map(|(i, x)| {
                    if atom_idxs.contains(&i) {
                        None
                    } else {
                        Some(*x)
                    }
                })
                .collect()
        }

        atoms.ids = filter_by_idx(&atom_idxs, &atoms.ids);
        atoms.types = filter_by_idx(&atom_idxs, &atoms.types);
        atoms.positions = filter_by_idx(&atom_idxs, &atoms.positions);
        atoms.velocities = filter_by_idx(&atom_idxs, &atoms.velocities);
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
    fn post_reverse_comm(&mut self) {
        Verlet::post_reverse_comm(self);
    }

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
                OutputSpec::Step => Value::Usize(step),
                OutputSpec::Compute(c) => c.compute(&self),
            };
            self.domain
                .send_to_main(msg::W2M::Output(thread::current().id(), value));
        }
    }
    fn initial_output(&self) {
        self.domain().send_to_main_once(msg::W2M::InitialOutput);
    }
}
