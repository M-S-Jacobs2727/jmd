use super::*;

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    potential_energy::compute(sim) + kinetic_energy::compute(sim)
}
