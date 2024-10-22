use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use crate::{parallel::message as msg, Error, Simulation};

/// Channels for communication between each process and the manager
pub struct Worker {
    rx: mpsc::Receiver<msg::M2W>,
    tx: mpsc::Sender<msg::W2M>,
    thread_ids: Vec<ThreadId>,
}
impl Worker {
    pub fn new(rx: mpsc::Receiver<msg::M2W>, tx: mpsc::Sender<msg::W2M>) -> Self {
        Self {
            rx,
            tx,
            thread_ids: Vec::new(),
        }
    }
    pub fn run_thread(&mut self) -> Result<(), Error> {
        self.tx
            .send(msg::W2M::Id(thread::current().id()))
            .expect("Disconnect error");

        self.thread_ids =
            if let msg::M2W::Setup(thread_ids) = self.rx.recv().expect("Disconnect error") {
                thread_ids
            } else {
                panic!("Invalid communication")
            };

        self.run()
    }
    pub fn thread_ids(&self) -> &Vec<ThreadId> {
        &self.thread_ids
    }
    pub fn send(&self, message: msg::W2M) {
        self.tx.send(message).expect("Disconnect error");
    }
    pub fn recv(&self) -> Result<msg::M2W, mpsc::RecvError> {
        self.rx.recv()
    }
    pub fn try_recv(&self) -> Result<msg::M2W, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
    pub fn error(&self, e: Error) {
        self.send(msg::W2M::Error(e));
    }

    fn run(&self) -> Result<(), Error> {
        let msg = self.rx.recv().unwrap();
        let res = match msg {
            msg::M2W::Run(f) => {
                let mut sim = Simulation::new();
                let b = Box::new(self);
                sim.init(b);
                f(&mut sim)
            }
            msg::M2W::Error(_) => Err(Error::OtherError),
            _ => panic!("Invalid communication"),
        };
        match res {
            Ok(_) => self.tx.send(msg::W2M::Complete).unwrap(),
            Err(e) => self.tx.send(msg::W2M::Error(e)).unwrap(),
        }
        res
    }
}
