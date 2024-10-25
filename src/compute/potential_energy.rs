use crate::{
    output::{Cumulation, OutputFormat},
    Simulation,
};

use super::{Compute, ComputeValue};

pub struct PotentialEnergy {}

impl Compute for PotentialEnergy {
    fn output_format(&self) -> OutputFormat {
        OutputFormat::new("PE", Cumulation::Sum)
    }
    fn compute(&self, sim: &Simulation) -> ComputeValue {
        ComputeValue::Float(sim.atomic_potential().compute_potential_energy(sim))
    }
}
