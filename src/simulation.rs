use crate::{
    parallel::{message::Message, Domain, Worker},
    region::{Rect, RegionTrait},
    AtomicPotential, AtomicPotentialTrait, Atoms, Container, Direction, Error, NeighborList, None_,
    UpdateSettings, BC,
};

pub struct Simulation {
    pub atoms: Atoms,
    pub container: Container,
    pub atomic_potential: AtomicPotential,
    pub neighbor_list: NeighborList,
    domain: Domain,
    nlocal: usize,
    max_distance_sq: f64,
    pos_at_prev_neigh_build: Vec<[f64; 3]>,
}

impl Simulation {
    pub fn new() -> Self {
        let container = Container::new(0., 10., 0.0, 10.0, 0.0, 10.0, BC::PP, BC::PP, BC::PP);
        let neighbor_list = NeighborList::new(&container, 1.0, 1.0, 1.0);
        Self {
            atoms: Atoms::new(),
            container,
            atomic_potential: None_::new().into(),
            neighbor_list,
            domain: Domain::new(),
            nlocal: 0,
            max_distance_sq: 0.0,
            pos_at_prev_neigh_build: Vec::new(),
        }
    }
    pub fn set_atom_types(&mut self, atom_types: usize) -> Result<(), Error> {
        self.atomic_potential.set_num_types(atom_types)
    }
    pub fn container(&self) -> &Container {
        &self.container
    }
    pub fn atomic_potential(&self) -> &crate::AtomicPotential {
        &self.atomic_potential
    }
    pub fn neighbor_list(&self) -> &NeighborList {
        &self.neighbor_list
    }
    pub fn domain(&self) -> &Domain {
        &self.domain
    }
    pub fn nlocal(&self) -> usize {
        self.nlocal
    }
    pub fn max_distance_sq(&self) -> f64 {
        self.max_distance_sq
    }
    pub fn set_container(&mut self, container: Container) {
        self.container = container;
        self.domain.reset_subdomain(&self.container);
    }
    pub fn set_atomic_potential(&mut self, atomic_potential: AtomicPotential) {
        self.atomic_potential = atomic_potential;
        let force_distance = self.atomic_potential.cutoff_distance();
        let skin_distance = 1.0;
        let bin_size = (force_distance + skin_distance) * 0.5;
        self.neighbor_list =
            NeighborList::new(self.container(), bin_size, force_distance, skin_distance);
    }
    pub fn set_neighbor_list(&mut self, neighbor_list: NeighborList) {
        self.neighbor_list = neighbor_list;
    }
    pub fn set_domain(&mut self, domain: Domain) {
        self.domain = domain;
    }
    pub fn set_neighbor_settings(&mut self, neighbor_settings: UpdateSettings) {
        self.neighbor_list.update_settings = neighbor_settings;
    }
    pub fn init(&mut self, worker: &Worker) {
        self.domain.init(&self.container, worker);
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
        dbg!(self.atoms.positions());
        self.wrap_pbs();
        self.comm_atom_ownership();
        self.neighbor_list.update(self.atoms.positions());
        self.pos_at_prev_neigh_build = self.atoms.positions.clone();
    }

    fn wrap_pbs(&mut self) {
        if self.container.is_periodic(Direction::Xlo) {
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[0] < self.container.xlo() {
                    p[0] += self.container.lx();
                } else if p[0] > self.container.xhi() {
                    p[0] -= self.container.lx();
                }
            });
        }
        if self.container.is_periodic(Direction::Ylo) {
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[1] < self.container.ylo() {
                    p[1] += self.container.ly();
                } else if p[1] > self.container.yhi() {
                    p[1] -= self.container.ly();
                }
            });
        }
        if self.container.is_periodic(Direction::Zlo) {
            self.atoms.positions.iter_mut().for_each(|p| {
                if p[2] < self.container.zlo() {
                    p[2] += self.container.lz();
                } else if p[2] > self.container.zhi() {
                    p[2] -= self.container.lz();
                }
            });
        }
    }

    fn collect_comm_atoms(&self, direction: Direction) -> Vec<usize> {
        let idx = direction.axis().index();
        self.atoms
            .positions
            .iter()
            .enumerate()
            .filter_map(|(i, p)| {
                if direction.is_lo() && p[idx] < self.domain.subdomain().lo()[idx] {
                    Some(i)
                } else if !direction.is_lo() && p[idx] > self.domain.subdomain().hi()[idx] {
                    Some(i)
                } else {
                    None
                }
            })
            .collect()
    }

    fn send_atoms(&mut self, direction: Direction) {
        let atom_idxs = self.collect_comm_atoms(direction.clone());
        let ids: Vec<usize> = atom_idxs.iter().map(|i| self.atoms.ids[*i]).collect();
        self.domain.send(Message::Idxs(ids), direction).unwrap();

        self.atoms.remove_idxs(atom_idxs);
    }

    fn recv_atoms(&mut self) {
        let msg = self.domain.receive().expect("Disconnect error");
        match msg {
            Message::Idxs(new_ids) => {
                let idx = self
                    .atoms
                    .ids
                    .iter()
                    .position(|id| new_ids.contains(id))
                    .expect("Missing atom");
                if self.container.rect().contains(&self.atoms.positions[idx]) {
                    self.nlocal += 1;
                }
            }
            _ => panic!("Invalid message"),
        };
    }

    fn comm_atom_ownership(&mut self) {
        self.send_atoms(Direction::Xlo);
        self.recv_atoms();

        self.send_atoms(Direction::Xhi);
        self.recv_atoms();

        self.send_atoms(Direction::Ylo);
        self.recv_atoms();

        self.send_atoms(Direction::Yhi);
        self.recv_atoms();

        self.send_atoms(Direction::Zlo);
        self.recv_atoms();

        self.send_atoms(Direction::Zhi);
        self.recv_atoms();
    }

    fn recv_reverse_comm(&self, forces: &mut Vec<[f64; 3]>) {
        let id_msg = self.domain.receive().expect("Disconnect error");
        let force_msg = self.domain.receive().expect("Disconnect error");
        match (id_msg, force_msg) {
            (Message::Idxs(ids), Message::Float3(new_forces)) => {
                self.accumulate_forces(&ids, &new_forces, forces)
            }
            (Message::Float3(new_forces), Message::Idxs(ids)) => {
                self.accumulate_forces(&ids, &new_forces, forces)
            }
            _ => panic!("Invalid communication"),
        }
    }

    pub fn reverse_comm(&self, forces: &mut Vec<[f64; 3]>) {
        let mut sent_ids: Vec<usize> = Vec::new();

        // z-direction
        let mut ids = self.send_reverse_comm(forces, Direction::Zhi);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);

        let mut ids = self.send_reverse_comm(forces, Direction::Zlo);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);

        // y-direction
        let mut ids = self.send_reverse_comm(forces, Direction::Yhi);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);

        let mut ids = self.send_reverse_comm(forces, Direction::Ylo);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);

        // x-direction
        let mut ids = self.send_reverse_comm(forces, Direction::Xhi);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);

        let mut ids = self.send_reverse_comm(forces, Direction::Xlo);
        sent_ids.append(&mut ids);
        self.recv_reverse_comm(forces);
    }

    fn recv_forward_comm(&mut self) {
        let id_msg = self.domain.receive().expect("Disconnect error");
        let type_msg = self.domain.receive().expect("Disconnect error");
        let mass_msg = self.domain.receive().expect("Disconnect error");
        let pos_msg = self.domain.receive().expect("Disconnect error");
        let vel_msg = self.domain.receive().expect("Disconnect error");
        match (id_msg, type_msg, mass_msg, pos_msg, vel_msg) {
            (
                Message::Idxs(ids),
                Message::Types(types),
                Message::Float(masses),
                Message::Float3(positions),
                Message::Float3(velocities),
            ) => {
                self.update_ghost_atoms(ids, types, masses, positions, velocities);
            }
            _ => panic!("Invalid message"),
        };
    }

    pub fn forward_comm(&mut self) {
        // x-direction
        self.send_forward_comm(Direction::Xlo);
        self.recv_forward_comm();
        self.send_forward_comm(Direction::Xhi);
        self.recv_forward_comm();

        // y-direction
        self.send_forward_comm(Direction::Ylo);
        self.recv_forward_comm();
        self.send_forward_comm(Direction::Yhi);
        self.recv_forward_comm();

        // z-direction
        self.send_forward_comm(Direction::Zlo);
        self.recv_forward_comm();
        self.send_forward_comm(Direction::Zhi);
        self.recv_forward_comm();
    }

    fn atoms_moved_too_far(&mut self) -> bool {
        if self.atoms.num_atoms() == 0 {
            return false;
        }
        let half_skin_dist = self.neighbor_list.skin_distance() * 0.5;
        let max_dist_sq = self
            .pos_at_prev_neigh_build
            .iter()
            .zip(self.atoms.positions().iter())
            .map(|(old, new)| {
                let [dx, dy, dz] = [new[0] - old[0], new[1] - old[1], new[2] - old[2]];
                dx * dx + dy * dy + dz * dz
            })
            .reduce(f64::max)
            .unwrap();

        max_dist_sq > half_skin_dist * half_skin_dist
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

    fn gather_owned_idxs(&self, rect: Rect) -> Vec<usize> {
        self.atoms
            .positions
            .iter()
            .take(self.nlocal)
            .enumerate()
            .filter(|(_id, pos)| rect.contains(pos))
            .map(|(id, _pos)| id)
            .collect()
    }

    fn accumulate_forces(
        &self,
        ids: &Vec<usize>,
        forces: &Vec<[f64; 3]>,
        cur_forces: &mut Vec<[f64; 3]>,
    ) {
        for i in 0..self.atoms.num_atoms() {
            let opt = ids.iter().position(|id| *id == self.atoms.ids[i]);
            if let Some(j) = opt {
                cur_forces[i][0] += forces[j][0];
                cur_forces[i][1] += forces[j][1];
                cur_forces[i][2] += forces[j][2];
            }
        }
    }

    fn send_reverse_comm(&self, forces: &Vec<[f64; 3]>, direction: Direction) -> Vec<usize> {
        let ids =
            self.gather_ghost_ids(self.domain.get_outer_rect(&direction, &self.neighbor_list));
        let mut send_forces: Vec<[f64; 3]> = Vec::new();
        send_forces.reserve(ids.len());

        for id in &ids {
            let j = self
                .atoms
                .ids
                .iter()
                .position(|i| *i == *id)
                .expect("Should exist");

            send_forces.push(forces[j]);
        }

        self.domain
            .send(Message::Idxs(ids.clone()), direction)
            .expect("Disconnect error");
        self.domain
            .send(Message::Float3(send_forces), direction)
            .expect("Disconnect error");
        ids
    }

    fn send_forward_comm(&self, direction: Direction) {
        let idxs =
            self.gather_owned_idxs(self.domain.get_inner_rect(&direction, &self.neighbor_list));

        let types: Vec<u32> = idxs.iter().map(|i| self.atoms.types[*i]).collect();
        let masses: Vec<f64> = idxs.iter().map(|i| self.atoms.masses[*i]).collect();
        let positions: Vec<[f64; 3]> = idxs.iter().map(|i| self.atoms.positions[*i]).collect();
        let velocities: Vec<[f64; 3]> = idxs.iter().map(|i| self.atoms.velocities[*i]).collect();

        self.domain
            .send(Message::Idxs(idxs), direction)
            .expect("Disconnect error");
        self.domain
            .send(Message::Types(types), direction)
            .expect("Disconnect error");
        self.domain
            .send(Message::Float(masses), direction)
            .expect("Disconnect error");
        self.domain
            .send(Message::Float3(positions), direction)
            .expect("Disconnect error");
        self.domain
            .send(Message::Float3(velocities), direction)
            .expect("Disconnect error");
    }

    fn update_ghost_atoms(
        &mut self,
        ids: Vec<usize>,
        types: Vec<u32>,
        masses: Vec<f64>,
        positions: Vec<[f64; 3]>,
        velocities: Vec<[f64; 3]>,
    ) {
        let ncomm = ids.len();
        for i in 0..ncomm {
            let opt_j = self.atoms.ids.iter().position(|id| *id == ids[i]);
            match opt_j {
                Some(j) => {
                    self.atoms.types[j] = types[i];
                    self.atoms.masses[j] = masses[i];
                    self.atoms.positions[j] = positions[i];
                    self.atoms.velocities[j] = velocities[i];
                }
                None => {
                    self.atoms.ids.push(ids[i]);
                    self.atoms.types.push(types[i]);
                    self.atoms.masses.push(masses[i]);
                    self.atoms.positions.push(positions[i]);
                    self.atoms.velocities.push(velocities[i]);
                }
            }
        }
    }
}
