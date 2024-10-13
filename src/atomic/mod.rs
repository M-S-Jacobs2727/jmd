pub mod ljcut;
pub mod none;

pub use ljcut::{LJCut, LJCutCoeff};

use crate::Atoms;
use enum_dispatch::enum_dispatch;
use none::None_;

#[enum_dispatch]
pub enum AP {
    LJCut,
    None_,
}
// TODO: add num_types to new method
#[enum_dispatch(AP)]
/// Trait for pairwise atomic potentials
pub trait AtomicPotential {
    /// Get the maximum distance for effective interaction
    fn cutoff_distance(&self) -> f64;

    /// Compute the pairwise force given a configuration of atoms
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]>;
}
