// TODO: determine API
// TODO: make sure that all private members have public getters
// TODO: implement ndarray?
// TODO: check best implementation of "atom has moved half the bin size"

pub mod atom_type;
pub mod atomic;
mod atoms;
mod compute;
mod container;
mod integrators;
mod jmd;
mod lattice;
mod neighbor;
mod output;
mod parallel;
mod region;
mod simulation;
mod traits;
pub mod utils;

pub use atomic::AtomicPotentialTrait;
pub use atoms::Atoms;
pub use compute::*;
pub use container::{Container, BC};
pub use integrators::*;
pub use jmd::Jmd;
pub use lattice::*;
pub use neighbor::NeighborList;
pub use output::{Output, OutputSpec};
pub use parallel::Worker;
pub use region::*;
pub use simulation::Simulation;
pub use utils::{Axis, Direction};
