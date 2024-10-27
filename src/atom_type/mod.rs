mod basic;

pub use basic::Basic;

pub trait AtomType {
    fn mass(&self) -> f64;
}
