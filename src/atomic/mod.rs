mod ljcut;
mod none;

pub use ljcut::LJCut;
use ndarray::Array2;
pub use none::None_;

use enum_dispatch::enum_dispatch;

use crate::{Atoms, Error};

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
    fn compute_forces(&self, atoms: &Atoms) -> Array2<f64>;

    /// Increase or decrease the number of atom types
    fn set_num_types(&mut self, num_types: usize) -> Result<(), Error>;

    fn num_types(&self) -> usize;
    fn compute_energy(&self, atoms: &Atoms) -> f64;

    fn type_idx(&self, typei: u32, typej: u32) -> usize {
        self.num_types() * typei as usize + typej as usize
    }
}
