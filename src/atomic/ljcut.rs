use super::*;

#[derive(Clone, Copy, Debug)]
pub struct LJCutCoeff {
    sigma: f64,
    epsilon: f64,
    rcut: f64,
    sigma6: f64,
    rcut2: f64,
    prefactor: f64,  // = -24 epsilon * sigma^6
    correction: f64, // currently, only shift is supported
}
impl LJCutCoeff {
    pub fn new(sigma: f64, epsilon: f64, rcut: f64) -> Self {
        let sigma6 = sigma * sigma * sigma * sigma * sigma * sigma;
        let rcut2 = rcut * rcut;
        let rcut6 = rcut2 * rcut2 * rcut2;
        let correction = 2.0 * epsilon * sigma6 / rcut6 * (sigma6 / rcut6 - 1.0);
        Self {
            sigma,
            epsilon,
            rcut,
            rcut2: rcut * rcut,
            sigma6,
            prefactor: -24.0 * epsilon * sigma6,
            correction,
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
    pub fn new(force_cutoff: f64) -> Self {
        assert!(
            force_cutoff > 0.0,
            "Force cutoff should be positive, found {}",
            force_cutoff
        );
        Self {
            num_types: 0,
            force_cutoff,
            coeffs: Vec::new(),
            coeff_set: Vec::new(),
        }
    }
    pub fn set_global_cutoff(&mut self, cutoff: f64) {
        assert!(
            self.coeffs.iter().zip(self.coeff_set.iter()).all(|(c, set)| !set || c.rcut <= cutoff),
            "Global cutoff distance {} should be greater than the maximum cutoff distance in the coefficients",
            cutoff
        );
        self.force_cutoff = cutoff;
    }
    fn default_coeff() -> LJCutCoeff {
        LJCutCoeff::new(0.0, 0.0, 0.0)
    }
}

impl<T: AtomType> AtomicPotentialTrait<T> for LJCut {
    type Coeff = LJCutCoeff;
    fn new() -> Self {
        Self::new(1.0)
    }
    fn cutoff_distance(&self) -> f64 {
        self.force_cutoff
    }
    // TODO: check that forces are not double counted
    // should be newton-pair full, not half, because half neighbor list
    fn compute_forces(&self, atoms: &Atoms<T>, neighbor_list: &NeighborList) -> Vec<[f64; 3]> {
        let mut forces: Vec<[f64; 3]> = Vec::new();
        forces.resize(atoms.num_total_atoms(), [0.0, 0.0, 0.0]);
        for i in 0..atoms.nlocal {
            let typei = &atoms.types[i];
            let posi = &atoms.positions[i];

            for j in &neighbor_list.neighbors()[i] {
                if i == *j {
                    continue;
                }
                // U(r) = 4 eps ((sig/r)^12 - (sig/r)^6) - const
                // f(r) = dU/dr = dU/d(r^2) d(r^2)/dr
                // f(r) = -24 r eps / r^2 (2(sig/r)^12 - (sig/r)^6)

                // If r_i = (0, 0) and r_j = (sig, 0), then the
                // force should be repulsive, ie., f_i ~ (-1, 0), f_j ~ (1, 0)
                // f(r_ij) = r_ij * -24 eps / sig^2, so if r_ij = r_j - r_i = (sig, 0),
                // then f_i = f(r_ij) and f_j = -f(r_ij)

                let typej = &atoms.types[*j];
                let posj = &atoms.positions[*j];

                let idx = <Self as AtomicPotentialTrait<T>>::type_idx(&self, *typei, *typej);
                let coeff = self.coeffs[idx];
                let r = [posi[0] - posj[0], posi[1] - posj[1], posi[2] - posj[2]];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];

                if r2 > coeff.rcut2 {
                    continue;
                }

                let r6 = r2 * r2 * r2;

                let f_mag = coeff.prefactor / r6 / r2 * (2.0 * coeff.sigma6 / r6 - 1.0);
                forces[i] = [r[0] * f_mag, r[1] * f_mag, r[2] * f_mag];
                forces[*j] = [-r[0] * f_mag, -r[1] * f_mag, -r[2] * f_mag];
            }
        }

        forces
    }
    fn compute_potential_energy(&self, atoms: &Atoms<T>, neighbor_list: &NeighborList) -> f64 {
        let mut energy = 0.0;

        for i in 0..atoms.nlocal {
            let typei = &atoms.types[i];
            let posi = &atoms.positions[i];

            for j in &neighbor_list.neighbors()[i] {
                if i == *j {
                    continue;
                }
                // U(r) = 4 eps ((sig/r)^12 - (sig/r)^6) - const
                // f(r) = dU/dr = dU/d(r^2) d(r^2)/dr
                // f(r) = -24 r eps / r^2 (2(sig/r)^12 - (sig/r)^6)

                // If r_i = (0, 0) and r_j = (sig, 0), then the
                // force should be repulsive, ie., f_i ~ (-1, 0), f_j ~ (1, 0)
                // f(r_ij) = r_ij * -24 eps / sig^2, so if r_ij = r_j - r_i = (sig, 0),
                // then f_i = f(r_ij) and f_j = -f(r_ij)

                let typej = &atoms.types[*j];
                let posj = &atoms.positions[*j];

                let idx = <Self as AtomicPotentialTrait<T>>::type_idx(&self, *typei, *typej);
                let coeff = self.coeffs[idx];
                let r = [posi[0] - posj[0], posi[1] - posj[1], posi[2] - posj[2]];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];

                if r2 > coeff.rcut2 {
                    continue;
                }

                let r6 = r2 * r2 * r2;

                energy += 4.0 * coeff.epsilon * coeff.sigma6 / r6 * (coeff.sigma6 / r6 - 1.0)
                    - coeff.correction;
            }
        }
        energy
    }
    fn num_types(&self) -> usize {
        self.num_types
    }
    fn set_num_types(&mut self, num_types: usize) {
        if self.num_types == num_types {
            return;
        }
        let new_len = num_types * num_types;
        if self.num_types == 0 {
            self.num_types = num_types;
            self.coeff_set.resize(new_len, false);
            self.coeffs.resize(new_len, LJCut::default_coeff());
            return;
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
                let i = n / self.num_types;
                let j = n % self.num_types;
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
            self.coeffs.resize(new_len, LJCut::default_coeff());
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
            self.coeffs.resize(new_len, LJCut::default_coeff());
            self.coeff_set.resize(new_len, false);
        }
    }
    fn all_set(&self) -> bool {
        self.coeff_set.iter().all(|&x| x)
    }
    fn set_coeff(&mut self, typei: usize, typej: usize, coeff: &Self::Coeff) {
        assert!(
            typei < self.num_types && typej < self.num_types,
            "Type indices should be less than the number of types (0-indexed)"
        );

        let index = <Self as AtomicPotentialTrait<T>>::type_idx(self, typei, typej);
        self.coeff_set[index] = true;
        self.coeffs[index] = coeff.clone();
    }
}
