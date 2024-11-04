mod rect;

pub use rect::Rect;

/// A region of the simulation space
pub trait Region {
    /// Whether the given coordinate is within the region, including
    /// the lower bounds but excluding the upper bounds.
    fn contains(&self, coord: &[f64; 3]) -> bool;
    /// Produces a random coordinate within the region, including
    /// the lower bounds but excluding the upper bounds.
    fn get_random_coord(&self) -> [f64; 3];
    fn bounding_box(&self) -> Rect;
    fn volume(&self) -> f64;
    fn surface_area(&self) -> f64;
}
