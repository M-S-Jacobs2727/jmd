use crate::Atom;

pub struct PeriodicBoundary {
    x: Option<(f64, f64)>,
    y: Option<(f64, f64)>,
    z: Option<(f64, f64)>,
}

impl PeriodicBoundary {
    pub fn new(x: Option<(f64, f64)>, y: Option<(f64, f64)>, z: Option<(f64, f64)>) -> Self {
        if let Some((xlo, xhi)) = x {
            if xlo >= xhi {
                panic!("xlo must be less than xhi");
            }
        }
        if let Some((ylo, yhi)) = y {
            if ylo >= yhi {
                panic!("ylo must be less than yhi");
            }
        }
        if let Some((zlo, zhi)) = z {
            if zlo >= zhi {
                panic!("zlo must be less than zhi");
            }
        }
        Self { x, y, z }
    }
    pub fn x(&self) -> Option<(f64, f64)> {
        self.x
    }
    pub fn y(&self) -> Option<(f64, f64)> {
        self.y
    }
    pub fn z(&self) -> Option<(f64, f64)> {
        self.z
    }
    pub fn apply(&self, atoms: &mut Vec<Atom>) {
        let lx = if let Some((xlo, xhi)) = self.x {
            xhi - xlo
        } else {
            0.0
        };
        let ly = if let Some((ylo, yhi)) = self.y {
            yhi - ylo
        } else {
            0.0
        };
        let lz = if let Some((zlo, zhi)) = self.z {
            zhi - zlo
        } else {
            0.0
        };
        for atom in atoms.iter_mut() {
            if let Some((xlo, xhi)) = self.x {
                if atom.position[0] < xlo {
                    atom.position[0] += lx;
                } else if atom.position[0] > xhi {
                    atom.position[0] -= lx;
                }
            }
            if let Some((ylo, yhi)) = self.y {
                if atom.position[1] < ylo {
                    atom.position[1] += ly;
                } else if atom.position[1] > yhi {
                    atom.position[1] -= ly;
                }
            }
            if let Some((zlo, zhi)) = self.z {
                if atom.position[2] < zlo {
                    atom.position[2] += lz;
                } else if atom.position[2] > zhi {
                    atom.position[2] -= lz;
                }
            }
        }
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
        let periodic_boundary = PeriodicBoundary::new(Some((0.0, 1.0)), None, None);
        periodic_boundary.apply(&mut atoms);
        assert_approx_eq!(f64, atoms[0].position[0], 0.0, epsilon = 1e-10);
        assert_approx_eq!(f64, atoms[1].position[0], 0.1, epsilon = 1e-10);
    }
}
