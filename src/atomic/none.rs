use ndarray::Array2;

use super::AtomicPotentialTrait;
use crate::Atoms;

pub struct None_ {}
impl None_ {
    pub fn new() -> Self {
        Self {}
    }
}
impl AtomicPotentialTrait for None_ {
    fn cutoff_distance(&self) -> f64 {
        0.0
    }
    fn compute_forces(&self, atoms: &Atoms) -> Array2<f64> {
        let natoms = atoms.num_atoms();
        Array2::zeros([natoms, 3])
    }
    fn set_num_types(&mut self, _num_types: usize) -> Result<(), crate::Error> {
        Ok(())
    }
    fn num_types(&self) -> usize {
        0
    }
    fn compute_energy(&self, _atoms: &Atoms) -> f64 {
        0.0
    }
}
