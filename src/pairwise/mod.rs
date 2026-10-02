use crate::Atoms;

mod ljcut;

pub use ljcut::{LJCut, LJCutCoeff};

pub trait Pairwise {
    fn compute_pairwise(&self, atoms: &mut Atoms);
}
