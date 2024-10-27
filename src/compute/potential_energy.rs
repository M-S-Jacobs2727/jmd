use crate::Simulation;

pub(super) fn compute(sim: &Simulation) -> f64 {
    sim.atomic_potential().compute_potential_energy(sim)
}
