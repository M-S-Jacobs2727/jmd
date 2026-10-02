#[derive(Copy, Clone, Debug)]
pub enum Lattice {
    Cubic(f64),
    Fcc(f64),
    Bcc(f64),
}

impl Lattice {
    pub fn lx(&self) -> f64 {
        match self {
            Lattice::Cubic(x) | Lattice::Bcc(x) | Lattice::Fcc(x) => *x,
        }
    }
    pub fn ly(&self) -> f64 {
        match self {
            Lattice::Cubic(x) | Lattice::Bcc(x) | Lattice::Fcc(x) => *x,
        }
    }
    pub fn lz(&self) -> f64 {
        match self {
            Lattice::Cubic(x) | Lattice::Bcc(x) | Lattice::Fcc(x) => *x,
        }
    }
    pub fn num_lattice_points(&self) -> u32 {
        match self {
            Lattice::Cubic(_) => 1,
            Lattice::Bcc(_) => 2,
            Lattice::Fcc(_) => 4,
        }
    }
    pub fn offsets(&self) -> Vec<[f64; 3]> {
        match self {
            Lattice::Cubic(_) => vec![[0.0, 0.0, 0.0]],
            Lattice::Bcc(x) => vec![[0.0, 0.0, 0.0], [*x * 0.5, *x * 0.5, *x * 0.5]],
            Lattice::Fcc(x) => vec![
                [0.0, 0.0, 0.0],
                [0.5 * *x, 0.5 * *x, 0.0],
                [0.5 * *x, 0.0, 0.5 * *x],
                [0.5 * *x, 0.0, 0.5 * *x],
            ],
        }
    }
}
