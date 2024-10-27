mod verlet;
pub use verlet::Verlet;

use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

/// Simulation integrator
pub trait Integrator<T, A>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn run(&self, simulation: &mut Simulation<T, A>, num_steps: usize);
}
