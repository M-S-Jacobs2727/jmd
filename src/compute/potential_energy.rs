use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

pub(super) fn compute<T, A>(sim: &Simulation<T, A>) -> f64
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    sim.atomic_potential()
        .compute_potential_energy(&sim.atoms, &sim.neighbor_list())
}
