mod bcc;
mod cubic;
mod fcc;
mod hcp;

pub use cubic::Cubic;

use crate::region::Region;

pub trait Lattice {
    fn coords_within_region<R: Region>(&self, region: &R, origin: &[f64; 3]) -> Vec<[f64; 3]>;
    fn cell_lengths(&self) -> [f64; 3];
}
