use crate::{Atom, Domain, System};

use super::Constraint;

pub struct PeriodicBoundary {
    x: bool,
    y: bool,
    z: bool,
}

impl PeriodicBoundary {
    pub fn new(x: bool, y: bool, z: bool) -> Self {
        Self { x, y, z }
    }
    pub fn x(&self) -> bool {
        self.x
    }
    pub fn y(&self) -> bool {
        self.y
    }
    pub fn z(&self) -> bool {
        self.z
    }
    pub fn wrap_atoms_across_boundaries(&self, atoms: &mut Vec<Atom>, domain: &Domain) {
        let lx = if self.x {
            domain.xhi() - domain.xlo()
        } else {
            0.0
        };
        let ly = if self.y {
            domain.yhi() - domain.ylo()
        } else {
            0.0
        };
        let lz = if self.z {
            domain.zhi() - domain.zlo()
        } else {
            0.0
        };
        for atom in atoms.iter_mut() {
            if atom.position[0] < domain.xlo() {
                atom.position[0] += lx;
            } else if atom.position[0] > domain.xhi() {
                atom.position[0] -= lx;
            }
            if atom.position[1] < domain.ylo() {
                atom.position[1] += ly;
            } else if atom.position[1] > domain.yhi() {
                atom.position[1] -= ly;
            }
            if atom.position[2] < domain.zlo() {
                atom.position[2] += lz;
            } else if atom.position[2] > domain.zhi() {
                atom.position[2] -= lz;
            }
        }
    }
}

impl Constraint for PeriodicBoundary {
    fn post_forward_communication(&mut self, system: &mut System) {
        self.wrap_atoms_across_boundaries(&mut system.atoms, &system.domain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use float_cmp::assert_approx_eq;

    #[test]
    fn test_periodic_boundary() {
        let mut atoms = vec![
            Atom {
                position: [0.0, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
                force: [0.0, 0.0, 0.0],
                mass: 1.0,
                atom_type: 1,
                id: 0,
            },
            Atom {
                position: [1.1, 0.0, 0.0],
                velocity: [0.0, 0.0, 0.0],
                force: [0.0, 0.0, 0.0],
                mass: 1.0,
                atom_type: 1,
                id: 1,
            },
        ];
        let mut domain = Domain::new(0.0, 1.0, 0.0, 1.0, 0.0, 1.0);
        let periodic_boundary = PeriodicBoundary::new(true, false, false);
        periodic_boundary.wrap_atoms_across_boundaries(&mut atoms, &mut domain);
        assert_approx_eq!(f64, atoms[0].position[0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms[1].position[0], 0.1, epsilon = 1e-10);
    }
}
