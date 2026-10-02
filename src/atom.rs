use rand::prelude::*;
use rand::rng;
use rand_distr::Normal;

pub struct Atoms {
    pub ids: Vec<usize>,
    pub atom_types: Vec<i64>,
    pub masses: Vec<f64>,
    pub positions: Vec<[f64; 3]>,
    pub velocities: Vec<[f64; 3]>,
    pub forces: Vec<[f64; 3]>,
}
impl Atoms {
    pub fn new() -> Self {
        Self {
            ids: vec![],
            atom_types: vec![],
            masses: vec![],
            positions: vec![],
            velocities: vec![],
            forces: vec![],
        }
    }
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn push(
        &mut self,
        id: usize,
        atom_type: i64,
        mass: f64,
        position: [f64; 3],
        velocity: [f64; 3],
        force: [f64; 3],
    ) {
        self.ids.push(id);
        self.atom_types.push(atom_type);
        self.masses.push(mass);
        self.positions.push(position);
        self.velocities.push(velocity);
        self.forces.push(force);
    }
    pub fn reserve(&mut self, n: usize) {
        self.ids.reserve(n);
        self.atom_types.reserve(n);
        self.masses.reserve(n);
        self.positions.reserve(n);
        self.velocities.reserve(n);
        self.forces.reserve(n);
    }
    pub fn zero_force(&mut self) {
        self.forces.iter_mut().for_each(|f| {*f = [0.0, 0.0, 0.0];});
    }
    pub fn zero_total_velocity(&mut self) {
        let avg_velocity: [f64; 3] = self
            .velocities
            .iter()
            .fold([0f64; 3], |a, b| [a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
        for v in self.velocities.iter_mut() {
            v[0] -= avg_velocity[0];
            v[1] -= avg_velocity[1];
            v[2] -= avg_velocity[2];
        }
    }

    pub fn scale_velocity_to_temperature(&mut self, temperature: f64) {
        let current_v2 = self
            .velocities
            .iter()
            .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
            .sum::<f64>()
            / self.velocities.len() as f64;
        let factor = (temperature / current_v2).sqrt();
        for v in self.velocities.iter_mut() {
            v[0] *= factor;
            v[1] *= factor;
            v[2] *= factor;
        }
    }

    pub fn set_temperature(&mut self, temperature: f64) {
        let mut rng = rng();
        let normal = Normal::new(0.0, 1.0).unwrap();

        for v in self.velocities.iter_mut() {
            v[0] = rng.sample(normal);
            v[1] = rng.sample(normal);
            v[2] = rng.sample(normal);
        }

        let v2 = self
            .velocities
            .iter()
            .map(|v| v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
            .sum::<f64>()
            / self.velocities.len() as f64;

        let factor = (3.0 * temperature / v2).sqrt();
        for v in self.velocities.iter_mut() {
            v[0] *= factor;
            v[1] *= factor;
            v[2] *= factor;
        }
    }
}
