use crate::{neighbor, sort};

pub struct Atoms {
    pub ids: Vec<usize>,
    pub types: Vec<u32>,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub masses: Vec<f64>,
}
impl Atoms {
    pub fn new() -> Self {
        Atoms {
            ids: Vec::new(),
            types: Vec::new(),
            positions: Vec::new(),
            velocities: Vec::new(),
            masses: Vec::new(),
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
            .map(|coord| bins.bin_idx_from_3d_idx(&bins.coord_to_3d_idx(coord)))
            .collect();
        let sort_indices = sort::get_sort_indices(&bin_indices);

        sort::sort_atoms(&sort_indices, &mut self.ids, 0usize);
        sort::sort_atoms(&sort_indices, &mut self.types, 0u32);
        sort::sort_atoms(&sort_indices, &mut self.positions, [0.0f64, 0.0, 0.0]);
        sort::sort_atoms(&sort_indices, &mut self.velocities, [0.0f64, 0.0, 0.0]);
        sort::sort_atoms(&sort_indices, &mut self.masses, 0.0f64);

        return self
            .positions
            .iter()
            .map(|coord| bins.bin_idx_from_3d_idx(&bins.coord_to_3d_idx(coord)))
            .collect();
    }
}
