mod adjacent_procs;
mod domain;
mod worker;

pub(crate) mod comm;
pub(crate) mod message;

pub(crate) use adjacent_procs::AdjacentProcs;
pub(crate) use domain::Domain;
pub(crate) use worker::Worker;
