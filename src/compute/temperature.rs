use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

use super::kinetic_energy;

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    kinetic_energy::compute(sim) / sim.atoms.num_atoms() as f64
}
