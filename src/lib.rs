pub mod atomic;
// TODO: determine API
// TODO: make sure that all private members have public getters
// TODO: look into using dyn AtomicPotential instead of generic (lots of potentials will make library larger)
// TODO: simplify box, change name?

pub mod atoms;
pub mod container;
pub mod error;
pub mod integrators;
pub mod jmd;
pub mod neighbor;
pub mod parallel;
pub mod region;
pub mod utils;

pub use atomic::*;
pub use atoms::Atoms;
pub use container::{Container, BC};
pub use error::Error;
pub use integrators::*;
pub use jmd::Jmd;
pub use neighbor::NeighborList;
pub use parallel::*;
