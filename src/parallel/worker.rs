use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use crate::{parallel::AtomInfo, Atoms, Box_, Error};

pub type ThreadIds = Vec<ThreadId>;

pub enum W2M {
    Error(Error),
    Complete,
    Output(String),
    Dump(Atoms, Box_),
    Id(thread::ThreadId),
    Sender(mpsc::Sender<AtomInfo>),
}
pub enum M2W {
    Error(Error),
    Setup(Vec<thread::ThreadId>),
    Run(fn(&mut Worker) -> ()),
    Sender(mpsc::Sender<AtomInfo>),
}
pub struct Worker {
    rx: mpsc::Receiver<M2W>,
    tx: mpsc::Sender<W2M>,
    thread_ids: Vec<ThreadId>,
}
impl Worker {
    pub fn new(rx: mpsc::Receiver<M2W>, tx: mpsc::Sender<W2M>) -> Self {
        Self {
            rx,
            tx,
            thread_ids: Vec::new(),
        }
    }
    pub fn run_thread(&mut self) {
        self.tx
            .send(W2M::Id(thread::current().id()))
            .expect("Disconnect error");

        self.thread_ids = if let M2W::Setup(thread_ids) = self.rx.recv().expect("Disconnect error")
        {
            thread_ids
        } else {
            panic!("Invalid communication")
        };

        let msg = self.rx.recv().unwrap();
        match msg {
            M2W::Run(f) => f(self),
            M2W::Error(_) => return,
            _ => panic!("Invalid communication"),
        };
    }
}
