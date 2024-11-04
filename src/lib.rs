// TODO: determine API
// TODO: make sure that all private members have public getters
// TODO: implement ndarray?
// TODO: check best implementation of "atom has moved half the bin size"

mod integrators;
mod jmd;
mod neighbor;
mod parallel;
mod traits;

pub mod atom_type;
pub mod atomic;
pub mod atoms;
pub mod compute;
pub mod container;
pub mod lattice;
pub mod output;
pub mod prelude;
pub mod region;
pub mod simulation;
pub mod utils;
