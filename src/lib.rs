mod atom;
mod domain;
mod force;

pub use atom::Atom;
pub use domain::Domain;
pub use force::{Force, LJCut, LJCutCoeff};

pub struct Engine {
    pub domain: Domain,
    pub atoms: Vec<Atom>,
    pub forces: Vec<Box<dyn Force>>,
    pub dt: f64,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            domain: Domain::new(),
            atoms: Vec::new(),
            forces: Vec::new(),
            dt: 0.0,
        }
    }
    pub fn add_atom(&mut self, atom: Atom) {
        self.atoms.push(atom);
    }
    pub fn add_force(&mut self, force: Box<dyn Force>) {
        self.forces.push(force);
    }
    pub fn run(&mut self, steps: usize) {
        for _ in 0..steps {
            self.compute_half_step_velocity();
            self.compute_position();
            self.compute_force();
            self.compute_half_step_velocity();
            self.output();
        }
    }
    fn compute_half_step_velocity(&mut self) {
        for atom in self.atoms.iter_mut() {
            atom.velocity[0] += atom.force[0] * 0.5 * self.dt;
            atom.velocity[1] += atom.force[1] * 0.5 * self.dt;
            atom.velocity[2] += atom.force[2] * 0.5 * self.dt;
        }
    }
    fn compute_position(&mut self) {
        for atom in self.atoms.iter_mut() {
            atom.position[0] += atom.velocity[0] * self.dt;
            atom.position[1] += atom.velocity[1] * self.dt;
            atom.position[2] += atom.velocity[2] * self.dt;
        }
    }
    fn compute_force(&mut self) {
        for force in &mut self.forces {
            force.apply(&mut self.atoms, &self.domain);
        }
    }
    fn output(&self) {}
}
