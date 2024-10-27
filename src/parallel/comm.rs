use crate::{parallel::message::AtomMessage, Direction, Rect, Region, Simulation};

pub(crate) fn reverse_comm(sim: &Simulation, forces: &mut Vec<[f64; 3]>) {
    let mut sent_ids: Vec<usize> = Vec::new();

    // z-direction
    let mut ids = send_reverse_comm(sim, forces, Direction::Zhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);

    let mut ids = send_reverse_comm(sim, forces, Direction::Zlo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);

    // y-direction
    let mut ids = send_reverse_comm(sim, forces, Direction::Yhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);

    let mut ids = send_reverse_comm(sim, forces, Direction::Ylo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);

    // x-direction
    let mut ids = send_reverse_comm(sim, forces, Direction::Xhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);

    let mut ids = send_reverse_comm(sim, forces, Direction::Xlo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim, forces);
}

pub(crate) fn forward_comm(sim: &mut Simulation) {
    // x-direction
    send_forward_comm(sim, Direction::Xlo);
    recv_forward_comm(sim);
    send_forward_comm(sim, Direction::Xhi);
    recv_forward_comm(sim);

    // y-direction
    send_forward_comm(sim, Direction::Ylo);
    recv_forward_comm(sim);
    send_forward_comm(sim, Direction::Yhi);
    recv_forward_comm(sim);

    // z-direction
    send_forward_comm(sim, Direction::Zlo);
    recv_forward_comm(sim);
    send_forward_comm(sim, Direction::Zhi);
    recv_forward_comm(sim);
}

fn collect_comm_atoms(sim: &Simulation, direction: &Direction) -> Vec<usize> {
    let idx = direction.axis().index();
    sim.atoms
        .positions
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            if direction.is_lo() && p[idx] < sim.domain().subdomain().lo()[idx] {
                Some(i)
            } else if !direction.is_lo() && p[idx] > sim.domain().subdomain().hi()[idx] {
                Some(i)
            } else {
                None
            }
        })
        .collect()
}

fn send_atoms(sim: &mut Simulation, direction: Direction) {
    let atom_idxs = collect_comm_atoms(sim, &direction);
    let ids: Vec<usize> = atom_idxs.iter().map(|i| sim.atoms.ids[*i]).collect();
    sim.domain()
        .send(AtomMessage::Idxs(ids), direction)
        .unwrap();

    sim.atoms.remove_idxs(atom_idxs);
}

fn recv_atoms(sim: &mut Simulation) {
    let msg = sim.domain().receive();
    match msg {
        AtomMessage::Idxs(new_ids) => {
            let opt = sim.atoms.ids.iter().position(|id| new_ids.contains(id));
            if let Some(idx) = opt {
                if sim.container().rect().contains(&sim.atoms.positions[idx]) {
                    sim.increment_nlocal();
                }
            }
        }
        _ => panic!("Invalid message"),
    };
}

pub(crate) fn comm_atom_ownership(sim: &mut Simulation) {
    // dbg!(&sim.atoms);
    send_atoms(sim, Direction::Xlo);
    // dbg!(&sim.atoms);
    recv_atoms(sim);

    send_atoms(sim, Direction::Xhi);
    recv_atoms(sim);

    send_atoms(sim, Direction::Ylo);
    recv_atoms(sim);

    send_atoms(sim, Direction::Yhi);
    recv_atoms(sim);

    send_atoms(sim, Direction::Zlo);
    recv_atoms(sim);

    send_atoms(sim, Direction::Zhi);
    recv_atoms(sim);
}

fn recv_reverse_comm(sim: &Simulation, forces: &mut Vec<[f64; 3]>) {
    let id_msg = sim.domain().receive();
    let force_msg = sim.domain().receive();
    match (id_msg, force_msg) {
        (AtomMessage::Idxs(ids), AtomMessage::Float3(new_forces)) => {
            accumulate_forces(sim, &ids, &new_forces, forces)
        }
        (AtomMessage::Float3(new_forces), AtomMessage::Idxs(ids)) => {
            accumulate_forces(sim, &ids, &new_forces, forces)
        }
        _ => panic!("Invalid communication"),
    }
}

fn recv_forward_comm(sim: &mut Simulation) {
    let id_msg = sim.domain().receive();
    let type_msg = sim.domain().receive();
    let mass_msg = sim.domain().receive();
    let pos_msg = sim.domain().receive();
    let vel_msg = sim.domain().receive();
    match (id_msg, type_msg, mass_msg, pos_msg, vel_msg) {
        (
            AtomMessage::Idxs(ids),
            AtomMessage::Types(types),
            AtomMessage::Float(masses),
            AtomMessage::Float3(positions),
            AtomMessage::Float3(velocities),
        ) => {
            update_ghost_atoms(sim, ids, types, masses, positions, velocities);
        }
        _ => panic!("Invalid message"),
    };
}

fn gather_ghost_ids(sim: &Simulation, rect: &Rect) -> Vec<usize> {
    sim.atoms
        .ids
        .iter()
        .skip(sim.nlocal())
        .zip(sim.atoms.positions.iter())
        .filter(|(_id, pos)| rect.contains(pos))
        .map(|(id, _pos)| *id)
        .collect()
}

fn gather_owned_idxs(sim: &Simulation, rect: &Rect) -> Vec<usize> {
    sim.atoms
        .positions
        .iter()
        .take(sim.nlocal())
        .enumerate()
        .filter(|(_id, pos)| rect.contains(pos))
        .map(|(id, _pos)| id)
        .collect()
}

fn accumulate_forces(
    sim: &Simulation,
    ids: &Vec<usize>,
    forces: &Vec<[f64; 3]>,
    cur_forces: &mut Vec<[f64; 3]>,
) {
    for i in 0..sim.atoms.num_atoms() {
        let opt = ids.iter().position(|id| *id == sim.atoms.ids[i]);
        if let Some(j) = opt {
            cur_forces[i][0] += forces[j][0];
            cur_forces[i][1] += forces[j][1];
            cur_forces[i][2] += forces[j][2];
        }
    }
}

fn send_reverse_comm(sim: &Simulation, forces: &Vec<[f64; 3]>, direction: Direction) -> Vec<usize> {
    let ids = gather_ghost_ids(
        sim,
        &sim.domain().get_outer_rect(&direction, sim.neighbor_list()),
    );
    let mut send_forces: Vec<[f64; 3]> = Vec::new();
    send_forces.reserve(ids.len());

    for id in &ids {
        let j = sim
            .atoms
            .ids
            .iter()
            .position(|i| *i == *id)
            .expect("Should exist");

        send_forces.push(forces[j]);
    }

    sim.domain()
        .send(AtomMessage::Idxs(ids.clone()), direction)
        .expect("Disconnect error");
    sim.domain()
        .send(AtomMessage::Float3(send_forces), direction)
        .expect("Disconnect error");
    ids
}

fn send_forward_comm(sim: &Simulation, direction: Direction) {
    let idxs = gather_owned_idxs(
        sim,
        &sim.domain().get_inner_rect(&direction, sim.neighbor_list()),
    );
    fn gather<T: Copy>(idxs: &Vec<usize>, vec: &Vec<T>) -> Vec<T> {
        idxs.iter().map(|i| vec[*i]).collect()
    }
    let send = |m| sim.domain().send(m, direction).expect("Disconnect error");

    let types: Vec<u32> = gather(&idxs, &sim.atoms.types);
    let masses: Vec<f64> = gather(&idxs, &sim.atoms.masses);
    let positions: Vec<[f64; 3]> = gather(&idxs, &sim.atoms.positions);
    let velocities: Vec<[f64; 3]> = gather(&idxs, &sim.atoms.velocities);

    send(AtomMessage::Idxs(idxs));
    send(AtomMessage::Types(types));
    send(AtomMessage::Float(masses));
    send(AtomMessage::Float3(positions));
    send(AtomMessage::Float3(velocities));
}

fn update_ghost_atoms(
    sim: &mut Simulation,
    mut ids: Vec<usize>,
    mut types: Vec<u32>,
    mut masses: Vec<f64>,
    mut positions: Vec<[f64; 3]>,
    mut velocities: Vec<[f64; 3]>,
) {
    assert_eq!(ids.len(), types.len(), "Invalid communication");
    assert_eq!(ids.len(), masses.len(), "Invalid communication");
    assert_eq!(ids.len(), positions.len(), "Invalid communication");
    assert_eq!(ids.len(), velocities.len(), "Invalid communication");

    sim.atoms.ids.resize(sim.atoms.nlocal, 0);
    sim.atoms.types.resize(sim.atoms.nlocal, 0);
    sim.atoms.masses.resize(sim.atoms.nlocal, 0.0);
    sim.atoms.positions.resize(sim.atoms.nlocal, [0.0; 3]);
    sim.atoms.velocities.resize(sim.atoms.nlocal, [0.0; 3]);

    sim.atoms.ids.append(&mut ids);
    sim.atoms.types.append(&mut types);
    sim.atoms.masses.append(&mut masses);
    sim.atoms.positions.append(&mut positions);
    sim.atoms.velocities.append(&mut velocities);
}
