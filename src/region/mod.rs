pub mod rect;
pub use rect::Rect;

pub use crate::Simulation;

pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn add_random_atoms(&self, sim: &mut Simulation, num_atoms: usize, atom_type: u32, mass: f64);
}
