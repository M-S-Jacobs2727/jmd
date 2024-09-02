use super::AtomicPotential;
use crate::simulation::Simulation;
use crate::type_range::TypeRange;

pub struct LJCut {
    num_types: u32,
    coeffs: Vec<LJCutCoeff>,
}

pub struct LJCutCoeff {
    sigma: f64,
    epsilon: f64,
    rcut: f64,
    sigma6: f64,
    rcut2: f64,
    prefactor: f64, // = -24 epsilon * sigma^6
}
impl LJCutCoeff {
    pub fn new(sigma: f64, epsilon: f64, rcut: f64) -> Self {
        let sigma6 = sigma * sigma * sigma * sigma * sigma * sigma;
        Self {
            sigma,
            epsilon,
            rcut,
            rcut2: rcut * rcut,
            sigma6,
            prefactor: -24.0 * epsilon * sigma6,
        }
    }
    pub fn sigma(&self) -> f64 {
        self.sigma
    }
    pub fn epsilon(&self) -> f64 {
        self.epsilon
    }
    pub fn rcut(&self) -> f64 {
        self.rcut
    }
}

pub struct LJCutCoeffsInit {
    types_i: TypeRange,
    types_j: TypeRange,
    sigma: f64,
    epsilon: f64,
    rcut: f64,
}
impl LJCut {
    pub fn new(passed_coeffs: &Vec<LJCutCoeffsInit>, num_types: u32) -> Self {
        let mut coeffs: Vec<LJCutCoeff> = Vec::new();
        for i in 0..num_types {
            for j in 0..num_types {
                let val = passed_coeffs.iter().find(|c| {
                    (c.types_i.contains(i) && c.types_j.contains(j))
                        || (c.types_i.contains(j) && c.types_j.contains(i))
                });
                match val {
                    Some(coeff) => {
                        coeffs.push(LJCutCoeff::new(coeff.sigma, coeff.epsilon, coeff.rcut));
                    },
                    None => panic!("Coefficient list passed to LJCut::new is missing values for type pair ({i}, {j})"),
                };
            }
        }
        Self { num_types, coeffs }
    }
}

impl AtomicPotential for LJCut {
    fn compute_forces(&self, sim: &Simulation) -> Vec<[f64; 3]> {
        let mut forces: Vec<[f64; 3]> = Vec::new();
        forces.resize(sim.num_atoms(), [0.0, 0.0, 0.0]);
        for i in 0..sim.num_atoms() - 1 {
            for j in i + 1..sim.num_atoms() {
                // U(r) = 4 eps ((sig/r)^12 - (sig/r)^6) - const
                // f(r) = dU/dr = dU/d(r^2) d(r^2)/dr
                // f(r) = -24 r eps / r^2 (2(sig/r)^12 - (sig/r)^6)

                // If r_i = (0, 0) and r_j = (sig, 0), then the
                // force should be repulsive, ie., f_i ~ (-1, 0), f_j ~ (1, 0)
                // f(r_ij) = r_ij * -24 eps / sig^2, so if r_ij = r_j - r_i = (sig, 0),
                // then f_i = f(r_ij) and f_j = -f(r_ij)

                let idx = self.type_index(sim.type_(i), sim.type_(j));
                let coeff = &self.coeffs[idx];
                let pos_i = sim.position(i);
                let pos_j = sim.position(j);
                let r = [
                    pos_i[0] - pos_j[0],
                    pos_i[1] - pos_j[1],
                    pos_i[2] - pos_j[2],
                ];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];

                if r2 > coeff.rcut2 {
                    continue;
                }

                let r6 = r2 * r2 * r2;

                let f_mag = coeff.prefactor / r6 / r2 * (2.0 * coeff.sigma6 / r6 - 1.0);
                forces[i] = [r[0] * f_mag, r[1] * f_mag, r[2] * f_mag];
                forces[j] = [-forces[i][0], -forces[i][1], -forces[i][2]];
            }
        }

        forces
    }

    fn type_index(&self, typei: u32, typej: u32) -> usize {
        (typei * self.num_types + typej)
            .try_into()
            .expect("type_index function should convert to type usize")
    }
}
