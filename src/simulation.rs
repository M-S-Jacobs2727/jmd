use crate::{box_::Box_, neighbor, sort};

pub struct Simulation {
    step: u64,
    box_: Box_,
    ghost_atom_ref_indices: Vec<usize>,
    ids: Vec<u64>,
    types: Vec<u32>,
    // group_mask: Vec<u32>,
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
    masses: Vec<f64>,
    bin_indices: Vec<usize>,
}
impl Simulation {
    pub fn num_atoms(&self) -> usize {
        self.ids.len()
    }
    pub fn num_ghost_atoms(&self) -> usize {
        self.ghost_atom_ref_indices.len()
    }
    pub fn ghost_atom_ref_indices(&self) -> &Vec<usize> {
        &self.ghost_atom_ref_indices
    }
    pub fn box_(&self) -> &Box_ {
        &self.box_
    }
    pub fn id(&self, i: usize) -> u64 {
        self.ids[i]
    }
    pub fn id_to_idx(&self, id: u64) -> Option<usize> {
        self.ids.iter().position(|&x| x == id)
    }
    pub fn type_(&self, i: usize) -> u32 {
        self.types[i]
    }
    pub fn position(&self, i: usize) -> &[f64; 3] {
        &self.positions[i]
    }
    pub fn velocity(&self, i: usize) -> &[f64; 3] {
        &self.velocities[i]
    }
    pub fn mass(&self, i: usize) -> f64 {
        self.masses[i]
    }
    pub fn increment_step(&mut self) {
        self.step += 1;
    }
    pub fn reset_timestep(&mut self, new_timestep: u64) {
        self.step = new_timestep;
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

    pub fn sort_atoms_by_bin(&mut self, bins: &neighbor::Grid) {
        let bin_indices = self
            .positions
            .iter()
            .map(|coord| bins.coord_to_bin_idx(coord))
            .collect();
        let sort_indices = sort::get_sort_indices(&bin_indices);

        sort::sort_atoms(&sort_indices, &mut self.ids, 0u64);
        sort::sort_atoms(&sort_indices, &mut self.types, 0u32);
        sort::sort_atoms(&sort_indices, &mut self.positions, [0.0f64, 0.0, 0.0]);
        sort::sort_atoms(&sort_indices, &mut self.velocities, [0.0f64, 0.0, 0.0]);
        sort::sort_atoms(&sort_indices, &mut self.masses, 0.0f64);

        self.bin_indices = self
            .positions
            .iter()
            .map(|coord| bins.coord_to_bin_idx(coord))
            .collect();
    }
}
