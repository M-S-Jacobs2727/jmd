use super::Region;
use crate::Container;

use rand;
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
        Self {
            xlo,
            xhi,
            ylo,
            yhi,
            zlo,
            zhi,
        }
    }
    pub fn from_box(container: &Container) -> Self {
        Self {
            xlo: container.xlo(),
            xhi: container.xhi(),
            ylo: container.ylo(),
            yhi: container.yhi(),
            zlo: container.zlo(),
            zhi: container.zhi(),
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
}
impl Region for Rect {
    fn contains(&self, coord: &[f64; 3]) -> bool {
        self.xlo <= coord[0]
            && coord[0] <= self.xhi
            && self.ylo <= coord[1]
            && coord[1] <= self.yhi
            && self.zlo <= coord[2]
            && coord[2] <= self.zhi
    }
    fn get_random_coord(&self) -> [f64; 3] {
        [
            rand::random::<f64>() * self.lx() + self.xlo,
            rand::random::<f64>() * self.ly() + self.ylo,
            rand::random::<f64>() * self.lz() + self.zlo,
        ]
    }
}
