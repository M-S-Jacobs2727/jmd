use std::fmt::Debug;

use rand_distr::Distribution;

use crate::{atom_type::AtomType, neighbor, region::Region, utils};

/// Atom properties during simulation, not including forces
#[derive(Debug)]
pub struct Atoms<T: AtomType> {
    pub ids: Vec<usize>,
    pub types: Vec<usize>,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub nlocal: usize,
    atom_types: Vec<T>,
}
impl<T: AtomType> Atoms<T> {
    pub fn new() -> Self {
        Atoms {
            ids: Vec::new(),
            types: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            nlocal: 0,
            atom_types: Vec::new(),
        }
    }
    pub fn num_atoms(&self) -> usize {
        self.ids.len()
    }
    pub fn ids(&self) -> &Vec<usize> {
        &self.ids
    }
    pub fn id_to_idx(&self, id: usize) -> Option<usize> {
        self.ids.iter().position(|x| *x == id)
    }
    pub fn types(&self) -> &Vec<usize> {
        &self.types
    }
    pub fn positions(&self) -> &Vec<[f64; 3]> {
        &self.positions
    }
    pub fn velocities(&self) -> &Vec<[f64; 3]> {
        &self.velocities
    }
    pub fn mass(&self, idx: usize) -> f64 {
        self.atom_types[self.types[idx]].mass()
    }
    pub fn atom_types(&self) -> &Vec<T> {
        &self.atom_types
    }
    pub fn num_types(&self) -> usize {
        self.atom_types.len()
    }
    pub fn increment_position(&mut self, i: usize, increment: [f64; 3]) {
        self.positions[i][0] += increment[0];
        self.positions[i][1] += increment[1];
        self.positions[i][2] += increment[2];
    }
    pub fn increment_velocity(&mut self, i: usize, increment: [f64; 3]) {
        self.velocities[i][0] += increment[0];
        self.velocities[i][1] += increment[1];
        self.velocities[i][2] += increment[2];
    }
    pub fn set_velocity(&mut self, i: usize, new_vel: [f64; 3]) {
        self.velocities[i] = new_vel;
    }
    pub fn sort_atoms_by_bin(&mut self, nl: &neighbor::NeighborList) -> Vec<usize> {
        let bin_indices = self
            .positions
            .iter()
            .map(|coord| nl.coord_to_index(coord).idx())
            .collect();
        let sort_indices = utils::get_sort_indices(&bin_indices);

        utils::sort_atoms(&sort_indices, &mut self.ids);
        utils::sort_atoms(&sort_indices, &mut self.types);
        utils::sort_atoms(&sort_indices, &mut self.positions);
        utils::sort_atoms(&sort_indices, &mut self.velocities);

        return self
            .positions
            .iter()
            .map(|coord| nl.coord_to_index(coord).idx())
            .collect();
    }
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
    pub fn remove_idxs(&mut self, atom_idxs: Vec<usize>) {
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

    pub(crate) fn set_atom_types(&mut self, atom_types: Vec<T>) {
        self.atom_types = atom_types
    }
}
