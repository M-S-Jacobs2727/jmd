use super::AtomicPotential;
use crate::Simulation;

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
    fn compute_forces(&self, sim: &Simulation) -> Vec<[f64; 3]> {
        let natoms = sim.atoms.num_atoms();
        let mut forces = Vec::new();
        forces.resize(natoms, [0.0, 0.0, 0.0]);
        forces
    }
    fn set_num_types(&mut self, _num_types: usize) -> Result<(), crate::Error> {
        Ok(())
    }
    fn num_types(&self) -> usize {
        0
    }
    fn compute_potential_energy(&self, _sim: &Simulation) -> f64 {
        0.0
    }
}
