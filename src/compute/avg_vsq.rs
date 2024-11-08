use super::*;

pub(super) fn vsq<T, A>(simulation: &Simulation<T, A>) -> Vec<f64>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    simulation
        .atoms
        .velocities
        .iter()
        .take(simulation.nlocal())
        .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        .collect()
}
pub(super) fn compute<T, A>(simulation: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    vsq(simulation).iter().sum::<f64>() / simulation.atoms.num_atoms_global() as f64
}
