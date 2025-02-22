use crate::{Atom, Domain};

use crate::constraint::Constraint;

pub struct LJCut {
    pub force_cutoff: f64,
    pub coeffs: Vec<LJCutCoeff>,
}

pub struct LJCutCoeff {
    type_i: i64,
    type_j: i64,
    epsilon: f64,
    sigma: f64,
    cutoff: f64,
    cutoff_sq: f64,
    sigma6: f64,
    four_epsilon_sigma6: f64,
}

impl LJCutCoeff {
    pub fn new(type_i: i64, type_j: i64, epsilon: f64, sigma: f64, cutoff: f64) -> Self {
        let cutoff_sq = cutoff * cutoff;
        let sigma2 = sigma * sigma;
        let sigma6 = sigma2 * sigma2 * sigma2;
        let four_epsilon_sigma6 = 4.0 * epsilon * sigma6;
        Self {
            type_i,
            type_j,
            epsilon,
            sigma,
            cutoff,
            cutoff_sq,
            sigma6,
            four_epsilon_sigma6,
        }
    }
    pub fn type_i(&self) -> i64 {
        self.type_i
    }
    pub fn type_j(&self) -> i64 {
        self.type_j
    }
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }
    pub fn sigma(&self) -> f64 {
        self.sigma
    }
    pub fn cutoff(&self) -> f64 {
        self.cutoff
    }
    pub fn cutoff_sq(&self) -> f64 {
        self.cutoff_sq
    }
    pub fn sigma6(&self) -> f64 {
        self.sigma6
    }
    pub fn four_epsilon_sigma6(&self) -> f64 {
        self.four_epsilon_sigma6
    }
}

impl LJCut {
    pub fn new(force_cutoff: f64) -> Self {
        Self {
            force_cutoff,
            coeffs: vec![],
        }
    }

    pub fn add_coeff(&mut self, coeff: LJCutCoeff) {
        self.coeffs.push(coeff);
    }

    pub fn get_coeff(&self, type_i: i64, type_j: i64) -> Option<&LJCutCoeff> {
        self.coeffs
            .iter()
            .find(|coeff| coeff.type_i == type_i && coeff.type_j == type_j)
    }
    pub fn energy(&self, atoms: &Vec<Atom>, _domain: &Domain) -> f64 {
        let mut energy = 0.0;
        for i in 0..atoms.len() {
            let type_i = atoms[i].atom_type;
            for j in i + 1..atoms.len() {
                let type_j = atoms[j].atom_type;
                let r = [
                    atoms[i].position[0] - atoms[j].position[0],
                    atoms[i].position[1] - atoms[j].position[1],
                    atoms[i].position[2] - atoms[j].position[2],
                ];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
                if let Some(coeff) = self.get_coeff(type_i, type_j) {
                    if r2 > coeff.cutoff_sq {
                        continue;
                    }
                    let r6 = r2 * r2 * r2;
                    energy += coeff.four_epsilon_sigma6 * (coeff.sigma6 / r6 - 1.0) / r6;
                }
            }
        }
        energy
    }
}

impl Constraint for LJCut {
    fn compute_force(&mut self, atoms: &mut Vec<Atom>, _domain: &mut Domain) {
        atoms.iter_mut().for_each(|atom| atom.force = [0.0; 3]);
        for i in 0..atoms.len() {
            let type_i = atoms[i].atom_type;

            for j in i + 1..atoms.len() {
                let type_j = atoms[j].atom_type;
                let r = [
                    atoms[i].position[0] - atoms[j].position[0],
                    atoms[i].position[1] - atoms[j].position[1],
                    atoms[i].position[2] - atoms[j].position[2],
                ];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
                if let Some(coeff) = self.get_coeff(type_i, type_j) {
                    if r2 > coeff.cutoff_sq {
                        continue;
                    }
                    let r6 = r2 * r2 * r2;
                    let force_div_r =
                        6.0 * coeff.four_epsilon_sigma6 * (2.0 * coeff.sigma6 / r6 - 1.0) / r2 / r6;
                    atoms[i].force[0] += force_div_r * r[0];
                    atoms[i].force[1] += force_div_r * r[1];
                    atoms[i].force[2] += force_div_r * r[2];
                    atoms[j].force[0] -= force_div_r * r[0];
                    atoms[j].force[1] -= force_div_r * r[1];
                    atoms[j].force[2] -= force_div_r * r[2];
                }
            }
        }
    }
}
