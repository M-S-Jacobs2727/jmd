pub mod ljcut;
use crate::Atoms;

pub trait AtomicPotential {
    fn compute_forces(&self, sim: &Atoms) -> Vec<[f64; 3]>;
    fn type_index(&self, typei: u32, typej: u32) -> usize;
}
