use super::*;

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    kinetic_energy::compute(sim) / sim.atoms.num_atoms_global() as f64
}
