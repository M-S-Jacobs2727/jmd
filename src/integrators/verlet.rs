use super::Integrator;
use crate::atomic::AtomicPotential;
use crate::atoms::Atoms;

pub struct Verlet<T: AtomicPotential> {
    timestep: f64,
    simulation: Atoms,
    atomic_potential: T,
}

impl<T: AtomicPotential> Verlet<T> {
    pub fn new(timestep: f64, simulation: Atoms, atomic_potential: T) -> Self {
        Self {
            timestep,
            simulation,
            atomic_potential,
        }
    }

    fn increment_velocity_halfstep(&mut self, forces: &Vec<[f64; 3]>) {
        for i in 0..self.simulation.num_atoms() {
            let mass = self.simulation.masses()[i];
            self.simulation.increment_velocity(
                i,
                [
                    0.5 * self.timestep * forces[i][0] / mass,
                    0.5 * self.timestep * forces[i][1] / mass,
                    0.5 * self.timestep * forces[i][2] / mass,
                ],
            );
        }
    }

    fn increment_positions(&mut self) {
        for i in 0..self.simulation.num_atoms() {
            let vel = self.simulation.velocities()[i];
            self.simulation.increment_position(
                i,
                [
                    self.timestep * vel[0],
                    self.timestep * vel[1],
                    self.timestep * vel[2],
                ],
            );
        }
    }
}

impl<T: AtomicPotential> Integrator for Verlet<T> {
    fn run(&mut self, num_steps: u64) {
        let forces = self.atomic_potential.compute_forces(&self.simulation);

        for _step in 0..num_steps {
            self.increment_velocity_halfstep(&forces);
            self.increment_positions();
            let forces = self.atomic_potential.compute_forces(&mut self.simulation);
            self.increment_velocity_halfstep(&forces);
        }
    }
}
