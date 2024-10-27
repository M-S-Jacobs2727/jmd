use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    0.5 * sim
        .atoms
        .velocities
        .iter()
        .enumerate()
        .take(sim.atoms.nlocal)
        .map(|(i, v)| sim.atoms.mass(i) * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
        .sum::<f64>()
}
