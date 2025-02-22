use crate::atom::Atom;
use crate::constraint::Constraint;
use crate::domain::Domain;
use crate::neighbor::{NeighborList, NeighborListSettings};

pub struct EngineBuilder {
    domain: Option<Domain>,
    neighbor_list_settings: Option<NeighborListSettings>,
}

pub struct Engine {
    domain: Domain,
    atoms: Vec<Atom>,
    constraints: Vec<Box<dyn Constraint>>,
    neighbor_list: NeighborList,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self {
            domain: None,
            neighbor_list_settings: None,
        }
    }

    pub fn with_domain(mut self, domain: Domain) -> Self {
        self.domain = Some(domain);
        self
    }

    pub fn with_neighbor_list_settings(mut self, settings: NeighborListSettings) -> Self {
        self.neighbor_list_settings = Some(settings);
        self
    }

    pub fn build(self) -> Result<Engine, &'static str> {
        // Validate parameters
        if self.domain.is_none() {
            return Err("Domain must be set");
        }
        if self.neighbor_list_settings.is_none() {
            return Err("Neighbor list settings must be set");
        }

        Ok(Engine {
            domain: self.domain.unwrap(),
            atoms: Vec::new(),
            constraints: Vec::new(),
            neighbor_list: NeighborList::new(self.neighbor_list_settings.unwrap()),
        })
    }
}

impl Engine {
    /// Adds a force to the engine.
    ///
    /// # Arguments
    ///
    /// * `force` - A force to be added to the engine.
    pub fn add_force(&mut self, force: Box<dyn Constraint>) {
        self.constraints.push(force);
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
        let max_id = self.atoms.iter().map(|atom| atom.id).max().unwrap_or(0);
        self.atoms.reserve(positions.len());
        for (i, &position) in positions.iter().enumerate() {
            self.atoms.push(Atom {
                id: max_id + i,
                position,
                velocity: [0.0, 0.0, 0.0],
                force: [0.0, 0.0, 0.0],
                mass,
                atom_type,
            });
        }
    }
    /// Runs the engine for a specified number of steps.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to run the engine for.
    pub fn run(&mut self, steps: usize) {
        self.forward_communication();
        self.build_neighbor_list(0);
        self.compute_force();
        self.reverse_communication();
        self.output(0);
        for step in 1..=steps {
            self.pre_forward_communication();
            self.forward_communication();
            self.post_forward_communication();

            self.build_neighbor_list(step);

            self.pre_compute_force();
            self.compute_force();

            self.pre_reverse_communication();
            self.reverse_communication();
            self.post_reverse_communication();

            self.output(step);
        }
    }
    /// Iterates velocities and positions before forward communication.
    /// Also includes other events occuring before forward communication.
    fn pre_forward_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.pre_forward_communication(&mut self.atoms, &mut self.domain);
        }
    }
    /// Forward communication of positions and velocities.
    fn forward_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.forward_communication(&mut self.atoms, &mut self.domain);
        }
    }
    /// Events occuring after position and velocity communication.
    fn post_forward_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.post_forward_communication(&mut self.atoms, &mut self.domain);
        }
    }
    /// Builds the neighbor list if the step is a multiple of the update frequency.
    fn build_neighbor_list(&mut self, step: usize) {
        if step % self.neighbor_list.settings.update_every == 0 {
            self.neighbor_list.generate(&self.atoms);
        }
    }
    /// Events occuring before force computation.
    fn pre_compute_force(&mut self) {
        for constraint in &mut self.constraints {
            constraint.pre_compute_force(&mut self.atoms, &mut self.domain);
        }
    }
    /// Iterates velocities before reverse communication.
    /// Also includes other events occuring before reverse communication.
    fn pre_reverse_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.pre_reverse_communication(&mut self.atoms, &mut self.domain);
        }
    }
    /// Reverse communication of forces.
    fn reverse_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.reverse_communication(&mut self.atoms, &mut self.domain);
        }
    }
    /// Events occuring after reverse communication.
    fn post_reverse_communication(&mut self) {
        for constraint in &mut self.constraints {
            constraint.post_reverse_communication(&mut self.atoms, &mut self.domain);
        }
    }
    fn compute_force(&mut self) {
        for constraint in &mut self.constraints {
            constraint.compute_force(&mut self.atoms, &mut self.domain);
        }
    }
    fn output(&self, step: usize) {}
}
