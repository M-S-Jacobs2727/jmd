pub mod rect;
use enum_dispatch::enum_dispatch;
pub use rect::Rect;
// TODO: change add_random_atoms to get_random_coord
// TODO: move add_random_atoms to Atoms struct
use crate::Atoms;

#[enum_dispatch]
pub enum Regions {
    Rect,
}
#[enum_dispatch(Regions)]
/// A region of the simulation space
pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn add_random_atoms(&self, sim: &mut Atoms, num_atoms: usize, atom_type: u32, mass: f64);
}
