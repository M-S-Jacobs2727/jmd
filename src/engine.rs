use crate::atom::Atom;
use crate::domain::Domain;
use crate::force::Force;
use crate::neighbor::NeighborList;

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
    pub fn add_force(&mut self, force: Box<dyn Force>) {
        self.forces.push(force);
    }
    pub fn add_atoms_at_coordinates(
        &mut self,
        positions: Vec<[f64; 3]>,
        mass: f64,
        atom_type: i64,
    ) {
        let max_id = self.atoms.iter().map(|atom| atom.id).max().unwrap_or(0);
        self.atoms.reserve(positions.len());
        for (i, &position) in positions.iter().enumerate() {
            self.atoms.push(Atom {
                id: max_id + i,
                position,
                velocity: [0.0, 0.0, 0.0],
                force: [0.0, 0.0, 0.0],
                mass,
                atom_type,
            });
        }
    }
    pub fn run(&mut self, steps: usize, neighbor_list: &mut NeighborList) {
        neighbor_list.generate(&self.atoms);
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
            force.apply(&mut self.atoms, &mut self.domain);
        }
    }
    fn output(&self) {}
}
