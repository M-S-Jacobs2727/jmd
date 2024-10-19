// TODO: add more to handle_messages
use std::sync::mpsc;
use std::thread;

use crate::{
    parallel::{Worker, M2W, W2M},
    Error, Simulation,
};

/// Main app, used to run a function through parallel workers
pub struct Jmd {
    rx: mpsc::Receiver<W2M>,
    tx: mpsc::Sender<W2M>,
    threads: Vec<mpsc::Sender<M2W>>,
}
impl Jmd {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let threads: Vec<mpsc::Sender<M2W>> = Vec::new();
        Self { rx, tx, threads }
    }
    fn setup(&mut self, num_threads: usize) {
        let mut thread_ids: Vec<thread::ThreadId> = Vec::new();

        for _ in 0..num_threads {
            let (tx2, rx2) = mpsc::channel();
            let mut main_thread = Worker::new(rx2, self.tx.clone());
            self.threads.push(tx2);

            thread::spawn(move || main_thread.run_thread());
            let result = self.rx.recv().expect("Disconnect error");
            if let W2M::Id(id) = result {
                thread_ids.push(id);
            } else {
                panic!("Invalid communication");
            }
        }

        for thread in &self.threads {
            thread
                .send(M2W::Setup(thread_ids.clone()))
                .expect("Disconnect error");
        }
    }
    fn handle_messages(&self) -> Result<(), Error> {
        let mut threads_complete = 0;
        loop {
            let message = self.rx.recv().expect("All procs disconnected");
            match message {
                W2M::Error(e) => return Err(e),
                W2M::Complete => threads_complete += 1,
                W2M::ProcDims(pd) => {
                    for t in &self.threads {
                        t.send(M2W::ProcDims(pd.clone())).unwrap();
                    }
                }
                W2M::Sender(tx, idx) => self.threads[idx].send(M2W::Sender(tx)).unwrap(),
                _ => {}
            };
            if threads_complete == self.threads.len() {
                return Ok(());
            }
        }
    }
    pub fn run(
        &mut self,
        num_threads: usize,
        f: fn(&mut Simulation) -> Result<(), Error>,
    ) -> Result<(), Error> {
        self.setup(num_threads);
        for thread in &self.threads {
            thread.send(M2W::Run(f)).unwrap();
        }
        self.handle_messages()
    }
}
