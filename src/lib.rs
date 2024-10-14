pub mod atomic;
// TODO: determine API
// TODO: make sure that all private members have public getters
// TODO: implement ndarray?
// TODO: check best implementation of "atom has moved half the bin size"

mod atoms;
mod container;
mod error;
mod integrators;
mod jmd;
mod neighbor;
mod parallel;
pub mod region;
mod simulation;
mod utils;

pub use atomic::*;
pub use atoms::Atoms;
pub use container::{Container, BC};
pub use error::Error;
pub use integrators::*;
pub use jmd::Jmd;
pub use neighbor::{NeighborList, UpdateSettings};
pub use simulation::Simulation;
pub use utils::{Axis, Direction};
