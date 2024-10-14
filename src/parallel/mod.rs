mod adjacent_procs;
mod domain;
mod worker;

pub use adjacent_procs::AdjacentProcs;
pub use domain::Domain;
pub use worker::{Worker, M2W, W2M};

pub struct AtomInfo {
    pub ids: Vec<usize>,
    pub types: Vec<u32>,
    pub data: Vec<f64>,
}
impl AtomInfo {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            types: Vec::new(),
            data: Vec::new(),
        }
    }
}
