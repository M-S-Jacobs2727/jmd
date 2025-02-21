use rand::prelude::*;
use rand::rng;
use rand_distr::Normal;

pub struct Atom {
    pub position: [f64; 3],
    pub velocity: [f64; 3],
    pub force: [f64; 3],
    pub mass: f64,
    pub atom_type: i64,
    pub id: usize,
}

pub fn zero_total_velocity(atoms: &mut Vec<Atom>) {
    let avg_velocity: [f64; 3] = atoms
        .iter()
        .map(|atom| atom.velocity)
        .reduce(|a, b| [a[0] + b[0], a[1] + b[1], a[2] + b[2]])
        .unwrap_or([0.0, 0.0, 0.0]);
    for atom in atoms.iter_mut() {
        atom.velocity[0] -= avg_velocity[0];
        atom.velocity[1] -= avg_velocity[1];
        atom.velocity[2] -= avg_velocity[2];
    }
}

pub fn scale_velocity_to_temperature(atoms: &mut Vec<Atom>, temperature: f64) {
    let current_v2 = atoms
        .iter()
        .map(|atom| {
            atom.velocity[0] * atom.velocity[0]
                + atom.velocity[1] * atom.velocity[1]
                + atom.velocity[2] * atom.velocity[2]
        })
        .sum::<f64>()
        / atoms.len() as f64;
    let factor = (temperature / current_v2).sqrt();
    for atom in atoms.iter_mut() {
        atom.velocity[0] *= factor;
        atom.velocity[1] *= factor;
        atom.velocity[2] *= factor;
    }
}

pub fn set_temperature(atoms: &mut Vec<Atom>, temperature: f64) {
    let mut rng = rng();
    let normal = Normal::new(0.0, 1.0).unwrap();

    for atom in atoms.iter_mut() {
        atom.velocity[0] = rng.sample(normal);
        atom.velocity[1] = rng.sample(normal);
        atom.velocity[2] = rng.sample(normal);
    }

    let v2 = atoms
        .iter()
        .map(|atom| {
            atom.velocity[0] * atom.velocity[0]
                + atom.velocity[1] * atom.velocity[1]
                + atom.velocity[2] * atom.velocity[2]
        })
        .sum::<f64>()
        / atoms.len() as f64;

    let factor = (3.0 * temperature / v2).sqrt();
    for atom in atoms.iter_mut() {
        atom.velocity[0] *= factor;
        atom.velocity[1] *= factor;
        atom.velocity[2] *= factor;
    }
}
