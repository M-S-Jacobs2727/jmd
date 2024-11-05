use std::{sync::mpsc, thread};

use crate::{
    atom_type::AtomType,
    atomic::AtomicPotentialTrait,
    atoms::Atoms,
    container::Container,
    output::{OutputSpec, Value},
    simulation::Simulation,
};

/// Message between procs communicating atom info
pub enum AtomMessage {
    Float3(Vec<[f64; 3]>),
    Float(Vec<f64>),
    Int3(Vec<[i32; 3]>),
    Types(Vec<usize>),
    Idxs(Vec<usize>),
}

/// Worker-to-Manager messages
pub enum W2M<T: AtomType> {
    Complete,
    Output(thread::ThreadId, Value),
    Dump(Atoms<T>, Container),
    Id(thread::ThreadId),
    Sender(Option<mpsc::Sender<AtomMessage>>, usize),
    ProcDims([usize; 3]),
    SetupOutput(Vec<OutputSpec>),
    InitialOutput,
    Sum(usize),
}

/// Manager-to-Worker messages
#[derive(Clone, Debug)]
pub enum M2W<T: AtomType, A: AtomicPotentialTrait<T>> {
    Setup(Vec<thread::ThreadId>),
    Run(fn(Simulation<T, A>) -> ()),
    Sender(Option<mpsc::Sender<AtomMessage>>),
    ProcDims([usize; 3]),
    SumResult(usize),
}
