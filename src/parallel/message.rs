use std::{sync::mpsc, thread};

use crate::{
    atom_type::AtomType,
    output::{OutputSpec, Value},
    Atoms, Container,
};

use super::Worker;

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
}

/// Manager-to-Worker messages
pub enum M2W<T: AtomType> {
    Setup(Vec<thread::ThreadId>),
    Run(fn(&Worker<T>) -> ()),
    Sender(Option<mpsc::Sender<AtomMessage>>),
    ProcDims([usize; 3]),
}
