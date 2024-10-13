// TODO: update comm, remove args from new, add init?
use super::{AtomInfo, Domain, NeighborDirection};
use crate::{
    container::BC,
    region::{Rect, Region},
    AtomicPotential, Atoms, Container, NeighborList,
};

pub struct Simulation<P: AtomicPotential> {
    pub atoms: Atoms,
    container: Container,
    atomic_potential: P,
    neighbor_list: NeighborList,
    domain: Domain,
    nlocal: usize,
    max_distance_sq: f64,
}

impl<P: AtomicPotential> Simulation<P> {
    pub fn new(domain: Domain) -> Self {
        let container = Container::new(0., 10., 0.0, 10.0, 0.0, 10.0, BC::PP, BC::PP, BC::PP);
        let neighbor_list = NeighborList::new(&container, 1.0, 1.0, 1.0);
        Self {
            atoms: Atoms::new(),
            container,
            atomic_potential: P::new(),
            neighbor_list,
            domain,
            nlocal: 0,
            max_distance_sq: 0.0,
        }
    }
    pub fn compute_forces(&self) -> Vec<[f64; 3]> {
        self.atomic_potential.compute_forces(&self.atoms)
    }

    pub fn check_build_neighbor_list(&mut self, step: &usize) {
        if !self
            .neighbor_list
            .update_settings
            .should_update_neighbors(*step)
        {
            return;
        }
        if self.neighbor_list.update_settings.check && !self.atoms_moved_too_far() {
            return;
        }
        self.build_neighbor_list();
    }

    pub fn build_neighbor_list(&mut self) {
        self.max_distance_sq = 0.0;
        self.neighbor_list.update(self.atoms.positions());
    }

    pub fn reverse_comm(&self, forces: &mut Vec<[f64; 3]>) {
        let mut sent_ids: Vec<usize> = Vec::new();

        // z-direction
        self.send_reverse_comm(forces, NeighborDirection::Zhi, &mut sent_ids);
        self.send_reverse_comm(forces, NeighborDirection::Zlo, &mut sent_ids);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }

        // y-direction
        self.send_reverse_comm(forces, NeighborDirection::Yhi, &mut sent_ids);
        self.send_reverse_comm(forces, NeighborDirection::Ylo, &mut sent_ids);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }

        // x-direction
        self.send_reverse_comm(forces, NeighborDirection::Xhi, &mut sent_ids);
        self.send_reverse_comm(forces, NeighborDirection::Xlo, &mut sent_ids);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.accumulate_forces(data, forces);
        }
    }

    pub fn forward_comm(&mut self) {
        // x-direction
        self.send_forward_comm(NeighborDirection::Xlo);
        self.send_forward_comm(NeighborDirection::Xhi);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }

        // y-direction
        self.send_forward_comm(NeighborDirection::Ylo);
        self.send_forward_comm(NeighborDirection::Yhi);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }

        // z-direction
        self.send_forward_comm(NeighborDirection::Zlo);
        self.send_forward_comm(NeighborDirection::Zhi);

        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }
        let result = self.domain.receive().expect("Disconnect error");
        if let Some(data) = result {
            self.update_ghost_atoms(data);
        }
    }

    fn atoms_moved_too_far(&mut self) -> bool {
        self.max_distance_sq
            > self.neighbor_list.skin_distance() * self.neighbor_list.skin_distance() * 0.25
    }
    pub fn update_max_distance_sq(&mut self, dist_sq: f64) {
        self.max_distance_sq = dist_sq.max(self.max_distance_sq);
    }

    fn gather_ghost_ids(&self, rect: Rect) -> Vec<usize> {
        self.atoms
            .ids
            .iter()
            .skip(self.nlocal)
            .zip(self.atoms.positions.iter())
            .filter(|(_id, pos)| rect.contains(pos))
            .map(|(id, _pos)| *id)
            .collect()
    }

    fn gather_owned_ids(&self, rect: Rect) -> Vec<usize> {
        self.atoms
            .ids
            .iter()
            .take(self.nlocal)
            .zip(self.atoms.positions.iter())
            .filter(|(_id, pos)| rect.contains(pos))
            .map(|(id, _pos)| *id)
            .collect()
    }

    fn accumulate_forces(&self, data: AtomInfo, forces: &mut Vec<[f64; 3]>) {
        for i in 0..self.atoms.num_atoms() {
            match data.ids.iter().position(|id| *id == self.atoms.ids[i]) {
                Some(j) => {
                    forces[i][0] += data.data[3 * j];
                    forces[i][1] += data.data[3 * j + 1];
                    forces[i][2] += data.data[3 * j + 2];
                }
                None => (),
            }
        }
    }

    fn send_reverse_comm(
        &self,
        forces: &Vec<[f64; 3]>,
        direction: NeighborDirection,
        sent_ids: &mut Vec<usize>,
    ) {
        let mut atom_info = AtomInfo::new();
        let mut ids =
            self.gather_ghost_ids(self.domain.get_outer_rect(&direction, &self.neighbor_list));

        atom_info.ids.append(&mut ids);
        atom_info.data.reserve(atom_info.ids.len() * 3);

        for id in &atom_info.ids {
            let j = self
                .atoms
                .ids
                .iter()
                .position(|i| *i == *id)
                .expect("Should exist");

            atom_info.data.push(forces[j][0]);
            atom_info.data.push(forces[j][1]);
            atom_info.data.push(forces[j][2]);
        }
        sent_ids.append(&mut atom_info.ids.clone());

        self.domain
            .send(atom_info, direction)
            .expect("Disconnect error");
    }

    fn send_forward_comm(&self, direction: NeighborDirection) {
        let mut atom_info = AtomInfo::new();
        let mut ids =
            self.gather_owned_ids(self.domain.get_inner_rect(&direction, &self.neighbor_list));

        atom_info.ids.append(&mut ids);
        atom_info.data.reserve(atom_info.ids.len() * 7);
        let indices: Vec<usize> = atom_info
            .ids
            .iter()
            .map(|id| {
                self.atoms
                    .ids
                    .iter()
                    .position(|i| *i == *id)
                    .expect("Should exist")
            })
            .collect();

        atom_info.types.reserve(atom_info.ids.len());
        for i in 0..indices.len() {
            atom_info.types.push(self.atoms.types[i]);
        }

        for i in 0..indices.len() {
            atom_info.data.push(self.atoms.masses[i]);
        }
        for i in 0..indices.len() {
            atom_info.data.push(self.atoms.positions[i][0]);
            atom_info.data.push(self.atoms.positions[i][1]);
            atom_info.data.push(self.atoms.positions[i][2]);
        }
        for i in 0..indices.len() {
            atom_info.data.push(self.atoms.velocities[i][0]);
            atom_info.data.push(self.atoms.velocities[i][1]);
            atom_info.data.push(self.atoms.velocities[i][2]);
        }

        self.domain
            .send(atom_info, direction)
            .expect("Disconnect error");
    }

    fn update_ghost_atoms(&mut self, data: AtomInfo) {
        let ncomm = data.ids.len();
        for i in 0..ncomm {
            let opt_j = self.atoms.ids.iter().position(|id| *id == data.ids[i]);
            match opt_j {
                Some(j) => {
                    self.atoms.types[j] = data.types[i];
                    self.atoms.masses[j] = data.data[i];
                    self.atoms.positions[j] = [
                        data.data[ncomm + 3 * i],
                        data.data[ncomm + 3 * i + 1],
                        data.data[ncomm + 3 * i + 2],
                    ];
                    self.atoms.velocities[j] = [
                        data.data[4 * ncomm + 3 * i],
                        data.data[4 * ncomm + 3 * i + 1],
                        data.data[4 * ncomm + 3 * i + 2],
                    ];
                }
                None => {
                    self.atoms.ids.push(data.ids[i]);
                    self.atoms.types.push(data.types[i]);
                    self.atoms.masses.push(data.data[i]);
                    self.atoms.positions.push([
                        data.data[ncomm + 3 * i],
                        data.data[ncomm + 3 * i + 1],
                        data.data[ncomm + 3 * i + 2],
                    ]);
                    self.atoms.velocities.push([
                        data.data[4 * ncomm + 3 * i],
                        data.data[4 * ncomm + 3 * i + 1],
                        data.data[4 * ncomm + 3 * i + 2],
                    ]);
                }
            }
        }
    }
}
