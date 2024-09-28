use crate::box_::Box_;

use super::Region;
use rand;

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
    pub fn from_box(box_: &Box_) -> Self {
        Self {
            xlo: box_.xlo(),
            xhi: box_.xhi(),
            ylo: box_.ylo(),
            yhi: box_.yhi(),
            zlo: box_.zlo(),
            zhi: box_.zhi(),
        }
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
    fn add_random_atoms(
        &self,
        sim: &mut super::Simulation,
        num_atoms: usize,
        atom_type: u32,
        mass: f64,
    ) {
        let atom_id = sim.ids().iter().max().unwrap_or(&0) + 1;
        sim.ids.extend(atom_id..atom_id + num_atoms);
        sim.types.reserve(num_atoms);
        sim.positions.reserve(num_atoms);
        sim.velocities.reserve(num_atoms);
        sim.masses.reserve(num_atoms);

        for _i in 0..num_atoms {
            sim.types.push(atom_type);
            sim.masses.push(mass);
            sim.velocities.push([0.0, 0.0, 0.0]);
            sim.positions.push([
                rand::random::<f64>() * sim.box_.lx() + sim.box_.xlo(),
                rand::random::<f64>() * sim.box_.ly() + sim.box_.ylo(),
                rand::random::<f64>() * sim.box_.lz() + sim.box_.zlo(),
            ])
        }
    }
}