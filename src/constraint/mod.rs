use crate::{Atoms, Domain};

pub trait Constraint {
    fn pre_forward_communication(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
    fn pre_neighbor_build(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
    fn post_neighbor_build(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
    fn pre_compute(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
    fn pre_reverse_communication(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
    fn post_compute(&mut self, _atoms: &mut Atoms, _domain: &mut Domain) {}
}
