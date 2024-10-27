use crate::Simulation;

use super::kinetic_energy;

pub(super) fn compute(sim: &Simulation) -> f64 {
    kinetic_energy::compute(sim) / sim.atoms.num_atoms() as f64
}
