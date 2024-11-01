use super::Integrator;
use crate::{atom_type::AtomType, AtomicPotentialTrait, Simulation};

/// Velocity-verlet integrator
pub struct Verlet {
    pub timestep: f64,
}

impl Verlet {
    pub fn new() -> Self {
        Self { timestep: 0.005 }
    }
    /// Steps the velocities of the simulation by half a timestep
    fn increment_velocity_halfstep<T, A>(
        &self,
        simulation: &mut Simulation<T, A>,
        forces: &Vec<[f64; 3]>,
    ) where
        T: AtomType,
        A: AtomicPotentialTrait<T>,
    {
        for i in 0..simulation.atoms.num_atoms() {
            simulation.atoms.increment_velocity(
                i,
                [
                    0.5 * self.timestep * forces[i][0] / simulation.atoms.mass(i),
                    0.5 * self.timestep * forces[i][1] / simulation.atoms.mass(i),
                    0.5 * self.timestep * forces[i][2] / simulation.atoms.mass(i),
                ],
            );
        }
    }
    /// Steps the positions of the simulation forward
    fn increment_positions<T, A>(&self, simulation: &mut Simulation<T, A>)
    where
        T: AtomType,
        A: AtomicPotentialTrait<T>,
    {
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

impl<T, A> Integrator<T, A> for Verlet
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn run(&self, simulation: &mut Simulation<T, A>, num_steps: usize) {
        simulation.initial_output();

        simulation.forward_comm();
        simulation.build_neighbor_list();
        dbg!(simulation.atoms.positions.len());
        dbg!(simulation.neighbor_list.neighbors().len());
        let mut forces = simulation.compute_forces();
        simulation.reverse_comm(&mut forces);
        simulation.check_do_output(&0);

        for step in 1..=num_steps {
            self.increment_velocity_halfstep(simulation, &forces);
            self.increment_positions(simulation);
            simulation.forward_comm();

            simulation.check_build_neighbor_list(step);
            dbg!(simulation.atoms.positions.len());
            dbg!(simulation.neighbor_list.neighbors().len());

            let mut forces = simulation.compute_forces();
            simulation.reverse_comm(&mut forces);

            self.increment_velocity_halfstep(simulation, &forces);

            simulation.check_do_output(&step);
        }
    }
}
