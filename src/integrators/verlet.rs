use super::*;

/// Velocity-verlet integrator
pub struct Verlet {}

impl Verlet {
    /// Steps the velocities of the simulation by half a timestep
    fn increment_velocity_halfstep<T, A>(simulation: &mut Simulation<T, A>)
    where
        T: AtomType,
        A: AtomicPotentialTrait<T>,
    {
        let half_ts = 0.5 * simulation.timestep();
        for i in 0..simulation.atoms.num_local_atoms() {
            let mass = simulation.atoms.mass(i);
            simulation.atoms.increment_velocity(
                i,
                [
                    half_ts * simulation.forces()[i][0] / mass,
                    half_ts * simulation.forces()[i][1] / mass,
                    half_ts * simulation.forces()[i][2] / mass,
                ],
            );
        }
    }
    /// Steps the positions of the simulation forward
    fn increment_positions<T, A>(simulation: &mut Simulation<T, A>)
    where
        T: AtomType,
        A: AtomicPotentialTrait<T>,
    {
        let ts = simulation.timestep();
        for i in 0..simulation.atoms.num_local_atoms() {
            let vel = simulation.atoms.velocities()[i];

            simulation
                .atoms
                .increment_position(i, [ts * vel[0], ts * vel[1], ts * vel[2]]);
        }
    }
}

impl<T, A> Integrator<T, A> for Verlet
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    fn pre_forward_comm(simulation: &mut Simulation<T, A>) {
        Verlet::increment_velocity_halfstep(simulation);
        Verlet::increment_positions(simulation);
    }
    fn post_reverse_comm(simulation: &mut Simulation<T, A>) {
        Verlet::increment_velocity_halfstep(simulation);
    }
}
