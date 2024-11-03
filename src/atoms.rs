use std::fmt::Debug;

use rand_distr::Distribution;

use crate::{atom_type::AtomType, region::Region};

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
    /// Set the velocity of the atom at the given index
    pub(crate) fn set_velocity(&mut self, i: usize, new_vel: [f64; 3]) {
        self.velocities[i] = new_vel;
    }
    /// Add a given number of atoms of the given type with the given region
    /// TODO: refactor to work correctly with more than one process
    pub fn add_random_atoms(&mut self, region: &impl Region, num_atoms: usize, atom_type: usize) {
        let atom_id = match self.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        self.ids.extend(atom_id..atom_id + num_atoms);
        self.types.reserve(num_atoms);
        self.positions.reserve(num_atoms);
        self.velocities.reserve(num_atoms);
        self.nlocal += num_atoms;

        for _i in 0..num_atoms {
            self.types.push(atom_type);
            self.velocities.push([0.0, 0.0, 0.0]);
            self.positions.push(region.get_random_coord())
        }
    }
    /// Add atoms of the given type at the given coordinates
    pub fn add_atoms(&mut self, atom_type: usize, coords: Vec<[f64; 3]>) {
        let num_atoms = coords.len();
        let atom_id = match self.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        self.ids.extend(atom_id..atom_id + num_atoms);
        self.types.reserve(num_atoms);
        self.positions.reserve(num_atoms);
        self.velocities.reserve(num_atoms);
        self.nlocal += num_atoms;

        for i in 0..num_atoms {
            self.types.push(atom_type);
            self.velocities.push([0.0, 0.0, 0.0]);
            self.positions.push(coords[i])
        }
    }
    /// Set the temperature
    /// TODO: move to simulation, make work across multiple processes
    pub fn set_temperature(&mut self, temperature: f64) {
        let mut rng = rand::thread_rng();
        let dist = rand_distr::Normal::new(0.0, temperature.sqrt()).expect("Invalid temperature");
        let sqrt_ke: Vec<f64> = dist.sample_iter(&mut rng).take(self.nlocal * 3).collect();
        for i in 0..self.nlocal {
            self.velocities[i] = [
                sqrt_ke[3 * i + 0] / self.atom_types[self.types[i]].mass().sqrt(),
                sqrt_ke[3 * i + 1] / self.atom_types[self.types[i]].mass().sqrt(),
                sqrt_ke[3 * i + 2] / self.atom_types[self.types[i]].mass().sqrt(),
            ];
        }
    }
    /// Remove atoms at the given indices
    /// TODO: change to IDs instead, add convenience functions for regions
    pub(crate) fn remove_idxs(&mut self, atom_idxs: Vec<usize>) {
        let num_local = atom_idxs.iter().filter(|&i| *i < self.nlocal).count();
        self.nlocal -= num_local;
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

        self.ids = filter_by_idx(&atom_idxs, &self.ids);
        self.types = filter_by_idx(&atom_idxs, &self.types);
        self.positions = filter_by_idx(&atom_idxs, &self.positions);
        self.velocities = filter_by_idx(&atom_idxs, &self.velocities);
    }
    /// Set the list of atom types
    /// TODO: Check if this needs to include side effects
    pub(crate) fn set_atom_types(&mut self, atom_types: Vec<T>) {
        self.atom_types = atom_types
    }
}
