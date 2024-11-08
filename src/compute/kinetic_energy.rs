use super::*;

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    0.5 * vsq(sim)
        .iter()
        .enumerate()
        .map(|(i, vsq)| sim.atoms.mass(i) * vsq)
        .sum::<f64>()
}
