use crate::Lattice;

pub trait Region {
    fn contains(&self, coord: [f64; 3]) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    xlo: f64,
    xhi: f64,
    ylo: f64,
    yhi: f64,
    zlo: f64,
    zhi: f64,
}

impl Rect {
    pub fn new(xlo: f64, xhi: f64, ylo: f64, yhi: f64, zlo: f64, zhi: f64) -> Self {
        if xlo >= xhi || ylo >= yhi || zlo >= zhi {
            panic!("Invalid bounds");
        }
        println!(
            "Created Rect from [{:.6}, {:.6}, {:.6}] to [{:.6}, {:.6}, {:.6}]",
            xlo, ylo, zlo, xhi, yhi, zhi
        );
        Self {
            xlo,
            xhi,
            ylo,
            yhi,
            zlo,
            zhi,
        }
    }
    /// Creates a Rect from a lattice and number of lattice boxes in each dimension.
    ///
    /// Assumes (xlo, ylo, zlo) = (0.0, 0.0, 0.0). For a different origin, use
    /// `from_lattice_with_origin`
    pub fn from_lattice(lattice: &Lattice, nx: u32, ny: u32, nz: u32) -> Self {
        Self::from_lattice_with_origin(&lattice, nx, ny, nz, [0.0, 0.0, 0.0])
    }
    pub fn from_lattice_with_origin(
        lattice: &Lattice,
        nx: u32,
        ny: u32,
        nz: u32,
        origin: [f64; 3],
    ) -> Self {
        let (xlo, ylo, zlo) = (origin[0], origin[1], origin[2]);
        let xhi = xlo + (nx as f64) * lattice.lx();
        let yhi = ylo + (ny as f64) * lattice.ly();
        let zhi = zlo + (nz as f64) * lattice.lz();
        Self::new(xlo, xhi, ylo, yhi, zlo, zhi)
    }
    pub fn set_bounds(&mut self, vec: Vec<(f64, f64)>) {
        self.xlo = vec[0].0;
        self.xhi = vec[0].1;
        self.ylo = vec[1].0;
        self.yhi = vec[1].1;
        self.zlo = vec[2].0;
        self.zhi = vec[2].1;
    }
    pub fn lx(&self) -> f64 {
        self.xhi - self.xlo
    }
    pub fn ly(&self) -> f64 {
        self.yhi - self.ylo
    }
    pub fn lz(&self) -> f64 {
        self.zhi - self.zlo
    }
    pub fn xlo(&self) -> f64 {
        self.xlo
    }
    pub fn xhi(&self) -> f64 {
        self.xhi
    }
    pub fn ylo(&self) -> f64 {
        self.ylo
    }
    pub fn yhi(&self) -> f64 {
        self.yhi
    }
    pub fn zlo(&self) -> f64 {
        self.zlo
    }
    pub fn zhi(&self) -> f64 {
        self.zhi
    }
}
impl Region for Rect {
    fn contains(&self, coord: [f64; 3]) -> bool {
        coord[0] >= self.xlo
            && coord[0] <= self.xhi
            && coord[1] >= self.ylo
            && coord[1] <= self.yhi
            && coord[2] >= self.zlo
            && coord[2] <= self.zhi
    }
}
