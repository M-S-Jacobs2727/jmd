use crate::{atom_type::AtomType, atomic::AtomicPotentialTrait, simulation::Simulation};

mod verlet;

pub use verlet::Verlet;

/// Simulation integrator
pub trait Integrator<T, A>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn pre_forward_comm(_simulation: &mut Simulation<T, A>) {}
    fn post_forward_comm(_simulation: &mut Simulation<T, A>) {}
    fn pre_reverse_comm(_simulation: &mut Simulation<T, A>) {}
    fn post_reverse_comm(_simulation: &mut Simulation<T, A>) {}
}
