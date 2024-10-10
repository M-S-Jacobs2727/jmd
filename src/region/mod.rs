pub mod rect;
pub use rect::Rect;

use crate::Atoms;

pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn add_random_atoms(&self, sim: &mut Atoms, num_atoms: usize, atom_type: u32, mass: f64);
}
