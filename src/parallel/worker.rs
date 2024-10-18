use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use crate::{parallel::message::Message, Atoms, Container, Error, Simulation};

/// Worker-to-Manager messages
pub enum W2M {
    Error(Error),
    Complete,
    Output(String),
    Dump(Atoms, Container),
    Id(thread::ThreadId),
    Sender(Option<mpsc::Sender<Message>>, usize),
    ProcDims([usize; 3]),
}
/// Manager-to-Worker messages
pub enum M2W {
    Error(Error),
    Setup(Vec<thread::ThreadId>),
    Run(fn(&mut Simulation) -> ()),
    Sender(Option<mpsc::Sender<Message>>),
    ProcDims([usize; 3]),
}
/// Channels for communication between each process and the manager
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

        self.run()
    }
    pub fn thread_ids(&self) -> &Vec<ThreadId> {
        &self.thread_ids
    }
    pub fn send(&self, message: W2M) -> Result<(), mpsc::SendError<W2M>> {
        self.tx.send(message)
    }
    pub fn recv(&self) -> Result<M2W, mpsc::RecvError> {
        self.rx.recv()
    }
    pub fn try_recv(&self) -> Result<M2W, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
    pub fn error(&self, e: Error) {
        self.send(W2M::Error(e)).unwrap();
    }

    fn run(&self) {
        let msg = self.rx.recv().unwrap();
        match msg {
            M2W::Run(f) => {
                let mut sim = Simulation::new();
                let b = Box::new(self);
                sim.init(b);
                f(&mut sim);
            }
            M2W::Error(_) => return,
            _ => panic!("Invalid communication"),
        };
        self.tx.send(W2M::Complete).unwrap();
    }
}
