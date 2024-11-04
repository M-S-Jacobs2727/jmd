use std::{sync::mpsc, thread};

use super::*;
use crate::{atom_type::AtomType, atomic::AtomicPotentialTrait, simulation::Simulation};

/// Channels for communication between each process and the manager
pub struct Worker<T: AtomType, A: AtomicPotentialTrait<T>> {
    rx: mpsc::Receiver<message::M2W<T, A>>,
    tx: mpsc::Sender<message::W2M<T>>,
    thread_ids: Vec<thread::ThreadId>,
}
impl<T: AtomType, A: AtomicPotentialTrait<T>> Worker<T, A> {
    pub fn new(rx: mpsc::Receiver<message::M2W<T, A>>, tx: mpsc::Sender<message::W2M<T>>) -> Self {
        Self {
            rx,
            tx,
            thread_ids: Vec::new(),
        }
    }
    pub fn run_thread(&mut self) {
        self.tx
            .send(message::W2M::Id(thread::current().id()))
            .expect("Disconnect error");

        self.thread_ids =
            if let message::M2W::Setup(thread_ids) = self.rx.recv().expect("Disconnect error") {
                thread_ids
            } else {
                panic!("Invalid communication")
            };

        self.run();
    }
    pub fn thread_ids(&self) -> &Vec<thread::ThreadId> {
        &self.thread_ids
    }
    pub fn send(&self, message: message::W2M<T>) {
        self.tx.send(message).expect("Disconnect error");
    }
    pub fn recv(&self) -> message::M2W<T, A> {
        self.rx.recv().expect("Disconnect error")
    }
    pub fn try_recv(&self) -> Result<message::M2W<T, A>, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    fn run(&self) {
        let message = self.rx.recv().unwrap();
        match message {
            message::M2W::Run(f) => f(Simulation::new()),
            _ => panic!("Invalid communication"),
        };
    }
}
