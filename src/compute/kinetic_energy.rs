use crate::{
    output::{Cumulation, OutputFormat},
    Simulation,
};

use super::{Compute, ComputeValue};

pub(super) fn compute_local_ke(sim: &Simulation) -> f64 {
    0.5 * sim
        .atoms
        .velocities
        .iter()
        .zip(sim.atoms.masses.iter())
        .take(sim.nlocal())
        .map(|(v, m)| m * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
        .sum::<f64>()
}

pub struct KineticEnergy {}

impl Compute for KineticEnergy {
    fn output_format(&self) -> OutputFormat {
        OutputFormat::new("KE", Cumulation::Sum)
    }
    fn compute(&self, sim: &Simulation) -> ComputeValue {
        ComputeValue::Float(compute_local_ke(sim))
    }
}
