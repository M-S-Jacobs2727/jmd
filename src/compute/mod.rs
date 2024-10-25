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

pub trait Compute {
    fn compute(&self, sim: &Simulation) -> ComputeValue;
    fn output_format(&self) -> OutputFormat;
}
