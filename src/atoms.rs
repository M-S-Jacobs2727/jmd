use rand_distr::Distribution;

use crate::{neighbor, region::Region, utils, Error};

/// Atom properties during simulation, not including forces
#[derive(Debug)]
pub struct Atoms {
    pub ids: Vec<usize>,
    pub types: Vec<u32>,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub masses: Vec<f64>,
    pub nlocal: usize,
}
impl Atoms {
    pub fn new() -> Self {
        Atoms {
            ids: Vec::new(),
            types: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
            nlocal: 0,
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
    pub fn types(&self) -> &Vec<u32> {
        &self.types
    }
    pub fn positions(&self) -> &Vec<[f64; 3]> {
        &self.positions
    }
    pub fn velocities(&self) -> &Vec<[f64; 3]> {
        &self.velocities
    }
    pub fn masses(&self) -> &Vec<f64> {
        &self.masses
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
    pub fn sort_atoms_by_bin(&mut self, bins: &neighbor::Grid) -> Vec<usize> {
        let bin_indices = self
            .positions
            .iter()
            .map(|coord| bins.coord_to_index(coord).idx())
            .collect();
        let sort_indices = utils::get_sort_indices(&bin_indices);

        utils::sort_atoms(&sort_indices, &mut self.ids, 0usize);
        utils::sort_atoms(&sort_indices, &mut self.types, 0u32);
        utils::sort_atoms(&sort_indices, &mut self.positions, [0.0f64, 0.0, 0.0]);
        utils::sort_atoms(&sort_indices, &mut self.velocities, [0.0f64, 0.0, 0.0]);
        utils::sort_atoms(&sort_indices, &mut self.masses, 0.0f64);

        return self
            .positions
            .iter()
            .map(|coord| bins.coord_to_index(coord).idx())
            .collect();
    }
    pub fn add_random_atoms(
        &mut self,
        region: &impl Region,
        num_atoms: usize,
        atom_type: u32,
        mass: f64,
    ) {
        let atom_id = match self.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        self.ids.extend(atom_id..atom_id + num_atoms);
        self.types.reserve(num_atoms);
        self.positions.reserve(num_atoms);
        self.velocities.reserve(num_atoms);
        self.masses.reserve(num_atoms);
        self.nlocal += num_atoms;

        for _i in 0..num_atoms {
            self.types.push(atom_type);
            self.masses.push(mass);
            self.velocities.push([0.0, 0.0, 0.0]);
            self.positions.push(region.get_random_coord())
        }
    }
    pub fn add_atoms(&mut self, atom_type: u32, mass: f64, coords: Vec<[f64; 3]>) {
        let num_atoms = coords.len();
        let atom_id = match self.ids().iter().max() {
            Some(j) => j + 1,
            None => 0,
        };
        self.ids.extend(atom_id..atom_id + num_atoms);
        self.types.reserve(num_atoms);
        self.positions.reserve(num_atoms);
        self.velocities.reserve(num_atoms);
        self.masses.reserve(num_atoms);
        self.nlocal += num_atoms;

        for i in 0..num_atoms {
            self.types.push(atom_type);
            self.masses.push(mass);
            self.velocities.push([0.0, 0.0, 0.0]);
            self.positions.push(coords[i])
        }
    }
    pub fn set_temperature(&mut self, temperature: f64) -> Result<(), Error> {
        let mut rng = rand::thread_rng();
        let dist =
            rand_distr::Normal::new(0.0, temperature.sqrt()).map_err(|_e| Error::OtherError)?;
        let sqrt_ke: Vec<f64> = dist.sample_iter(&mut rng).take(self.nlocal * 3).collect();
        for i in 0..self.nlocal {
            self.velocities[i] = [
                sqrt_ke[3 * i + 0] / self.masses[i].sqrt(),
                sqrt_ke[3 * i + 1] / self.masses[i].sqrt(),
                sqrt_ke[3 * i + 2] / self.masses[i].sqrt(),
            ];
        }
        Ok(())
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
        self.masses = filter_by_idx(&atom_idxs, &self.masses);
        self.positions = filter_by_idx(&atom_idxs, &self.positions);
        self.velocities = filter_by_idx(&atom_idxs, &self.velocities);
    }
}
