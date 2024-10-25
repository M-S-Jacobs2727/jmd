use std::thread;
use std::{sync::mpsc, time::Duration};

use crate::compute::ComputeValue;
use crate::{output::*, parallel::message as msg, parallel::Worker, Error, Simulation};

struct ThreadContainer {
    pub id: thread::ThreadId,
    pub tx: mpsc::Sender<msg::M2W>,
    pub handle: thread::JoinHandle<Result<(), Error>>,
}

/// Main app, used to run a function through parallel workers
pub struct Jmd {
    rx: mpsc::Receiver<msg::W2M>,
    tx: mpsc::Sender<msg::W2M>,
    threads: Vec<ThreadContainer>,
}
impl Jmd {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx,
            tx,
            threads: Vec::new(),
        }
    }
    fn setup(&mut self, num_threads: usize) {
        for _ in 0..num_threads {
            let (tx2, rx2) = mpsc::channel();
            let mut main_thread = Worker::new(rx2, self.tx.clone());

            let handle = thread::spawn(move || main_thread.run_thread());
            let result = self.rx.recv().expect("Disconnect error");
            if let msg::W2M::Id(id) = result {
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
        for thread in &self.threads {
            thread
                .tx
                .send(msg::M2W::Setup(thread_ids.clone()))
                .expect("Disconnect error");
        }
    }
    fn receive(&self) -> msg::W2M {
        self.rx.recv().expect("All procs diconnected")
    }
    fn output(&self, id: thread::ThreadId, value: ComputeValue, output_spec: &Vec<OutputSpec>) {
        let num_messages_expected = self.threads.len() * output_spec.len();
        let mut values: Vec<ComputeValue> = output_spec
            .iter()
            .map(|s| match s {
                OutputSpec::Step => ComputeValue::Usize(0),
                OutputSpec::KineticE
                | OutputSpec::PotentialE
                | OutputSpec::Temp
                | OutputSpec::TotalE => ComputeValue::Float(0.0),
            })
            .collect();

        let mut num_messages_per_thread: Vec<usize> = Vec::new();
        num_messages_per_thread.resize(self.threads.len(), 0);

        let mut handle_output_message = |id: thread::ThreadId, value: ComputeValue| {
            let idx = self
                .threads
                .iter()
                .position(|t| t.id == id)
                .expect("Invalid thread id");
            let v_idx = num_messages_per_thread[idx];
            match output_spec[v_idx] {
                OutputSpec::Step => {}
                OutputSpec::KineticE
                | OutputSpec::PotentialE
                | OutputSpec::Temp
                | OutputSpec::TotalE => values[v_idx] += value,
            };
            num_messages_per_thread[idx] += 1;
        };

        handle_output_message(id, value);
        for _i in 1..num_messages_expected {
            let message = self.receive();
            match message {
                msg::W2M::Output(id, value) => handle_output_message(id, value),
                _ => panic!("Invalid communication"),
            };
        }

        for v in values {
            print!("{}\t", v);
        }
        println!();
    }
    fn handle_message(
        &self,
        message: msg::W2M,
        threads_complete: &mut usize,
        output_spec: &mut Vec<OutputSpec>,
    ) -> Result<(), Error> {
        match message {
            msg::W2M::Error(e) => return Err(e),
            msg::W2M::Complete => *threads_complete += 1,
            msg::W2M::ProcDims(pd) => {
                for t in &self.threads {
                    t.tx.send(msg::M2W::ProcDims(pd.clone())).unwrap();
                }
            }
            msg::W2M::Sender(tx, idx) => self.threads[idx].tx.send(msg::M2W::Sender(tx)).unwrap(),
            msg::W2M::SetupOutput(specs) => *output_spec = specs,
            msg::W2M::Output(id, value) => self.output(id, value, &output_spec),
            msg::W2M::InitialOutput => self.initial_output(output_spec),
            _ => {}
        };
        Ok(())
    }
    fn manage_comm(&self) -> Result<(), Error> {
        let mut threads_complete = 0usize;
        let mut output_spec: Vec<OutputSpec> = Vec::new();
        loop {
            let result = self.rx.recv_timeout(Duration::from_millis(200));
            match result {
                Ok(message) => {
                    self.handle_message(message, &mut threads_complete, &mut output_spec)?
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => panic!("Thread disconnected"),
                _ => {}
            };
            if threads_complete == self.threads.len() {
                return Ok(());
            }
            for t in &self.threads {
                if t.handle.is_finished() {
                    panic!("Thread finished without message");
                }
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
            thread.tx.send(msg::M2W::Run(f)).unwrap();
        }
        self.manage_comm()
    }

    fn initial_output(&self, output_spec: &Vec<OutputSpec>) {
        for spec in output_spec {
            print!("{}\t", spec)
        }
        println!();
    }
}
