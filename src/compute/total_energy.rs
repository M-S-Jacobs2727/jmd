use crate::Simulation;

use super::kinetic_energy;

pub(super) fn compute(sim: &Simulation) -> f64 {
    sim.atomic_potential().compute_potential_energy(sim) + kinetic_energy::compute(sim)
}
