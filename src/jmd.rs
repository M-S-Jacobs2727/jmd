use std::{sync::mpsc, thread, time::Duration};

use crate::{
    atom_type::AtomType,
    atomic::AtomicPotentialTrait,
    output::{Operatable, Operation, OutputSpec, Value},
    parallel::{Worker, M2W, W2M},
    simulation::Simulation,
};

struct ThreadContainer<T: AtomType, A: AtomicPotentialTrait<T>> {
    pub id: thread::ThreadId,
    pub tx: mpsc::Sender<M2W<T, A>>,
    pub handle: thread::JoinHandle<()>,
}

/// Main app, used to run a function through parallel workers
pub struct Jmd<T: AtomType, A: AtomicPotentialTrait<T>> {
    rx: mpsc::Receiver<W2M<T>>,
    tx: mpsc::Sender<W2M<T>>,
    threads: Vec<ThreadContainer<T, A>>,
}
impl<T, A> Jmd<T, A>
where
    T: AtomType + Send + 'static,
    A: AtomicPotentialTrait<T> + Send + 'static,
{
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx,
            tx,
            threads: Vec::new(),
        }
    }
    pub fn run(&mut self, num_threads: usize, f: fn(Simulation<T, A>) -> ()) {
        self.setup(num_threads);
        for t in 0..self.threads.len() {
            self.send(t, M2W::Run(f));
        }
        self.manage_comm();
    }

    fn setup(&mut self, num_threads: usize) {
        for _ in 0..num_threads {
            let (tx2, rx2) = mpsc::channel();
            let mut thread = Worker::new(rx2, self.tx.clone());

            let handle = thread::spawn(move || thread.run_thread());
            let result = self.recv();
            if let W2M::Id(id) = result {
                self.threads.push(ThreadContainer {
                    id,
                    tx: tx2,
                    handle,
                });
            } else {
                panic!("Invalid communication");
            }
        }

        let thread_ids: Vec<thread::ThreadId> = self.threads.iter().map(|c| c.id).collect();
        for t in 0..self.threads.len() {
            self.send(t, M2W::Setup(thread_ids.clone()));
        }
    }
    fn recv(&self) -> W2M<T> {
        self.rx.recv().expect("Disconnect error")
    }
    fn send(&self, thread_idx: usize, msg: M2W<T, A>) {
        self.threads[thread_idx]
            .tx
            .send(msg)
            .expect("Disconnect error");
    }
    fn initial_output(&self, output_spec: &Vec<OutputSpec>) {
        for spec in output_spec {
            print!("{}\t", spec)
        }
        println!();
    }
    fn output(&self, id: thread::ThreadId, value: Value, output_spec: &Vec<OutputSpec>) {
        let num_messages_expected = self.threads.len() * output_spec.len();
        let mut num_messages_per_thread: Vec<usize> = Vec::new();
        num_messages_per_thread.resize(self.threads.len(), 0);

        let mut values_per_thread: Vec<Vec<Value>> = Vec::new();
        values_per_thread.resize_with(self.threads.len(), || Vec::new());
        let idx = self
            .threads
            .iter()
            .position(|t| t.id == id)
            .expect("Invalid thread id");
        values_per_thread[idx].push(value);
        for _i in 1..num_messages_expected {
            let message = self.recv();
            match message {
                W2M::Output(id, value) => {
                    let idx = self
                        .threads
                        .iter()
                        .position(|t| t.id == id)
                        .expect("Invalid thread id");
                    values_per_thread[idx].push(value);
                }
                _ => {}
            };
        }

        let values: Vec<Value> = output_spec
            .iter()
            .enumerate()
            .map(|(i, spec)| match spec {
                OutputSpec::Step => values_per_thread[0][i].clone(),
                OutputSpec::Compute(c) => values_per_thread
                    .iter()
                    .map(|vec| vec[i].clone())
                    .reduce(|acc, v| match c.op() {
                        Operation::Sum => acc + v,
                        Operation::First => acc,
                        Operation::Max => acc.max(v),
                        Operation::Min => acc.min(v),
                    })
                    .expect("No threads"),
            })
            .collect();

        for v in values {
            print!("{}\t", v);
        }
        println!();
    }
    fn sum(&self, mut value: usize) {
        for _ in 0..self.threads.len() - 1 {
            let message = self.recv();
            match message {
                W2M::Sum(v) => value += v,
                _ => panic!("Invalid message"),
            }
        }
        for t in 0..self.threads.len() {
            self.send(t, M2W::SumResult(value));
        }
    }
    fn handle_message(
        &self,
        message: W2M<T>,
        threads_complete: &mut usize,
        output_spec: &mut Vec<OutputSpec>,
    ) {
        match message {
            W2M::Complete => *threads_complete += 1,
            W2M::ProcDims(pd) => {
                for t in &self.threads {
                    t.tx.send(M2W::ProcDims(pd.clone())).unwrap();
                }
            }
            W2M::Sender(tx, idx) => self.threads[idx].tx.send(M2W::Sender(tx)).unwrap(),
            W2M::SetupOutput(specs) => *output_spec = specs,
            W2M::Output(id, value) => self.output(id, value, &output_spec),
            W2M::InitialOutput => self.initial_output(output_spec),
            W2M::Sum(value) => self.sum(value),
            _ => {}
        };
    }
    fn manage_comm(&self) {
        let mut threads_complete = 0usize;
        let mut output_spec: Vec<OutputSpec> = Vec::new();
        loop {
            let result = self.rx.recv_timeout(Duration::from_millis(200));
            match result {
                Ok(message) => {
                    self.handle_message(message, &mut threads_complete, &mut output_spec)
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("Thread disconnected"),
                _ => {}
            };
            if threads_complete == self.threads.len() {
                return;
            }
            for t in &self.threads {
                if t.handle.is_finished() {
                    panic!("Thread finished without message");
                }
            }
        }
    }
}
