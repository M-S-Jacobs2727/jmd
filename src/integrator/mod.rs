use super::{Atoms, Domain};

mod nve;

pub use nve::NVE;

pub trait Integrator {
    fn initial_integrate(&self, atoms: &mut Atoms, domain: &mut Domain);
    fn final_integrate(&self, atoms: &mut Atoms, domain: &mut Domain);
}
