use crate::{
    atom_type::AtomType,
    output::{Operatable, Operation, Value},
    traits::Named,
    AtomicPotentialTrait, Simulation,
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
impl<T, A> ComputeTrait<T, A> for Compute
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn compute(&self, sim: &Simulation<T, A>) -> Value {
        match self {
            Compute::AvgVsq => Value::Float(avg_vsq::compute(sim)),
            Compute::KineticE => Value::Float(kinetic_energy::compute(sim)),
            Compute::PotentialE => Value::Float(potential_energy::compute(sim)),
            Compute::Temperature => Value::Float(temperature::compute(sim)),
            Compute::TotalE => Value::Float(total_energy::compute(sim)),
        }
    }
}
impl Named for Compute {
    fn name(&self) -> &str {
        match self {
            Compute::AvgVsq => "AvgVsq",
            Compute::KineticE => "KineticE",
            Compute::PotentialE => "PotentialE",
            Compute::Temperature => "Temperature",
            Compute::TotalE => "TotalE",
        }
    }
}
impl Operatable for Compute {
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

pub trait ComputeTrait<T, A>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn compute(&self, sim: &Simulation<T, A>) -> Value;
}
