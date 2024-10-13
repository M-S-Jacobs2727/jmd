pub mod ljcut;
pub mod none;

pub use ljcut::{LJCut, LJCutCoeff};
pub enum AP {
    LJ(LJCut),
    N(none::None_),
}
impl AP {
    pub fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]> {
        match self {
            AP::LJ(lj) => lj.compute_forces(atoms),
            AP::N(n) => n.compute_forces(atoms),
        }
    }
}
use crate::Atoms;
// TODO: add num_types to new method
/// Trait for pairwise atomic potentials
pub trait AtomicPotential {
    fn new() -> Self;

    /// Get the maximum distance for effective interaction
    fn cutoff_distance(&self) -> f64;

    /// Compute the pairwise force given a configuration of atoms
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]>;
}
