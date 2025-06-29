use crate::{Atom, NeighborList, NeighborListSettings, SimulationBox};

pub struct SystemBuilder {
    simulation_box: Option<SimulationBox>,
    neighbor_list_settings: Option<NeighborListSettings>,
}

pub struct System {
    pub atoms: Vec<Atom>,
    pub(crate) simulation_box: SimulationBox,
    pub(crate) neighbor_list: NeighborList,
}

impl SystemBuilder {
    pub fn new() -> Self {
        Self {
            simulation_box: None,
            neighbor_list_settings: None,
        }
    }
    pub fn with_simulation_box(mut self, simulation_box: SimulationBox) -> Self {
        self.simulation_box = Some(simulation_box);
        self
    }
    pub fn with_neighbor_list_settings(mut self, settings: NeighborListSettings) -> Self {
        self.neighbor_list_settings = Some(settings);
        self
    }
    pub fn build(self) -> System {
        if self.simulation_box.is_none() {
            panic!("Simulation box is not set");
        }
        if self.neighbor_list_settings.is_none() {
            panic!("Neighbor list settings are not set");
        }
        System {
            atoms: Vec::new(),
            simulation_box: self.simulation_box.unwrap(),
            neighbor_list: self
                .neighbor_list_settings
                .unwrap()
                .neighbor_list(self.simulation_box.unwrap().domain()),
        }
    }
}

impl System {
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
                id: max_id + i + 1,
                position,
                velocity: [0.0, 0.0, 0.0],
                force: [0.0, 0.0, 0.0],
                mass,
                atom_type,
            });
        }
    }
    pub fn generate_neighbor_list(&mut self) {
        self.neighbor_list.generate(&self.atoms);
    }
    pub fn atom(&self, id: usize) -> &Atom {
        &self.atoms[id]
    }
    pub fn atoms(&self) -> &[Atom] {
        &self.atoms
    }
}
