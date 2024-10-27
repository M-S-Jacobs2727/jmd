use crate::Simulation;

pub(super) fn compute(simulation: &Simulation) -> f64 {
    simulation
        .atoms
        .velocities
        .iter()
        .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        .sum::<f64>()
        / simulation.atoms.num_atoms() as f64
}
