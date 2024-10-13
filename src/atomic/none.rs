use super::AtomicPotential;
use crate::Atoms;

pub struct None_ {}
impl None_ {
    pub fn new() -> Self {
        Self {}
    }
}
impl AtomicPotential for None_ {
    fn cutoff_distance(&self) -> f64 {
        0.0
    }
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]> {
        let natoms = atoms.num_atoms();
        let mut forces = Vec::new();
        forces.resize(natoms, [0.0, 0.0, 0.0]);
        forces
    }
}
