use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

use super::kinetic_energy;
use super::potential_energy;

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    potential_energy::compute(sim) + kinetic_energy::compute(sim)
}
