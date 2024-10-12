pub mod rect;
pub use rect::Rect;
// TODO: change add_random_atoms to get_random_coord
// TODO: move add_random_atoms to Atoms struct
use crate::Atoms;

pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn add_random_atoms(&self, sim: &mut Atoms, num_atoms: usize, atom_type: u32, mass: f64);
}
