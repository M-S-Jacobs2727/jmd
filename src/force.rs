mod ljcut;

use crate::{Atom, Domain};

pub trait Force {
    fn apply(&self, atoms: &mut Vec<Atom>, domain: &Domain);
    fn energy(&self, atoms: &Vec<Atom>, domain: &Domain) -> f64;
}

pub use ljcut::{LJCut, LJCutCoeff};
