pub mod ljcut;
use crate::simulation::Simulation;

pub trait AtomicPotential {
    fn compute_forces(&self, sim: &Simulation) -> Vec<[f64; 3]>;
    fn type_index(&self, typei: u32, typej: u32) -> usize;
}
