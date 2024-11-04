use super::*;
use crate::{
    lattice::Lattice,
    utils::{Axis, Direction},
};

use rand;
#[derive(Clone, Copy, Debug)]
/// Rectangular prism region
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
        assert!(
            xlo < xhi && ylo < yhi && zlo < zhi,
            "Low corner ({}, {}, {}) should be less than high corner ({}, {}, {})",
            xlo,
            ylo,
            zlo,
            xhi,
            yhi,
            zhi
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
    pub fn from_lattice(lattice: &impl Lattice, num_cells: [usize; 3]) -> Self {
        let lengths = lattice.cell_lengths();
        Self {
            xlo: -0.5 * (num_cells[0] as f64) * lengths[0],
            xhi: 0.5 * (num_cells[0] as f64) * lengths[0],
            ylo: -0.5 * (num_cells[1] as f64) * lengths[1],
            yhi: 0.5 * (num_cells[1] as f64) * lengths[1],
            zlo: -0.5 * (num_cells[2] as f64) * lengths[2],
            zhi: 0.5 * (num_cells[2] as f64) * lengths[2],
        }
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
    pub fn lx(&self) -> f64 {
        self.xhi - self.xlo
    }
    pub fn ly(&self) -> f64 {
        self.yhi - self.ylo
    }
    pub fn lz(&self) -> f64 {
        self.zhi - self.zlo
    }
    pub fn lo(&self) -> [f64; 3] {
        [self.xlo, self.ylo, self.zlo]
    }
    pub fn hi(&self) -> [f64; 3] {
        [self.xhi, self.yhi, self.zhi]
    }
    pub fn lengths(&self) -> [f64; 3] {
        [self.lx(), self.ly(), self.lz()]
    }

    // Generic getters
    pub fn get_length(&self, axis: Axis) -> f64 {
        self.lengths()[axis.index()]
    }
    pub fn get_bound(&self, direction: Direction) -> f64 {
        match direction {
            Direction::Xlo => self.xlo,
            Direction::Xhi => self.xhi,
            Direction::Ylo => self.ylo,
            Direction::Yhi => self.yhi,
            Direction::Zlo => self.zlo,
            Direction::Zhi => self.zhi,
        }
    }
    pub fn get_bounds(&self, axis: Axis) -> [f64; 2] {
        match axis {
            Axis::X => [self.xlo, self.xhi],
            Axis::Y => [self.ylo, self.yhi],
            Axis::Z => [self.zlo, self.zhi],
        }
    }

    pub(crate) fn set_bound(&mut self, direction: Direction, bound: f64) {
        match direction {
            Direction::Xlo => self.xlo = bound,
            Direction::Xhi => self.xhi = bound,
            Direction::Ylo => self.ylo = bound,
            Direction::Yhi => self.yhi = bound,
            Direction::Zlo => self.zlo = bound,
            Direction::Zhi => self.zhi = bound,
        };
    }
    pub fn intersect(&self, other: &Self) -> Self {
        Self {
            xlo: self.xlo.min(other.xlo),
            xhi: self.xhi.min(other.xhi),
            ylo: self.ylo.min(other.ylo),
            yhi: self.yhi.min(other.yhi),
            zlo: self.zlo.min(other.zlo),
            zhi: self.zhi.min(other.zhi),
        }
    }
}
impl Region for Rect {
    fn contains(&self, coord: &[f64; 3]) -> bool {
        self.xlo <= coord[0]
            && coord[0] < self.xhi
            && self.ylo <= coord[1]
            && coord[1] < self.yhi
            && self.zlo <= coord[2]
            && coord[2] < self.zhi
    }
    fn get_random_coord(&self) -> [f64; 3] {
        [
            rand::random::<f64>() * self.lx() + self.xlo,
            rand::random::<f64>() * self.ly() + self.ylo,
            rand::random::<f64>() * self.lz() + self.zlo,
        ]
    }
    fn bounding_box(&self) -> Rect {
        self.clone()
    }
    fn volume(&self) -> f64 {
        let l = self.lengths();
        l[0] * l[1] * l[2]
    }
    fn surface_area(&self) -> f64 {
        let l = self.lengths();
        2.0 * (l[0] * l[1] + l[0] * l[2] + l[1] * l[2])
    }
}

#[cfg(test)]
mod tests {
    use super::Rect;
    #[test]
    fn test_bounds() {
        let r = Rect::new(0.0, 1.0, 2.0, 3.0, 4.0, 5.0);
        assert_eq!(r.xlo(), 0.0);
        assert_eq!(r.xhi(), 1.0);
        assert_eq!(r.ylo(), 2.0);
        assert_eq!(r.yhi(), 3.0);
        assert_eq!(r.zlo(), 4.0);
        assert_eq!(r.zhi(), 5.0);

        assert_eq!(r.lx(), 1.0);
        assert_eq!(r.ly(), 1.0);
        assert_eq!(r.lz(), 1.0);

        assert_eq!(r.lo(), [0.0, 2.0, 4.0]);
        assert_eq!(r.hi(), [1.0, 3.0, 5.0]);
    }
}
