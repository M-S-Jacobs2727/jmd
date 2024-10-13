use crate::{parallel::Simulation, AtomicPotential};

pub mod verlet;
pub use verlet::Verlet;

/// Simulation integrator
pub trait Integrator {
    fn new() -> Self;
    fn run<P: AtomicPotential>(&self, simulation: &mut Simulation<P>, num_steps: usize);
}
