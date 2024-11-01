mod ljcut;
mod none;

pub use ljcut::{LJCut, LJCutCoeff};
pub use none::None_;

use crate::{atom_type::AtomType, Atoms, NeighborList};

/// Trait for pairwise atomic potentials
pub trait AtomicPotentialTrait<T: AtomType> {
    type CoeffType;

    /// Get the maximum distance for effective interaction
    fn cutoff_distance(&self) -> f64;

    /// Compute the pairwise force given a configuration of atoms
    fn compute_forces(&self, atoms: &Atoms<T>, neighbor_list: &NeighborList) -> Vec<[f64; 3]>;

    /// Increase or decrease the number of atom types
    fn set_num_types(&mut self, num_types: usize);

    fn num_types(&self) -> usize;
    fn compute_potential_energy(&self, atoms: &Atoms<T>, neighbor_list: &NeighborList) -> f64;

    fn type_idx(&self, typei: usize, typej: usize) -> usize {
        self.num_types() * typei + typej
    }

    fn all_set(&self) -> bool;
}
