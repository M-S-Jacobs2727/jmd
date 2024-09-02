pub mod verlet;

pub trait Integrator {
    fn run(&mut self, num_steps: u64);
}