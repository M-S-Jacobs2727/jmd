mod verlet;
pub use verlet::Verlet;

use crate::parallel::Simulation;

/// Simulation integrator
pub trait Integrator {
    fn new() -> Self;
    fn run(&self, simulation: &mut Simulation, num_steps: usize);
}
