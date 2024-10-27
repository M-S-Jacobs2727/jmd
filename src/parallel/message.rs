use std::{sync::mpsc, thread};

use crate::{
    output::{OutputSpec, Value},
    Atoms, Container, Error, Simulation,
};

/// Message between procs communicating atom info
pub enum AtomMessage {
    Float3(Vec<[f64; 3]>),
    Float(Vec<f64>),
    Int3(Vec<[i32; 3]>),
    Types(Vec<u32>),
    Idxs(Vec<usize>),
}

/// Worker-to-Manager messages
pub enum W2M {
    Error(Error),
    Complete,
    Output(thread::ThreadId, Value),
    Dump(Atoms, Container),
    Id(thread::ThreadId),
    Sender(Option<mpsc::Sender<AtomMessage>>, usize),
    ProcDims([usize; 3]),
    SetupOutput(Vec<OutputSpec>),
    InitialOutput,
}

/// Manager-to-Worker messages
pub enum M2W {
    Error(Error),
    Setup(Vec<thread::ThreadId>),
    Run(fn(&mut Simulation) -> Result<(), Error>),
    Sender(Option<mpsc::Sender<AtomMessage>>),
    ProcDims([usize; 3]),
}
