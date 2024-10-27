use crate::{
    output::{Operation, Value},
    Simulation,
};

mod avg_vsq;
mod kinetic_energy;
mod potential_energy;
mod temperature;
mod total_energy;

#[derive(Debug, Clone, PartialEq)]
pub enum Compute {
    AvgVsq,
    KineticE,
    PotentialE,
    Temperature,
    TotalE,
}
impl ComputeTrait for Compute {
    fn compute(&self, sim: &Simulation) -> Value {
        match self {
            Compute::AvgVsq => Value::Float(avg_vsq::compute(sim)),
            Compute::KineticE => Value::Float(kinetic_energy::compute(sim)),
            Compute::PotentialE => Value::Float(potential_energy::compute(sim)),
            Compute::Temperature => Value::Float(temperature::compute(sim)),
            Compute::TotalE => Value::Float(total_energy::compute(sim)),
        }
    }
    fn name(&self) -> &str {
        match self {
            Compute::AvgVsq => "AvgVsq",
            Compute::KineticE => "KineticE",
            Compute::PotentialE => "PotentialE",
            Compute::Temperature => "Temperature",
            Compute::TotalE => "TotalE",
        }
    }
    fn op(&self) -> Operation {
        match self {
            Compute::AvgVsq
            | Compute::KineticE
            | Compute::PotentialE
            | Compute::Temperature
            | Compute::TotalE => Operation::Sum,
        }
    }
}

pub trait ComputeTrait {
    fn compute(&self, sim: &Simulation) -> Value;
    fn name(&self) -> &str;
    fn op(&self) -> Operation;
}
