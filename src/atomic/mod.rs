pub mod ljcut;
pub mod none;

use crate::Atoms;

pub trait AtomicPotential {
    fn new() -> Self;
    fn cutoff_distance(&self) -> f64;
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]>;
}
