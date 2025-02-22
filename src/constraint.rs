mod force;
mod integrator;
mod periodic_boundary;

pub use force::{LJCut, LJCutCoeff};
pub use integrator::VelocityVerlet;
pub use periodic_boundary::PeriodicBoundary;

use crate::{Atom, Domain};

pub trait Constraint {
    fn pre_forward_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn forward_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn post_forward_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn pre_compute_force(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn compute_force(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn pre_reverse_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn reverse_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
    fn post_reverse_communication(&mut self, _atoms: &mut Vec<Atom>, _domain: &mut Domain) {}
}
