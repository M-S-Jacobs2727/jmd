use crate::parallel::Simulation;

mod verlet;
pub use verlet::Verlet;

/// Simulation integrator
pub trait Integrator {
    fn new() -> Self;
    fn run(&self, simulation: &mut Simulation, num_steps: usize);
}
