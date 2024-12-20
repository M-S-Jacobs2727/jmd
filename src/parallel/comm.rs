use super::*;
use crate::{
    atom_type::AtomType,
    atomic::AtomicPotentialTrait,
    atoms::Atom,
    region::{Rect, Region},
    simulation::Simulation,
    utils::Direction,
};

pub(crate) fn reverse_comm<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let mut sent_ids: Vec<usize> = Vec::new();

    // z-direction
    let mut ids = send_reverse_comm(sim, Direction::Zhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);

    let mut ids = send_reverse_comm(sim, Direction::Zlo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);

    // y-direction
    let mut ids = send_reverse_comm(sim, Direction::Yhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);

    let mut ids = send_reverse_comm(sim, Direction::Ylo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);

    // x-direction
    let mut ids = send_reverse_comm(sim, Direction::Xhi);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);

    let mut ids = send_reverse_comm(sim, Direction::Xlo);
    sent_ids.append(&mut ids);
    recv_reverse_comm(sim);
}

pub(crate) fn forward_comm<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    // delete all ghost atoms
    sim.atoms.ids.resize(sim.atoms.nlocal, 0);
    sim.atoms.types.resize(sim.atoms.nlocal, 0);
    sim.atoms.positions.resize(sim.atoms.nlocal, [0.0; 3]);
    sim.atoms.velocities.resize(sim.atoms.nlocal, [0.0; 3]);

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

fn collect_comm_atoms<T, A>(sim: &Simulation<T, A>, direction: &Direction) -> Vec<usize>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
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

fn send_atoms<T, A>(sim: &mut Simulation<T, A>, direction: Direction)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let atom_idxs = collect_comm_atoms(sim, &direction);
    let ids: Vec<usize> = atom_idxs.iter().map(|i| sim.atoms.ids[*i]).collect();
    sim.domain().send(AtomMessage::Idxs(ids), direction);

    sim.remove_idxs(atom_idxs);
}

fn recv_atoms<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let msg = sim.domain().receive();
    match msg {
        AtomMessage::Idxs(new_ids) => {
            let num_new_owned_atoms = sim
                .atoms
                .ids
                .iter()
                .enumerate()
                .filter(|(i, curr_id)| {
                    new_ids.contains(curr_id)
                        && sim.domain().subdomain().contains(&sim.atoms.positions[*i])
                })
                .count();
            sim.atoms.nlocal += num_new_owned_atoms;
        }
        _ => panic!("Invalid message"),
    };
}

pub(crate) fn comm_atom_ownership<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
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

fn recv_reverse_comm<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let id_msg = sim.domain().receive();
    let force_msg = sim.domain().receive();
    match (id_msg, force_msg) {
        (AtomMessage::Idxs(ids), AtomMessage::Float3(new_forces)) => {
            accumulate_forces(sim, &ids, &new_forces)
        }
        (AtomMessage::Float3(new_forces), AtomMessage::Idxs(ids)) => {
            accumulate_forces(sim, &ids, &new_forces)
        }
        _ => panic!("Invalid communication"),
    }
}

fn recv_forward_comm<T, A>(sim: &mut Simulation<T, A>)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let msg = sim.domain().receive();
    match msg {
        AtomMessage::Atom(atoms) => {
            for atom in atoms {
                sim.atoms.update_or_push(atom);
            }
        }
        _ => panic!("Invalid message"),
    };
}

fn gather_ghost_ids<T, A>(sim: &Simulation<T, A>, rect: &Rect) -> Vec<usize>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    sim.atoms
        .ids
        .iter()
        .skip(sim.nlocal())
        .zip(sim.atoms.positions.iter())
        .filter_map(
            |(&id, pos)| {
                if rect.contains(pos) {
                    Some(id)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn gather_owned_idxs<T, A>(sim: &Simulation<T, A>, rect: &Rect) -> Vec<usize>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    sim.atoms
        .positions
        .iter()
        .take(sim.nlocal())
        .enumerate()
        .filter_map(
            |(idx, pos)| {
                if rect.contains(pos) {
                    Some(idx)
                } else {
                    None
                }
            },
        )
        .collect()
}

fn accumulate_forces<T, A>(
    sim: &mut Simulation<T, A>,
    communicated_ids: &Vec<usize>,
    communicated_forces: &Vec<[f64; 3]>,
) where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let natoms = sim.atoms.num_total_atoms();
    for i in 0..natoms {
        let opt = communicated_ids
            .iter()
            .position(|id| *id == sim.atoms.ids[i]);
        if let Some(j) = opt {
            sim.mut_forces()[i][0] += communicated_forces[j][0];
            sim.mut_forces()[i][1] += communicated_forces[j][1];
            sim.mut_forces()[i][2] += communicated_forces[j][2];
        }
    }
}

fn send_reverse_comm<T, A>(sim: &Simulation<T, A>, direction: Direction) -> Vec<usize>
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let ids = gather_ghost_ids(sim, &sim.domain().get_outer_rect(&direction, sim.nl()));
    let mut send_forces: Vec<[f64; 3]> = Vec::new();
    send_forces.reserve(ids.len());

    for id in &ids {
        let j = sim
            .atoms
            .ids
            .iter()
            .position(|i| *i == *id)
            .expect("Should exist");

        send_forces.push(sim.forces()[j].clone());
    }

    sim.domain().send(AtomMessage::Idxs(ids.clone()), direction);
    sim.domain()
        .send(AtomMessage::Float3(send_forces), direction);
    ids
}

fn send_forward_comm<T, A>(sim: &Simulation<T, A>, direction: Direction)
where
    T: AtomType,
    A: AtomicPotentialTrait<T>,
{
    let idxs = gather_owned_idxs(sim, &sim.domain().get_inner_rect(&direction, sim.nl()));
    let atoms: Vec<Atom> = sim
        .atoms
        .ids
        .iter()
        .enumerate()
        .filter_map(|(i, id)| {
            if idxs.contains(&i) {
                Some(Atom {
                    id: *id,
                    type_: sim.atoms.types[i],
                    position: sim.atoms.positions[i],
                    velocity: sim.atoms.velocities[i],
                })
            } else {
                None
            }
        })
        .collect();

    sim.domain().send(AtomMessage::Atom(atoms), direction);
}
