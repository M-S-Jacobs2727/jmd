use crate::{
    output::{Cumulation, OutputFormat},
    Simulation,
};

use super::{kinetic_energy::compute_local_ke, Compute, ComputeValue};

pub struct TotalEnergy {}

impl Compute for TotalEnergy {
    fn output_format(&self) -> OutputFormat {
        OutputFormat::new("PE", Cumulation::Sum)
    }
    fn compute(&self, sim: &Simulation) -> ComputeValue {
        ComputeValue::Float(
            sim.atomic_potential().compute_potential_energy(sim) + compute_local_ke(sim),
        )
    }
}
