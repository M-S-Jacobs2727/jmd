use crate::{Atoms, Domain};

use super::Pairwise;

#[derive(Clone, Debug)]
pub struct LJCut {
    pub force_cutoff: f64,
    pub coeffs: Vec<LJCutCoeff>,
}

#[derive(Clone, Debug)]
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
    pub fn energy(&self, atoms: &Atoms, _domain: &Domain) -> f64 {
        let mut energy = 0.0;
        for i in 0..atoms.len() {
            let type_i = atoms.atom_types[i];
            for j in i + 1..atoms.len() {
                let type_j = atoms.atom_types[j];
                let r = [
                    atoms.positions[i][0] - atoms.positions[j][0],
                    atoms.positions[i][1] - atoms.positions[j][1],
                    atoms.positions[i][2] - atoms.positions[j][2],
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

impl Pairwise for LJCut {
    fn compute_pairwise(&self, atoms: &mut Atoms) {
        for i in 0..atoms.len() {
            let type_i = atoms.atom_types[i];

            for j in i + 1..atoms.len() {
                let type_j = atoms.atom_types[j];
                let r = [
                    atoms.positions[i][0] - atoms.positions[j][0],
                    atoms.positions[i][1] - atoms.positions[j][1],
                    atoms.positions[i][2] - atoms.positions[j][2],
                ];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
                if let Some(coeff) = self.get_coeff(type_i, type_j) {
                    if r2 > coeff.cutoff_sq {
                        continue;
                    }
                    let r6 = r2 * r2 * r2;
                    let force_div_r =
                        6.0 * coeff.four_epsilon_sigma6 * (2.0 * coeff.sigma6 / r6 - 1.0) / r2 / r6;
                    atoms.forces[i][0] += force_div_r * r[0];
                    atoms.forces[i][1] += force_div_r * r[1];
                    atoms.forces[i][2] += force_div_r * r[2];
                    atoms.forces[j][0] -= force_div_r * r[0];
                    atoms.forces[j][1] -= force_div_r * r[1];
                    atoms.forces[j][2] -= force_div_r * r[2];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use crate::{integrator::NVE, region::Rect, NeighborListSettings, SimulationBuilder};

    use float_cmp::assert_approx_eq;

    #[test]
    fn test_ljcut_one_atom() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0]];
        let mut sim = SimulationBuilder::new()
            .with_box(Rect::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
            .with_pbc(
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
            )
            .with_neighbor_list(NeighborListSettings::new(0.3, 2.5, 1, 0))
            .with_integrator(NVE::new(0.005))
            .build();
        sim.add_atoms_at_coordinates(positions, 1.0, 1);
        sim.add_pairwise(ljcut);
        sim.run(0);

        let atoms = sim.atoms();
        assert_approx_eq!(f64, atoms.forces[0][0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_ljcut_one_sigma() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut sim = SimulationBuilder::new()
            .with_box(Rect::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
            .with_pbc(
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
            )
            .with_neighbor_list(NeighborListSettings::new(0.3, 2.5, 1, 0))
            .with_integrator(NVE::new(0.005))
            .build();
        sim.add_atoms_at_coordinates(positions, 1.0, 1);
        sim.add_pairwise(ljcut);
        sim.run(0);

        let atoms = sim.atoms();
        assert_approx_eq!(f64, atoms.forces[0][0], -24.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][2], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][0], 24.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_ljcut_2v1_6_sigma() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0], [2.0_f64.powf(1.0 / 6.0), 0.0, 0.0]];
        let mut sim = SimulationBuilder::new()
            .with_box(Rect::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0))
            .with_pbc(
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
                BoundaryCondition::Periodic,
            )
            .with_neighbor_list(NeighborListSettings::new(0.3, 2.5, 1, 0))
            .with_integrator(NVE::new(0.005))
            .build();
        sim.add_atoms_at_coordinates(positions, 1.0, 1);
        sim.add_pairwise(ljcut);
        sim.run(0);

        let atoms = sim.atoms();
        assert_approx_eq!(f64, atoms.forces[0][0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[0][2], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms.forces[1][2], 0.0, epsilon = 1e-10);
    }
}
