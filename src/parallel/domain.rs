// TODO: integrate utils::indices
use std::{sync::mpsc, thread};

use super::*;
use crate::{
    atom_type::AtomType,
    atomic,
    container::Container,
    neighbor::NeighborList,
    region::Rect,
    utils::{Direction, Index},
};

/// Determine and return the best configuration of processes to
/// reduce surface area for communication
fn procs_in_box(nprocs: usize, lx: f64, ly: f64, lz: f64) -> [usize; 3] {
    if nprocs == 1 {
        return [1, 1, 1];
    }
    // This score is proportional to the surface area and therefore should be minimized
    let score = |nx: usize, ny: usize, nz: usize| {
        lx * ly / (nx * ny) as f64 + ly * lz / (ny * nz) as f64 + lx * lz / (nx * nz) as f64
    };

    let factors: Vec<usize> = (1..=nprocs).filter(|i| nprocs % i == 0).collect();
    let (i, j, _) = factors
        .clone()
        .iter()
        .enumerate()
        .map(|(i, &nx)| {
            factors
                .iter()
                .enumerate()
                .filter(|(_, &ny)| ny * nx <= nprocs && nprocs % (ny * nx) == 0)
                .map(|(j, &ny)| {
                    let nz = nprocs / nx / ny;
                    (i, j, score(nx, ny, nz))
                })
                .reduce(|x, y| if x.2 < y.2 { x } else { y })
                .unwrap_or((usize::MAX, usize::MAX, f64::MAX))
        })
        .reduce(|x, y| if x.2 < y.2 { x } else { y })
        .unwrap();

    [factors[i], factors[j], nprocs / factors[i] / factors[j]]
}

/// Represents a process in relation to the other neighboring processes
pub struct Domain<'a, T: AtomType, A: atomic::AtomicPotentialTrait<T>> {
    receiver: mpsc::Receiver<AtomMessage>,
    my_sender: mpsc::Sender<AtomMessage>,
    worker: Option<Box<&'a Worker<T, A>>>,
    procs: AdjacentProcs,
    subdomain: Rect,
    proc_index: Index,
}
impl<'a, T: AtomType, A: atomic::AtomicPotentialTrait<T>> Domain<'a, T, A> {
    pub(crate) fn new() -> Self {
        let neighbor_procs: AdjacentProcs = AdjacentProcs::new();
        let (my_sender, receiver) = mpsc::channel();

        Self {
            receiver,
            my_sender,
            worker: None,
            procs: neighbor_procs,
            subdomain: Rect::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0),
            proc_index: Index::new(),
        }
    }
    pub(crate) fn init(&mut self, container: &Container, worker: Box<&'a Worker<T, A>>) {
        self.worker = Some(worker);

        let num_threads = self.thread_ids().len();

        let proc_dimensions = procs_in_box(
            num_threads,
            container.rect().lx(),
            container.rect().ly(),
            container.rect().lz(),
        );
        let idx = self
            .thread_ids()
            .iter()
            .position(|&id| thread::current().id() == id)
            .unwrap();
        self.proc_index = Index::from_1d(idx, proc_dimensions);

        self.reset_subdomain(container.rect());

        vec![
            Direction::Xlo,
            Direction::Xhi,
            Direction::Ylo,
            Direction::Yhi,
            Direction::Zlo,
            Direction::Zhi,
        ]
        .iter()
        .for_each(|direction| self.setup_neighbor(*direction, container));
    }
    pub(crate) fn proc_index(&self) -> usize {
        self.proc_index.idx()
    }
    pub(crate) fn subdomain(&self) -> &Rect {
        &self.subdomain
    }
    pub(crate) fn worker(&self) -> &Box<&'a Worker<T, A>> {
        self.worker.as_ref().expect("Must init")
    }
    fn setup_neighbor(&mut self, direction: Direction, container: &Container) {
        // Get index of neighbor (3d then 1d), if neighbor is present, send Option<mpsc::Sender> to main with proc idx, otherwise None and 0
        // Receive from main Option<mpsc::Sender> for opposite neighbor
        let idx = self.get_neighbor(direction.clone(), container);
        let message = match idx {
            Some(i) => W2M::Sender(Some(self.my_sender.clone()), i.idx()),
            None => W2M::Sender(None, 0),
        };
        self.worker().send(message);
        let message = self.worker().recv();
        match message {
            M2W::Sender(Some(sender)) => {
                self.procs.set(direction.opposite(), sender);
            }
            M2W::Sender(None) => {}
            _ => panic!("Invalid message"),
        };
    }
    pub fn initialized(&self) -> bool {
        self.worker.is_some()
    }
    pub fn reset_subdomain(&mut self, rect: &Rect) {
        let bounds = self.proc_index.bounds();
        let l = [
            rect.lx() / (bounds[0] as f64),
            rect.ly() / (bounds[1] as f64),
            rect.lz() / (bounds[2] as f64),
        ];
        let lo = rect.lo();
        let idx3d = self.proc_index.to_3d();
        let sdlo = [
            lo[0] + l[0] * idx3d[0] as f64,
            lo[1] + l[1] * idx3d[1] as f64,
            lo[2] + l[2] * idx3d[2] as f64,
        ];
        self.subdomain = Rect::new(
            sdlo[0],
            sdlo[0] + l[0],
            sdlo[1],
            sdlo[1] + l[1],
            sdlo[2],
            sdlo[2] + l[2],
        );
    }

    pub fn get_inner_rect(&self, direction: &Direction, neighbor_list: &NeighborList) -> Rect {
        let dist = neighbor_list.max_neighbor_distance();
        let half_skin = neighbor_list.skin_distance() * 0.5;

        match direction {
            Direction::Xlo => Rect::new(
                self.subdomain.xlo() - half_skin,
                self.subdomain.xlo() + dist,
                self.subdomain.ylo() - half_skin,
                self.subdomain.yhi() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Xhi => Rect::new(
                self.subdomain.xhi() - dist,
                self.subdomain.xhi() + half_skin,
                self.subdomain.ylo() - half_skin,
                self.subdomain.yhi() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Ylo => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - half_skin,
                self.subdomain.ylo() + dist,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Yhi => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.yhi() - dist,
                self.subdomain.yhi() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Zlo => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - dist,
                self.subdomain.yhi() + dist,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zlo() + dist,
            ),
            Direction::Zhi => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - dist,
                self.subdomain.yhi() + dist,
                self.subdomain.zhi() - dist,
                self.subdomain.zhi() + half_skin,
            ),
        }
    }

    pub fn get_outer_rect(&self, direction: &Direction, neighbor_list: &NeighborList) -> Rect {
        let dist = neighbor_list.max_neighbor_distance();
        let half_skin = neighbor_list.skin_distance() * 0.5;

        match direction {
            Direction::Xlo => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xlo() + half_skin,
                self.subdomain.ylo() - half_skin,
                self.subdomain.yhi() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Xhi => Rect::new(
                self.subdomain.xhi() - half_skin,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - half_skin,
                self.subdomain.yhi() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Ylo => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - dist,
                self.subdomain.ylo() + half_skin,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Yhi => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.yhi() - half_skin,
                self.subdomain.yhi() + dist,
                self.subdomain.zlo() - half_skin,
                self.subdomain.zhi() + half_skin,
            ),
            Direction::Zlo => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - dist,
                self.subdomain.yhi() + dist,
                self.subdomain.zlo() - dist,
                self.subdomain.zlo() + half_skin,
            ),
            Direction::Zhi => Rect::new(
                self.subdomain.xlo() - dist,
                self.subdomain.xhi() + dist,
                self.subdomain.ylo() - dist,
                self.subdomain.yhi() + dist,
                self.subdomain.zhi() - half_skin,
                self.subdomain.zhi() + dist,
            ),
        }
    }

    pub fn clone_sender(&self) -> mpsc::Sender<AtomMessage> {
        self.my_sender.clone()
    }
    pub fn receiver(&self) -> &mpsc::Receiver<AtomMessage> {
        &self.receiver
    }
    pub fn neighbor_procs(&self) -> &AdjacentProcs {
        &self.procs
    }
    pub fn set_neighbor_proc(&mut self, direction: Direction, sender: mpsc::Sender<AtomMessage>) {
        self.procs.set(direction, sender);
    }
    pub fn thread_ids(&self) -> &Vec<thread::ThreadId> {
        self.worker().thread_ids()
    }
    pub fn num_neighbors(&self) -> usize {
        self.procs
            .as_vec()
            .iter()
            .filter(|&&p| (*p).is_some())
            .count()
    }
    pub fn receive(&self) -> AtomMessage {
        self.receiver.recv().expect("Disconnect error")
    }
    pub fn send(&self, value: AtomMessage, neighbor: Direction) {
        let n = match neighbor {
            Direction::Xlo => self.procs.xlo(),
            Direction::Xhi => self.procs.xhi(),
            Direction::Ylo => self.procs.ylo(),
            Direction::Yhi => self.procs.yhi(),
            Direction::Zlo => self.procs.zlo(),
            Direction::Zhi => self.procs.zhi(),
        };
        if let Some(s) = n {
            s.send(value).expect("Disconnect error");
        }
    }
    pub(crate) fn send_to_main(&self, message: W2M<T>) {
        self.worker().send(message);
    }
    pub(crate) fn recv_from_main(&self) -> M2W<T, A> {
        self.worker().recv()
    }

    fn get_neighbor(&self, direction: Direction, container: &Container) -> Option<Index> {
        let axis_index = direction.axis().index();
        let my_idx = self.proc_index.to_3d();
        let bounds = self.proc_index.bounds();

        let i = my_idx[axis_index];
        let n = bounds[axis_index];
        let across_box = if direction.is_lo() {
            i == 0
        } else {
            i == n - 1
        };
        let mut idx = my_idx.clone();
        match (across_box, direction.is_lo()) {
            (false, false) => {
                idx[axis_index] += 1;
            }
            (false, true) => {
                idx[axis_index] -= 1;
            }
            (true, false) => {
                idx[axis_index] = 0;
            }
            (true, true) => {
                idx[axis_index] = n - 1;
            }
        };

        if across_box && !container.is_periodic(direction.axis()) {
            None
        } else {
            Some(Index::from_3d(&idx, &bounds))
        }
    }

    pub(crate) fn send_to_main_once(&self, message: W2M<T>) {
        if self.proc_index.idx() == 0 {
            self.send_to_main(message);
        }
    }
}
