use crate::Simulation;

pub(super) fn compute(sim: &Simulation) -> f64 {
    0.5 * sim
        .atoms
        .velocities
        .iter()
        .zip(sim.atoms.masses.iter())
        .take(sim.nlocal())
        .map(|(v, m)| m * (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]))
        .sum::<f64>()
}
