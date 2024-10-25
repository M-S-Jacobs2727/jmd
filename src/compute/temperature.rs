use crate::{
    output::{Cumulation, OutputFormat},
    Simulation,
};

use super::{kinetic_energy::compute_local_ke, Compute, ComputeValue};

pub struct Temperature {}

impl Compute for Temperature {
    fn output_format(&self) -> OutputFormat {
        OutputFormat::new("KE", Cumulation::Sum)
    }
    fn compute(&self, sim: &Simulation) -> ComputeValue {
        ComputeValue::Float(compute_local_ke(sim) / sim.atoms.num_atoms() as f64)
    }
}
