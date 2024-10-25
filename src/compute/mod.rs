use std::{fmt::Display, ops::AddAssign};

use crate::{output::OutputFormat, Simulation};

mod kinetic_energy;
mod potential_energy;
mod temperature;
mod total_energy;

use kinetic_energy::compute_local_ke;
pub use kinetic_energy::KineticEnergy;
pub use potential_energy::PotentialEnergy;
pub use temperature::Temperature;
pub use total_energy::TotalEnergy;

pub enum ComputeValue {
    Float(f64),
    Int(i32),
    Usize(usize),
    Bool(bool),
}
impl AddAssign for ComputeValue {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (ComputeValue::Float(i), ComputeValue::Float(j)) => *i += j,
            (ComputeValue::Int(i), ComputeValue::Int(j)) => *i += j,
            (ComputeValue::Usize(i), ComputeValue::Usize(j)) => *i += j,
            _ => panic!("Mismatched types"),
        }
    }
}
impl Display for ComputeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComputeValue::Float(v) => v.fmt(f),
            ComputeValue::Int(v) => v.fmt(f),
            ComputeValue::Usize(v) => v.fmt(f),
            ComputeValue::Bool(v) => v.fmt(f),
        }
    }
}

pub trait Compute {
    fn compute(&self, sim: &Simulation) -> ComputeValue;
    fn output_format(&self) -> OutputFormat;
}
