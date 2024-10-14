mod verlet;
pub use verlet::Verlet;

use enum_dispatch::enum_dispatch;

use crate::Simulation;

#[enum_dispatch]
pub enum Integrator {
    Verlet,
}
#[enum_dispatch(Integrator)]
/// Simulation integrator
pub trait IntegratorTrait {
    fn run(&self, simulation: &mut Simulation, num_steps: usize);
}
