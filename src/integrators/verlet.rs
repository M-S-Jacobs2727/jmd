use super::Integrator;
use crate::{AtomicPotential, Simulation};

pub struct Verlet<P: AtomicPotential> {
    timestep: f64,
    simulation: Simulation<P>,
}

impl<'a, P: AtomicPotential> Verlet<P> {
    pub fn new(timestep: f64, simulation: Simulation<P>) -> Self {
        Self {
            timestep,
            simulation,
        }
    }

    fn increment_velocity_halfstep(&mut self, forces: &Vec<[f64; 3]>) {
        for i in 0..self.simulation.atoms.num_atoms() {
            let mass = self.simulation.atoms.masses()[i];
            self.simulation.atoms.increment_velocity(
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
        for i in 0..self.simulation.atoms.num_atoms() {
            let vel = self.simulation.atoms.velocities()[i];
            self.simulation.update_max_distance_sq(
                self.timestep * (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]),
            );

            self.simulation.atoms.increment_position(
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

impl<P: AtomicPotential> Integrator<P> for Verlet<P> {
    fn new(simulation: Simulation<P>) -> Self {
        Self {
            timestep: 0.005,
            simulation,
        }
    }
    fn run(&mut self, num_steps: usize) {
        self.simulation.forward_comm();
        self.simulation.build_neighbor_list();
        let mut forces = self.simulation.compute_forces();
        self.simulation.reverse_comm(&mut forces);

        for step in 0..num_steps {
            self.increment_velocity_halfstep(&forces);
            self.increment_positions();
            self.simulation.forward_comm();

            self.simulation.check_build_neighbor_list(&step);

            let mut forces = self.simulation.compute_forces();
            self.simulation.reverse_comm(&mut forces);

            self.increment_velocity_halfstep(&forces);
        }
    }
}
