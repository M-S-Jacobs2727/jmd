pub mod rect;
use enum_dispatch::enum_dispatch;
pub use rect::Rect;

#[enum_dispatch]
pub enum Regions {
    Rect,
}
#[enum_dispatch(Regions)]
/// A region of the simulation space
pub trait Region {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn get_random_coord(&self) -> [f64; 3];
}
