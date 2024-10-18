use crate::{region::Rect, Axis};

/// Boundary conditions for simulation box.
///
/// P: Periodic (must be set for both sides)
/// F: Fixed boundary
/// S: Shrink-wrapped boundary
/// M: Shrink-wrapped boundary with a minimum
pub enum BC {
    PP,
    FF,
    FM,
    FS,
    MF,
    MM,
    MS,
    SF,
    SM,
    SS,
}
impl BC {
    pub fn is_periodic(&self) -> bool {
        match self {
            BC::PP => true,
            _ => false,
        }
    }
}

/// Simulation box, represented by x, y, and z Bounds
pub struct Container {
    rect: Rect,
    bc: [BC; 3],
}

impl Container {
    pub fn new(
        xlo: f64,
        xhi: f64,
        ylo: f64,
        yhi: f64,
        zlo: f64,
        zhi: f64,
        xbc: BC,
        ybc: BC,
        zbc: BC,
    ) -> Self {
        let rect = Rect::new(xlo, xhi, ylo, yhi, zlo, zhi);
        Self {
            rect,
            bc: [xbc, ybc, zbc],
        }
    }
    pub fn from_rect(rect: Rect) -> Self {
        Self {
            rect,
            bc: [BC::PP, BC::PP, BC::PP],
        }
    }
    pub fn is_periodic(&self, axis: Axis) -> bool {
        self.bc[axis.index()].is_periodic()
    }
    pub fn rect(&self) -> &Rect {
        &self.rect
    }
    pub fn lx(&self) -> f64 {
        self.rect.lx()
    }
    pub fn ly(&self) -> f64 {
        self.rect.ly()
    }
    pub fn lz(&self) -> f64 {
        self.rect.lz()
    }
    pub fn lengths(&self) -> [f64; 3] {
        [self.lx(), self.ly(), self.lz()]
    }
    pub fn lo(&self) -> [f64; 3] {
        [self.xlo(), self.ylo(), self.zlo()]
    }
    pub fn hi(&self) -> [f64; 3] {
        [self.xhi(), self.yhi(), self.zhi()]
    }
    pub fn xlo(&self) -> f64 {
        self.rect.xlo()
    }
    pub fn xhi(&self) -> f64 {
        self.rect.xhi()
    }
    pub fn ylo(&self) -> f64 {
        self.rect.ylo()
    }
    pub fn yhi(&self) -> f64 {
        self.rect.yhi()
    }
    pub fn zlo(&self) -> f64 {
        self.rect.zlo()
    }
    pub fn zhi(&self) -> f64 {
        self.rect.zhi()
    }
}
