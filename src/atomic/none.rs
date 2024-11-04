use super::AtomicPotentialTrait;
use crate::{atom_type::AtomType, Atoms, NeighborList};

pub struct None_ {}
impl<T: AtomType> AtomicPotentialTrait<T> for None_ {
    type Coeff = ();
    fn new() -> Self {
        Self {}
    }
    fn cutoff_distance(&self) -> f64 {
        0.0
    }
    fn compute_forces(&self, atoms: &Atoms<T>, _neighbor_list: &NeighborList) -> Vec<[f64; 3]> {
        let natoms = atoms.num_total_atoms();
        let mut forces = Vec::new();
        forces.resize(natoms, [0.0, 0.0, 0.0]);
        forces
    }
    fn set_num_types(&mut self, _num_types: usize) {}
    fn num_types(&self) -> usize {
        0
    }
    fn compute_potential_energy(&self, _atoms: &Atoms<T>, _neighbor_list: &NeighborList) -> f64 {
        0.0
    }
    fn all_set(&self) -> bool {
        true
    }
    fn set_coeff(&mut self, _typei: usize, _typej: usize, _coeff: &Self::Coeff) {}
}
