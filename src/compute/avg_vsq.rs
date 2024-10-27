use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

pub(super) fn compute<T, A>(simulation: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    simulation
        .atoms
        .velocities
        .iter()
        .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        .sum::<f64>()
        / simulation.atoms.num_atoms() as f64
}
