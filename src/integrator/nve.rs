use crate::prelude::*;
use crate::Atoms;

#[derive(Clone, Copy, Debug)]
pub struct NVE {
    pub dt: f64,
}

impl NVE {
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }
    pub fn compute_half_step_velocity(&self, atoms: &mut Atoms) {
        let vel = &mut atoms.velocities;
        let f = &atoms.forces;
        for i in 0..vel.len() {
            vel[i][0] += f[i][0] * 0.5 * self.dt;
            vel[i][1] += f[i][1] * 0.5 * self.dt;
            vel[i][2] += f[i][2] * 0.5 * self.dt;
        }
    }
    pub fn compute_position(&self, atoms: &mut Atoms) {
        let pos = &mut atoms.positions;
        let vel = &atoms.velocities;
        for i in 0..vel.len() {
            pos[i][0] += vel[i][0] * self.dt;
            pos[i][1] += vel[i][1] * self.dt;
            pos[i][2] += vel[i][2] * self.dt;
        }
    }
}

impl Integrator for NVE {
    fn initial_integrate(&self, atoms: &mut Atoms, _: &mut Domain) {
        self.compute_half_step_velocity(atoms);
        self.compute_position(atoms);
    }
    fn final_integrate(&self, atoms: &mut Atoms, _: &mut Domain) {
        self.compute_half_step_velocity(atoms);
    }
}
