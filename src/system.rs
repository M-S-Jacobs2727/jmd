use crate::{Atom, Domain, NeighborList, NeighborListSettings};

pub struct SystemBuilder {
    domain: Option<Domain>,
    neighbor_list_settings: Option<NeighborListSettings>,
}

pub struct System {
    pub(crate) atoms: Vec<Atom>,
    pub(crate) domain: Domain,
    pub(crate) neighbor_list: NeighborList,
}

impl SystemBuilder {
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
    pub fn build(self) -> System {
        if self.domain.is_none() {
            panic!("Domain is not set");
        }
        if self.neighbor_list_settings.is_none() {
            panic!("Neighbor list settings are not set");
        }
        System {
            atoms: Vec::new(),
            domain: self.domain.unwrap(),
            neighbor_list: NeighborList::new(self.neighbor_list_settings.unwrap()),
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
                id: max_id + i,
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
}
