use crate::box_::Box_;

pub struct Simulation {
    step: u64,
    box_: Box_,
    ids: Vec<u64>,
    types: Vec<u32>,
    // group_mask: Vec<u32>,
    positions: Vec<[f64; 3]>,
    velocities: Vec<[f64; 3]>,
    masses: Vec<f64>,
}

impl Simulation {
    pub fn num_atoms(&self) -> usize {
        self.ids.len()
    }
    pub fn box_(&self) -> &Box_ {
        &self.box_
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
    pub fn increment_position(&mut self, atom_idx: usize, increment: [f64; 3]) {
        self.positions[atom_idx][0] += increment[0];
        self.positions[atom_idx][1] += increment[1];
        self.positions[atom_idx][2] += increment[2];
    }
    pub fn increment_velocity(&mut self, atom_idx: usize, increment: [f64; 3]) {
        self.velocities[atom_idx][0] += increment[0];
        self.velocities[atom_idx][1] += increment[1];
        self.velocities[atom_idx][2] += increment[2];
    }
    pub fn set_velocity(&mut self, atom_idx: usize, new_vel: [f64; 3]) {
        self.velocities[atom_idx] = new_vel;
    }
}
