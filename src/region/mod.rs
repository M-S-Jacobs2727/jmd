use enum_dispatch::enum_dispatch;

mod rect;
pub use rect::Rect;

#[enum_dispatch]
pub enum Region {
    Rect,
}
#[enum_dispatch(Region)]
/// A region of the simulation space
pub trait RegionTrait {
    fn contains(&self, coord: &[f64; 3]) -> bool;
    fn get_random_coord(&self) -> [f64; 3];
}
