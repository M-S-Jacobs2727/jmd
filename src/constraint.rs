mod force;
mod integrator;
mod periodic_boundary;

pub use force::{LJCut, LJCutCoeff};
pub use integrator::VelocityVerlet;
pub use periodic_boundary::PeriodicBoundary;

use crate::System;

pub trait Constraint {
    fn pre_forward_communication(&mut self, _system: &mut System) {}
    fn forward_communication(&mut self, _system: &mut System) {}
    fn post_forward_communication(&mut self, _system: &mut System) {}
    fn pre_compute_force(&mut self, _system: &mut System) {}
    fn compute_force(&mut self, _system: &mut System) {}
    fn pre_reverse_communication(&mut self, _system: &mut System) {}
    fn reverse_communication(&mut self, _system: &mut System) {}
    fn post_reverse_communication(&mut self, _system: &mut System) {}
}
