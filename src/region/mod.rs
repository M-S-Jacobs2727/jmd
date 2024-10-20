mod rect;
pub use rect::Rect;

/// A region of the simulation space
pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn get_random_coord(&self) -> [f64; 3];
}
