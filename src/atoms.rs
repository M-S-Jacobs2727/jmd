use std::fmt::Debug;

use crate::atom_type::AtomType;

pub(crate) struct Atom {
    pub(crate) id: usize,
    pub(crate) type_: usize,
    pub(crate) position: [f64; 3],
    pub(crate) velocity: [f64; 3],
}

/// Atom properties during simulation, not including forces
#[derive(Debug)]
pub struct Atoms<T: AtomType> {
    pub(crate) ids: Vec<usize>,
    pub(crate) types: Vec<usize>,
    pub(crate) positions: Vec<[f64; 3]>,
    pub(crate) velocities: Vec<[f64; 3]>,
    pub(crate) atom_types: Vec<T>,
    pub(crate) nlocal: usize,
    pub(crate) num_atoms_global: usize,
}
impl<T: AtomType> Atoms<T> {
    /// Create a new, empty set of atoms
    ///
    /// ```rust
    /// use jmd;
    /// let atoms: jmd::Atoms<jmd::atom_type::Basic> = Atoms::new();
    /// ```
    pub fn new() -> Self {
        Atoms {
            ids: Vec::new(),
            types: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            atom_types: Vec::new(),
            nlocal: 0,
            num_atoms_global: 0,
        }
    }
    /// The total number of atoms in the simulation
    pub fn num_atoms_global(&self) -> usize {
        self.num_atoms_global
    }
    /// The number of atoms owned by the current process
    pub fn num_local_atoms(&self) -> usize {
        self.nlocal
    }
    /// The number of atoms known but not owned by the current process
    pub fn num_ghost_atoms(&self) -> usize {
        self.ids.len() - self.nlocal
    }
    /// The total number of atoms known by the current process
    pub fn num_total_atoms(&self) -> usize {
        self.ids.len()
    }
    /// A reference to the atom IDs
    pub fn ids(&self) -> &Vec<usize> {
        &self.ids
    }
    /// Find the index of the given atom ID in the current process, if it exists
    pub fn id_to_idx(&self, id: usize) -> Option<usize> {
        self.ids.iter().position(|x| *x == id)
    }
    /// The type index of each atom
    pub fn types(&self) -> &Vec<usize> {
        &self.types
    }
    /// The position of each atom
    pub fn positions(&self) -> &Vec<[f64; 3]> {
        &self.positions
    }
    /// The velocity of each atom
    pub fn velocities(&self) -> &Vec<[f64; 3]> {
        &self.velocities
    }
    /// The mass of a given atom (defined by the atom type)
    pub fn mass(&self, idx: usize) -> f64 {
        self.atom_types[self.types[idx]].mass()
    }
    /// A reference to the list of properties per atom type
    pub fn atom_types(&self) -> &Vec<T> {
        &self.atom_types
    }
    /// The number of atom types
    pub fn num_types(&self) -> usize {
        self.atom_types.len()
    }
    /// Increment the position of the atom at the given index by the given increment
    pub(crate) fn increment_position(&mut self, i: usize, increment: [f64; 3]) {
        self.positions[i][0] += increment[0];
        self.positions[i][1] += increment[1];
        self.positions[i][2] += increment[2];
    }
    /// Increment the velocity of the atom at the given index by the given increment
    pub(crate) fn increment_velocity(&mut self, i: usize, increment: [f64; 3]) {
        self.velocities[i][0] += increment[0];
        self.velocities[i][1] += increment[1];
        self.velocities[i][2] += increment[2];
    }
    pub(crate) fn update_or_push(&mut self, atom: Atom) {
        let idx = self.ids.iter().position(|id| *id == atom.id);
        match idx {
            Some(i) => {
                self.ids[i] = atom.id;
                self.types[i] = atom.type_;
                self.positions[i] = atom.position;
                self.velocities[i] = atom.velocity;
            }
            None => {
                self.ids.push(atom.id);
                self.types.push(atom.type_);
                self.positions.push(atom.position);
                self.velocities.push(atom.velocity);
            }
        };
    }
}
