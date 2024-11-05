use std::{sync::mpsc, thread};

use super::*;
use crate::{atom_type::AtomType, atomic::AtomicPotentialTrait, simulation::Simulation};

/// Channels for communication between each process and the manager
pub struct Worker<T: AtomType, A: AtomicPotentialTrait<T>> {
    rx: mpsc::Receiver<M2W<T, A>>,
    tx: mpsc::Sender<W2M<T>>,
    thread_ids: Vec<thread::ThreadId>,
}
impl<T: AtomType, A: AtomicPotentialTrait<T>> Worker<T, A> {
    pub fn new(rx: mpsc::Receiver<M2W<T, A>>, tx: mpsc::Sender<W2M<T>>) -> Self {
        Self {
            rx,
            tx,
            thread_ids: Vec::new(),
        }
    }
    pub fn run_thread(&mut self) {
        self.send(W2M::Id(thread::current().id()));

        self.thread_ids = if let M2W::Setup(thread_ids) = self.recv() {
            thread_ids
        } else {
            panic!("Invalid communication")
        };

        self.run();
    }
    pub fn thread_ids(&self) -> &Vec<thread::ThreadId> {
        &self.thread_ids
    }
    pub fn send(&self, message: W2M<T>) {
        self.tx.send(message).expect("Disconnect error");
    }
    pub fn recv(&self) -> M2W<T, A> {
        self.rx.recv().expect("Disconnect error")
    }
    pub fn try_recv(&self) -> Result<M2W<T, A>, mpsc::TryRecvError> {
        self.rx.try_recv()
    }

    fn run(&self) {
        let message = self.recv();
        match message {
            M2W::Run(f) => {
                let mut sim = Simulation::new();
                sim.connect(Box::new(self));
                f(sim);
            }
            _ => panic!("Invalid communication"),
        };
    }
}
