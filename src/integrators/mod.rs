use crate::{parallel::Simulation, AtomicPotential};

pub mod verlet;
pub use verlet::Verlet;

/// Simulation integrator
pub trait Integrator<P: AtomicPotential> {
    fn new(simulation: Simulation<P>) -> Self;
    fn run(&mut self, num_steps: usize);
}
