use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use crate::{atom_type::AtomType, parallel::message as msg, Error};

/// Channels for communication between each process and the manager
pub struct Worker<T: AtomType> {
    rx: mpsc::Receiver<msg::M2W<T>>,
    tx: mpsc::Sender<msg::W2M<T>>,
    thread_ids: Vec<ThreadId>,
}
impl<T: AtomType> Worker<T> {
    pub fn new(rx: mpsc::Receiver<msg::M2W<T>>, tx: mpsc::Sender<msg::W2M<T>>) -> Self {
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
    pub fn send(&self, message: msg::W2M<T>) {
        self.tx.send(message).expect("Disconnect error");
    }
    pub fn recv(&self) -> Result<msg::M2W<T>, mpsc::RecvError> {
        self.rx.recv()
    }
    pub fn try_recv(&self) -> Result<msg::M2W<T>, mpsc::TryRecvError> {
        self.rx.try_recv()
    }
    pub fn error(&self, e: Error) {
        self.send(msg::W2M::Error(e));
    }

    fn run(&self) -> Result<(), Error> {
        let msg = self.rx.recv().unwrap();
        let res = match msg {
            msg::M2W::Run(f) => f(self),
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
