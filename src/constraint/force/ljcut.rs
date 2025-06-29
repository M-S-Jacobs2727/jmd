use crate::{Atom, Domain, System};

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
    fn compute_force(&mut self, system: &mut System) {
        system
            .atoms
            .iter_mut()
            .for_each(|atom| atom.force = [0.0; 3]);
        for i in 0..system.atoms.len() {
            let type_i = system.atoms[i].atom_type;

            for j in i + 1..system.atoms.len() {
                let type_j = system.atoms[j].atom_type;
                let r = [
                    system.atoms[i].position[0] - system.atoms[j].position[0],
                    system.atoms[i].position[1] - system.atoms[j].position[1],
                    system.atoms[i].position[2] - system.atoms[j].position[2],
                ];
                let r2 = r[0] * r[0] + r[1] * r[1] + r[2] * r[2];
                if let Some(coeff) = self.get_coeff(type_i, type_j) {
                    if r2 > coeff.cutoff_sq {
                        continue;
                    }
                    let r6 = r2 * r2 * r2;
                    let force_div_r =
                        6.0 * coeff.four_epsilon_sigma6 * (2.0 * coeff.sigma6 / r6 - 1.0) / r2 / r6;
                    system.atoms[i].force[0] += force_div_r * r[0];
                    system.atoms[i].force[1] += force_div_r * r[1];
                    system.atoms[i].force[2] += force_div_r * r[2];
                    system.atoms[j].force[0] -= force_div_r * r[0];
                    system.atoms[j].force[1] -= force_div_r * r[1];
                    system.atoms[j].force[2] -= force_div_r * r[2];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BoundaryCondition, NeighborListSettings, SimulationBox, SystemBuilder};

    use float_cmp::assert_approx_eq;

    #[test]
    fn test_ljcut_one_atom() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0]];
        let mut system = SystemBuilder::new()
            .with_simulation_box(SimulationBox::new(
                Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
                [
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                ],
            ))
            .with_neighbor_list_settings(NeighborListSettings::new(2.8, 2.5, 1, 0))
            .build();
        system.add_atoms_at_coordinates(positions, 1.0, 1);
        ljcut.compute_force(&mut system);
        assert_approx_eq!(f64, system.atoms[0].force[0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_ljcut_one_sigma() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let mut system = SystemBuilder::new()
            .with_simulation_box(SimulationBox::new(
                Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
                [
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                ],
            ))
            .with_neighbor_list_settings(NeighborListSettings::new(2.8, 2.5, 1, 0))
            .build();
        system.add_atoms_at_coordinates(positions, 1.0, 1);
        ljcut.compute_force(&mut system);
        assert_approx_eq!(f64, system.atoms[0].force[0], -24.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[2], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[0], 24.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[2], 0.0, epsilon = 1e-10);
    }

    #[test]
    fn test_ljcut_2v1_6_sigma() {
        let mut ljcut = LJCut::new(2.5);
        ljcut.add_coeff(LJCutCoeff::new(1, 1, 1.0, 1.0, 2.5));
        let positions = vec![[0.0, 0.0, 0.0], [2.0_f64.powf(1.0 / 6.0), 0.0, 0.0]];
        let mut system = SystemBuilder::new()
            .with_simulation_box(SimulationBox::new(
                Domain::new(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0),
                [
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                    BoundaryCondition::Periodic,
                ],
            ))
            .with_neighbor_list_settings(NeighborListSettings::new(2.8, 2.5, 1, 0))
            .build();
        system.add_atoms_at_coordinates(positions, 1.0, 1);
        ljcut.compute_force(&mut system);
        assert_approx_eq!(f64, system.atoms[0].force[0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[0].force[2], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[1], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, system.atoms[1].force[2], 0.0, epsilon = 1e-10);
    }
}
