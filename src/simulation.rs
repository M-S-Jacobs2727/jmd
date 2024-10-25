use crate::{
    atomic, compute, output,
    parallel::{
        comm,
        message::{self as msg},
        Domain, Worker,
    },
    Atoms, Axis, Container, Error, NeighborList, OutputSpec, BC,
};

pub struct Simulation<'a> {
    pub atoms: Atoms,
    container: Container,
    atomic_potential: Box<dyn atomic::AtomicPotential>,
    neighbor_list: NeighborList,
    domain: Domain<'a>,
    output: output::Output,
    nlocal: usize,
    pos_at_prev_neigh_build: Vec<[f64; 3]>,
    computes: Vec<Box<dyn compute::Compute>>,
}
impl<'a> Simulation<'a> {
    pub fn new() -> Self {
        let container = Container::new(0., 10., 0.0, 10.0, 0.0, 10.0, BC::PP, BC::PP, BC::PP);
        let neighbor_list = NeighborList::new(&container, 1.0, 1.0, 1.0);
        Self {
            atoms: Atoms::new(),
            container,
            atomic_potential: Box::new(atomic::None_::new()),
            neighbor_list,
            domain: Domain::new(),
            output: output::Output::new(),
            nlocal: 0,
            pos_at_prev_neigh_build: Vec::new(),
            computes: Vec::new(),
        }
    }

    /// Initializes the simulation from a worker thread
    pub(crate) fn init(&mut self, worker: Box<&'a Worker>) {
        self.domain.init(&self.container, worker);
    }

    // Getters
    pub fn container(&self) -> &Container {
        &self.container
    }
    pub fn atomic_potential(&self) -> &Box<dyn atomic::AtomicPotential> {
        &self.atomic_potential
    }
    pub fn neighbor_list(&self) -> &NeighborList {
        &self.neighbor_list
    }
    pub fn domain(&self) -> &Domain {
        &self.domain
    }
    pub fn nlocal(&self) -> usize {
        self.nlocal
    }
    pub fn computes(&self) -> &Vec<Box<dyn compute::Compute>> {
        &self.computes
    }

    // Setters
    pub fn set_atom_types(&mut self, atom_types: usize) -> Result<(), Error> {
        self.atomic_potential.set_num_types(atom_types)
    }
    pub fn set_container(&mut self, container: Container) {
        self.container = container;
        self.domain.reset_subdomain(&self.container);
    }
    pub fn set_atomic_potential(
        &mut self,
        atomic_potential: impl atomic::AtomicPotential + 'static,
    ) {
        self.atomic_potential = Box::new(atomic_potential);
        let force_distance = self.atomic_potential.cutoff_distance();
        let skin_distance = 1.0;
        let bin_size = (force_distance + skin_distance) * 0.5;
        self.neighbor_list =
            NeighborList::new(self.container(), bin_size, force_distance, skin_distance);
    }
    pub fn set_neighbor_list(&mut self, neighbor_list: NeighborList) {
        self.neighbor_list = neighbor_list;
    }
    pub fn set_domain(&mut self, domain: Domain<'a>) {
        self.domain = domain;
    }
    pub fn set_output(&mut self, output: output::Output) {
        self.output = output.clone();
        for o in &output.values {
            match o {
                OutputSpec::Step => {}
                OutputSpec::KineticE => self.add_compute(compute::KineticEnergy {}),
                OutputSpec::PotentialE => self.add_compute(compute::PotentialEnergy {}),
                OutputSpec::Temp => self.add_compute(compute::Temperature {}),
                OutputSpec::TotalE => self.add_compute(compute::TotalEnergy {}),
            }
        }
        self.domain
            .send_to_main(msg::W2M::SetupOutput(output.values));
    }
    pub(crate) fn increment_nlocal(&mut self) {
        self.nlocal += 1;
    }
    pub fn add_compute(&mut self, compute: impl compute::Compute + 'static) {
        self.computes.push(Box::new(compute));
    }
    pub fn add_computes(&mut self, mut computes: Vec<Box<dyn compute::Compute + 'static>>) {
        self.computes.append(&mut computes);
    }

    pub(crate) fn forward_comm(&mut self) {
        comm::forward_comm(self);
    }
    pub(crate) fn reverse_comm(&self, forces: &mut Vec<[f64; 3]>) {
        comm::reverse_comm(self, forces);
    }

    pub(crate) fn compute_forces(&self) -> Vec<[f64; 3]> {
        self.atomic_potential.compute_forces(self)
    }

    pub(crate) fn check_build_neighbor_list(&mut self, step: &usize) {
        if !self
            .neighbor_list
            .update_settings
            .should_update_neighbors(*step)
        {
            return;
        }
        if self.neighbor_list.update_settings.check && !self.atoms_moved_too_far() {
            return;
        }
        self.build_neighbor_list();
    }

    pub(crate) fn build_neighbor_list(&mut self) {
        dbg!(self.atoms.positions());
        self.wrap_pbs();
        comm::comm_atom_ownership(self);
        self.neighbor_list.update(self.atoms.positions());
        self.pos_at_prev_neigh_build = self.atoms.positions.clone();
    }

    // TODO: Move to output
    pub(crate) fn check_do_output(&self, step: &usize) {
        if step % self.output.every != 0 {
            return;
        }

        let ke = if self.output.values.contains(&output::OutputSpec::KineticE)
            || self.output.values.contains(&output::OutputSpec::Temp)
            || self.output.values.contains(&output::OutputSpec::TotalE)
        {
            Some(self.compute_local_ke())
        } else {
            None
        };

        let pe = if self.output.values.contains(&output::OutputSpec::PotentialE)
            || self.output.values.contains(&output::OutputSpec::TotalE)
        {
            Some(self.compute_local_pe())
        } else {
            None
        };

        for v in &self.output.values {
            let (value, op) = match v {
                output::OutputSpec::Step => (output::Value::Usize(*step), output::Operation::First),
                output::OutputSpec::Temp => (
                    output::Value::Float(ke.unwrap_or(0.0) * 3.0 / self.atoms.num_atoms() as f64),
                    output::Operation::Sum,
                ),
                output::OutputSpec::KineticE => (
                    output::Value::Float(ke.unwrap_or(0.0)),
                    output::Operation::Sum,
                ),
                output::OutputSpec::PotentialE => (
                    output::Value::Float(pe.unwrap_or(0.0)),
                    output::Operation::Sum,
                ),
                output::OutputSpec::TotalE => (
                    output::Value::Float(ke.unwrap_or(0.0) + pe.unwrap_or(0.0)),
                    output::Operation::Sum,
                ),
            };
            self.domain
                .send_to_main(msg::W2M::Output(output::OutputMessage::new(value, op)));
        }
    }

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
}
