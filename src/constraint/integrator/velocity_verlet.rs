use crate::constraint::Constraint;
use crate::{Atom, Domain};

pub struct VelocityVerlet {
    pub dt: f64,
}

impl VelocityVerlet {
    pub fn new(dt: f64) -> Self {
        Self { dt }
    }
    pub fn compute_half_step_velocity(&mut self, atoms: &mut Vec<Atom>) {
        for atom in atoms.iter_mut() {
            atom.velocity[0] += atom.force[0] * 0.5 * self.dt;
            atom.velocity[1] += atom.force[1] * 0.5 * self.dt;
            atom.velocity[2] += atom.force[2] * 0.5 * self.dt;
        }
    }
    pub fn compute_position(&mut self, atoms: &mut Vec<Atom>) {
        for atom in atoms.iter_mut() {
            atom.position[0] += atom.velocity[0] * self.dt;
            atom.position[1] += atom.velocity[1] * self.dt;
            atom.position[2] += atom.velocity[2] * self.dt;
        }
    }
}

impl Constraint for VelocityVerlet {
    fn pre_forward_communication(&mut self, atoms: &mut Vec<Atom>, _domain: &mut Domain) {
        self.compute_half_step_velocity(atoms);
        self.compute_position(atoms);
    }
    fn pre_reverse_communication(&mut self, atoms: &mut Vec<Atom>, _domain: &mut Domain) {
        self.compute_half_step_velocity(atoms);
    }
}
