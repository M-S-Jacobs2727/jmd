use crate::prelude::*;
use crate::region::*;
use crate::{Atoms, Domain, Lattice, NeighborList, NeighborListSettings};

pub struct Simulation {
    atoms: Atoms,
    domain: Domain,
    integrator: Box<dyn Integrator>,
    neighbor_list: NeighborList,
    pairwise: Vec<Box<dyn Pairwise>>,
    constraints: Vec<Box<dyn Constraint>>,
}

impl Simulation {
    pub fn atoms(&self) -> &Atoms {
        &self.atoms
    }
    pub fn domain(&self) -> &Domain {
        &self.domain
    }
    pub fn integrator(&self) -> &Box<dyn Integrator> {
        &self.integrator
    }
    pub fn neighbor_list_settings(&self) -> &NeighborListSettings {
        &self.neighbor_list.settings
    }
    //pub(crate) fn neighbor_list(&self) -> &NeighborList {
    //    &self.neighbor_list
    //}
    pub fn pairwise(&self) -> &Vec<Box<dyn Pairwise>> {
        &self.pairwise
    }
    pub fn mut_pairwise(&mut self) -> &mut Vec<Box<dyn Pairwise>> {
        &mut self.pairwise
    }
    pub fn constraints(&self) -> &Vec<Box<dyn Constraint>> {
        &self.constraints
    }
    pub fn mut_constraints(&mut self) -> &mut Vec<Box<dyn Constraint>> {
        &mut self.constraints
    }
    pub fn set_velocities(&mut self, vx: f64, vy: f64, vz: f64) {
        self.atoms.velocities.iter_mut().for_each(|v| {*v = [vx, vy, vz];});
    }

    /// Adds atoms at specified coordinates.
    ///
    /// # Arguments
    ///
    /// * `positions` - A vector of coordinates for the atoms.
    /// * `mass` - The mass of the atoms.
    /// * `atom_type` - The type of the atoms.
    pub fn add_atoms_at_coordinates(
        &mut self,
        positions: Vec<[f64; 3]>,
        mass: f64,
        atom_type: i64,
    ) {
        let max_id: usize = self
            .atoms
            .ids
            .iter()
            .max()
            .and_then(|m| Some(*m))
            .unwrap_or(0usize);
        self.atoms.reserve(positions.len());
        for (i, &position) in positions.iter().enumerate() {
            self.atoms.push(
                max_id + i + 1,
                atom_type,
                mass,
                position,
                [0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
            );
        }
    }
    pub fn generate_neighbor_list(&mut self) {
        self.neighbor_list.generate(&self.atoms);
    }
    pub fn create_atoms_on_lattice(
        &mut self,
        lattice: &Lattice,
        atom_type: i64,
        mass: f64,
        region: &Rect,
    ) {
        let (lx, ly, lz) = (region.lx(), region.ly(), region.lz());
        let nx = (lx / lattice.lx()).round() as u32;
        let ny = (ly / lattice.ly()).round() as u32;
        let nz = (lz / lattice.lz()).round() as u32;
        let offsets = lattice.offsets();

        let max_id: usize = self
            .atoms
            .ids
            .iter()
            .max()
            .and_then(|m| Some(*m))
            .unwrap_or(0usize);
        let mut id = max_id;

        for k in 0..nz {
            let lat_z = region.zlo() + (k as f64) * lattice.lz();
            for j in 0..ny {
                let lat_y = region.ylo() + (j as f64) * lattice.ly();
                for i in 0..nx {
                    let lat_x = region.xlo() + (i as f64) * lattice.lx();
                    for offset in &offsets {
                        id += 1;
                        let position = [lat_x + offset[0], lat_y + offset[1], lat_z + offset[2]];
                        if !self.domain.region().contains(position) {
                            panic!("Cannot create atoms outside of simulation box")
                        }
                        self.atoms.push(
                            id,
                            atom_type,
                            mass,
                            position,
                            [0.0, 0.0, 0.0],
                            [0.0, 0.0, 0.0],
                        )
                    }
                }
            }
        }
    }
    /// Adds a pairwise force to the simulation.
    ///
    /// # Arguments
    ///
    /// * `pairwise` - A pairwise force to be added to the simulation.
    pub fn add_pairwise<T>(&mut self, pairwise: T)
    where
        T: Sized + Pairwise + 'static,
    {
        self.pairwise.push(Box::new(pairwise));
    }
    /// Adds a constraint to the simulation.
    ///
    /// # Arguments
    ///
    /// * `constraint` - A Constraint to be added to the simulation.
    pub fn add_constraint<T>(&mut self, constraint: T)
    where
        T: Sized + Constraint + 'static,
    {
        self.constraints.push(Box::new(constraint));
    }
    /// Runs the simulation for a specified number of steps.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to run the simulation for.
    pub fn run(&mut self, steps: usize) {
        self.forward_communication();
        self.generate_neighbor_list();
        self.compute_pairwise();
        self.reverse_communication();
        self.output(0);

        for step in 1..=steps {
            let atoms = &mut self.atoms;
            let domain = &mut self.domain;
        
            self.integrator.initial_integrate(atoms, domain);

            self.maybe_rebuild_neighbor_list(step);

            self.atoms.zero_force();
            self.pre_compute();

            self.compute_pairwise();

            self.pre_reverse_communication();
            self.reverse_communication();

            self.post_compute();

            let atoms = &mut self.atoms;
            let domain = &mut self.domain;
            self.integrator.final_integrate(atoms, domain);
            self.end_of_step();

            self.output(step);
        }
    }
    /// Builds the neighbor list if the step is a multiple of the update frequency.
    fn maybe_rebuild_neighbor_list(&mut self, step: usize) {
        if self.neighbor_list.should_rebuild(step, &self.atoms) {
            self.forward_communication();
            return;
        }
        self.generate_neighbor_list();
    }
    /// Forward communication of positions and velocities.
    fn forward_communication(&mut self) {
    }
    /// Events occuring before pairwise computation.
    fn pre_compute(&mut self) {
        let atoms = &mut self.atoms;
        let domain = &mut self.domain;
        for constraint in &mut self.constraints {
            constraint.pre_compute(atoms, domain);
        }
    }
    fn compute_pairwise(&mut self) {
        let atoms = &mut self.atoms;
        for pairwise in &mut self.pairwise {
            pairwise.compute_pairwise(atoms);
        }
    }
    /// Also includes other events occuring before reverse communication.
    fn pre_reverse_communication(&mut self) {
        let atoms = &mut self.atoms;
        let domain = &mut self.domain;
        for constraint in &mut self.constraints {
            constraint.pre_reverse_communication(atoms, domain);
        }
    }
    fn post_compute(&mut self) {
        let atoms = &mut self.atoms;
        let domain = &mut self.domain;
        for constraint in &mut self.constraints {
            constraint.post_compute(atoms, domain);
        }

    }
    /// Reverse communication of forces.
    fn reverse_communication(&mut self) {
    }
    fn end_of_step(&mut self) {

    }
    fn output(&self, _step: usize) {}
}

pub struct SimulationBuilder {
    simulation_box: Option<Rect>,
    pbcs: Option<[BoundaryCondition; 3]>,
    integrator: Option<Box<dyn Integrator>>,
    nl_settings: Option<NeighborListSettings>,
}
impl SimulationBuilder {
    pub fn new() -> Self {
        Self {
            simulation_box: None,
            pbcs: None,
            integrator: None,
            nl_settings: None,
        }
    }
    pub fn with_box(mut self, simulation_box: Rect) -> Self {
        self.simulation_box = Some(simulation_box);
        self
    }
    pub fn with_pbc(
        mut self,
        xbc: BoundaryCondition,
        ybc: BoundaryCondition,
        zbc: BoundaryCondition,
    ) -> Self {
        self.pbcs = Some([xbc, ybc, zbc]);
        self
    }
    pub fn with_integrator<T>(mut self, integrator: T) -> Self
    where
        T: Sized + Integrator + 'static,
    {
        self.integrator = Some(Box::new(integrator));
        self
    }
    pub fn with_neighbor_list(mut self, settings: NeighborListSettings) -> Self {
        self.nl_settings = Some(settings);
        self
    }
    pub fn build(self) -> Simulation {
        let simulation_box = match self.simulation_box {
            Some(b) => b,
            None => panic!("Simulation box should be set"),
        };
        let pbcs = match self.pbcs {
            Some(p) => p,
            None => panic!("Boundary conditions should be set"),
        };
        let domain = Domain::new(simulation_box, pbcs);

        let integrator = match self.integrator {
            Some(i) => i,
            None => panic!("Integrator should be set"),
        };
        let neighbor_list = match self.nl_settings {
            Some(s) => s.neighbor_list(&domain),
            None => panic!("Neighbor list should be set"),
        };

        Simulation {
            atoms: Atoms::new(),
            domain,
            integrator,
            neighbor_list,
            constraints: vec![],
            pairwise: vec![],
        }
    }
}
