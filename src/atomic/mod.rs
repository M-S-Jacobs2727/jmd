// TODO: add num_types to new method
pub mod ljcut;
pub mod none;

pub use ljcut::{LJCut, LJCutCoeff};

use enum_dispatch::enum_dispatch;

use crate::Atoms;
use none::None_;

#[enum_dispatch]
pub enum AtomicPotential {
    LJCut,
    None_,
}
#[enum_dispatch(AtomicPotential)]
/// Trait for pairwise atomic potentials
pub trait AtomicPotentialTrait {
    /// Get the maximum distance for effective interaction
    fn cutoff_distance(&self) -> f64;

    /// Compute the pairwise force given a configuration of atoms
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]>;
}
