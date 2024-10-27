use super::Integrator;
use crate::Simulation;

/// Velocity-verlet integrator
pub struct Verlet {
    pub timestep: f64,
}

impl Verlet {
    pub fn new() -> Self {
        Self { timestep: 0.005 }
    }
    /// Steps the velocities of the simulation by half a timestep
    fn increment_velocity_halfstep(&self, simulation: &mut Simulation, forces: &Vec<[f64; 3]>) {
        for i in 0..simulation.atoms.num_atoms() {
            let mass = simulation.atoms.masses()[i];
            simulation.atoms.increment_velocity(
                i,
                [
                    0.5 * self.timestep * forces[i][0] / mass,
                    0.5 * self.timestep * forces[i][1] / mass,
                    0.5 * self.timestep * forces[i][2] / mass,
                ],
            );
        }
    }
    /// Steps the positions of the simulation forward
    fn increment_positions(&self, simulation: &mut Simulation) {
        for i in 0..simulation.atoms.num_atoms() {
            let vel = simulation.atoms.velocities()[i];

            simulation.atoms.increment_position(
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

impl Integrator for Verlet {
    fn run(&self, simulation: &mut Simulation, num_steps: usize) {
        simulation.initial_output();

        simulation.forward_comm();
        simulation.build_neighbor_list();
        let mut forces = simulation.compute_forces();
        simulation.reverse_comm(&mut forces);
        simulation.check_do_output(&0);

        for step in 1..=num_steps {
            self.increment_velocity_halfstep(simulation, &forces);
            self.increment_positions(simulation);
            simulation.forward_comm();

            simulation.check_build_neighbor_list(&step);

            let mut forces = simulation.compute_forces();
            simulation.reverse_comm(&mut forces);

            self.increment_velocity_halfstep(simulation, &forces);

            simulation.check_do_output(&step);
            // dbg!(&simulation.atoms.positions);
        }
    }
}
