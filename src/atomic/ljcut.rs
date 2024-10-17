use super::AtomicPotentialTrait;
use crate::{utils::Types, Atoms, Error};

#[derive(Clone, Copy, Debug)]
struct LJCutCoeff {
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

/// Lennard-Jones 12-6 potential
pub struct LJCut {
    num_types: usize,
    force_cutoff: f64,
    coeffs: Vec<LJCutCoeff>,
    coeff_set: Vec<bool>,
}
impl LJCut {
    pub fn new() -> Self {
        Self {
            num_types: 0,
            force_cutoff: 2.5,
            coeffs: Vec::new(),
            coeff_set: Vec::new(),
        }
    }
    pub fn set_coeff(
        &mut self,
        type_i: Types,
        type_j: Types,
        sigma: f64,
        epsilon: f64,
        rcut: f64,
    ) -> Result<(), Error> {
        let itypes = type_i.to_vec();
        let jtypes = type_j.to_vec();
        if itypes.iter().any(|&t| t >= self.num_types)
            || jtypes.iter().any(|&t| t >= self.num_types)
        {
            return Err(Error::AtomicPotentialError);
        }

        let coeff = LJCutCoeff::new(sigma, epsilon, rcut);
        for i in &itypes {
            for j in &jtypes {
                let index = (i * self.num_types + j) as usize;
                self.coeff_set[index] = true;
                self.coeffs[index] = coeff.clone();
            }
        }

        Ok(())
    }
    pub fn all_set(&self) -> bool {
        self.coeff_set.iter().all(|&x| x)
    }
    fn default_coeff(&self) -> LJCutCoeff {
        LJCutCoeff::new(0.0, 0.0, 0.0)
    }
}

impl AtomicPotentialTrait for LJCut {
    fn cutoff_distance(&self) -> f64 {
        self.force_cutoff.clone()
    }
    fn compute_forces(&self, atoms: &Atoms) -> Vec<[f64; 3]> {
        let mut forces: Vec<[f64; 3]> = Vec::new();
        forces.resize(atoms.num_atoms(), [0.0, 0.0, 0.0]);
        for i in 0..atoms.num_atoms() - 1 {
            let typei = &atoms.types[i];
            let posi = &atoms.positions[i];
            for j in i + 1..atoms.num_atoms() {
                // U(r) = 4 eps ((sig/r)^12 - (sig/r)^6) - const
                // f(r) = dU/dr = dU/d(r^2) d(r^2)/dr
                // f(r) = -24 r eps / r^2 (2(sig/r)^12 - (sig/r)^6)

                // If r_i = (0, 0) and r_j = (sig, 0), then the
                // force should be repulsive, ie., f_i ~ (-1, 0), f_j ~ (1, 0)
                // f(r_ij) = r_ij * -24 eps / sig^2, so if r_ij = r_j - r_i = (sig, 0),
                // then f_i = f(r_ij) and f_j = -f(r_ij)

                let typej = &atoms.types[j];
                let posj = &atoms.positions[j];
                let coeff = self.coeffs[self.num_types * *typei as usize + *typej as usize];
                let r = [posi[0] - posj[0], posi[1] - posj[1], posi[2] - posj[2]];
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
    fn set_num_types(&mut self, num_types: usize) -> Result<(), Error> {
        if self.num_types == num_types {
            return Ok(());
        }
        let new_len = (num_types * num_types) as usize;
        if self.num_types == 0 {
            self.num_types = num_types;
            self.coeff_set.resize(new_len, false);
            self.coeffs.resize(new_len, self.default_coeff());
            return Ok(());
        }

        // Get currently set indices
        let mut set_indices: Vec<[usize; 2]> = self
            .coeff_set
            .iter()
            .enumerate()
            .filter_map(|(n, set)| {
                if !set {
                    return None;
                }
                let i = n / self.num_types as usize;
                let j = n % self.num_types as usize;
                if i >= num_types || j >= num_types {
                    return None;
                }
                Some([i, j])
            })
            .collect();
        set_indices.sort_by(|a, b| {
            if a[0] == b[0] {
                a[1].cmp(&b[1])
            } else {
                a[0].cmp(&b[0])
            }
        });

        if self.num_types < num_types {
            // Adding more types: Resize first, then shift coeffs
            self.coeffs.resize(new_len, self.default_coeff());
            self.coeff_set.resize(new_len, false);
            for [i, j] in set_indices.iter().rev() {
                let old_idx = i * self.num_types + j;
                let new_idx = i * num_types + j;
                self.coeffs.swap(old_idx, new_idx);
                self.coeff_set.swap(old_idx, new_idx);
            }
        } else {
            // Removing types: shift coeffs first, then resize
            for [i, j] in set_indices.iter().rev() {
                let old_idx = i * self.num_types + j;
                let new_idx = i * num_types + j;
                self.coeffs.swap(old_idx, new_idx);
                self.coeff_set.swap(old_idx, new_idx);
            }
            self.coeffs.resize(new_len, self.default_coeff());
            self.coeff_set.resize(new_len, false);
        }

        Ok(())
    }
}
