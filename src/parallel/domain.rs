// TODO: integrate utils::indices
use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use super::{message as msg, worker::Worker, AdjacentProcs};
use crate::{
    region::Rect,
    utils::{indices::Index, Direction},
    Container, NeighborList,
};

/// Determine and return the best configuration of processes to
/// reduce surface area for communication
fn procs_in_box(nprocs: usize, lx: f64, ly: f64, lz: f64) -> [usize; 3] {
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
pub struct Domain<'a> {
    receiver: mpsc::Receiver<msg::Message>,
    my_sender: mpsc::Sender<msg::Message>,
    worker: Option<Box<&'a Worker>>,
    procs: AdjacentProcs,
    subdomain: Rect,
    proc_location: Index,
}
impl<'a> Domain<'a> {
    pub(crate) fn new() -> Self {
        let neighbor_procs: AdjacentProcs = AdjacentProcs::new();
        let (my_sender, receiver) = mpsc::channel();

        Self {
            receiver,
            my_sender,
            worker: None,
            procs: neighbor_procs,
            subdomain: Rect::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0),
            proc_location: Index::new(),
        }
    }
    pub(crate) fn init(&mut self, container: &Container, worker: Box<&'a Worker>) {
        self.worker = Some(worker);

        let num_threads = self.thread_ids().len();

        let proc_dimensions =
            procs_in_box(num_threads, container.lx(), container.ly(), container.lz());
        let idx = self
            .thread_ids()
            .iter()
            .position(|&id| thread::current().id() == id)
            .unwrap();
        self.proc_location.set_bounds(proc_dimensions);
        self.proc_location.set_idx(idx);

        self.reset_subdomain(container);

        self.setup_neighbor(Direction::Xlo, container);
        self.setup_neighbor(Direction::Xhi, container);
        self.setup_neighbor(Direction::Ylo, container);
        self.setup_neighbor(Direction::Yhi, container);
        self.setup_neighbor(Direction::Zlo, container);
        self.setup_neighbor(Direction::Zhi, container);
    }
    pub(crate) fn subdomain(&self) -> &Rect {
        &self.subdomain
    }
    pub(crate) fn worker(&self) -> &Box<&'a Worker> {
        self.worker.as_ref().expect("Must init")
    }
    fn setup_neighbor(&mut self, direction: Direction, container: &Container) {
        // Get index of neighbor (3d then 1d), if neighbor is present, send Option<mpsc::Sender> to main with proc idx, otherwise None and 0
        // Receive from main Option<mpsc::Sender> for opposite neighbor
        let idx = self.get_neighbor(direction.clone(), container);
        let message = match idx {
            Some(i) => msg::W2M::Sender(Some(self.my_sender.clone()), i.idx()),
            None => msg::W2M::Sender(None, 0),
        };
        self.worker().send(message);
        let msg = self.worker().recv();
        match msg {
            Ok(msg::M2W::Sender(Some(sender))) => self.procs.set(direction.opposite(), sender),
            Ok(msg::M2W::Sender(None)) => {}
            Ok(_) => panic!("Invalid message"),
            _ => panic!("Disconnect error"),
        };
    }
    pub fn initialized(&self) -> bool {
        self.worker.is_some()
    }
    pub fn reset_subdomain(&mut self, container: &Container) {
        let bounds = self.proc_location.bounds();
        let l = [
            container.lx() / (bounds[0] as f64),
            container.ly() / (bounds[1] as f64),
            container.lz() / (bounds[2] as f64),
        ];
        let lo = container.lo();
        let idx3d = self.proc_location.to_3d();
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
        let dist = neighbor_list.neighbor_distance();
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
        let dist = neighbor_list.neighbor_distance();
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

    pub fn clone_sender(&self) -> mpsc::Sender<msg::Message> {
        self.my_sender.clone()
    }
    pub fn receiver(&self) -> &mpsc::Receiver<msg::Message> {
        &self.receiver
    }
    pub fn neighbor_procs(&self) -> &AdjacentProcs {
        &self.procs
    }
    pub fn set_neighbor_proc(&mut self, direction: Direction, sender: mpsc::Sender<msg::Message>) {
        self.procs.set(direction, sender);
    }
    pub fn thread_ids(&self) -> &Vec<ThreadId> {
        self.worker().thread_ids()
    }
    pub fn num_neighbors(&self) -> usize {
        self.procs
            .as_vec()
            .iter()
            .filter(|&&p| (*p).is_some())
            .count()
    }
    pub fn receive(&self) -> Result<msg::Message, mpsc::RecvError> {
        self.receiver.recv()
    }
    pub fn send(
        &self,
        value: msg::Message,
        neighbor: Direction,
    ) -> Result<(), mpsc::SendError<msg::Message>> {
        let n = match neighbor {
            Direction::Xlo => self.procs.xlo(),
            Direction::Xhi => self.procs.xhi(),
            Direction::Ylo => self.procs.ylo(),
            Direction::Yhi => self.procs.yhi(),
            Direction::Zlo => self.procs.zlo(),
            Direction::Zhi => self.procs.zhi(),
        };
        match n {
            Some(s) => s.send(value),
            None => Ok(()),
        }
    }
    pub(crate) fn send_to_main(&self, message: msg::W2M) {
        self.worker().send(message);
    }

    fn get_neighbor(&self, direction: Direction, container: &Container) -> Option<Index> {
        let axis_index = direction.axis().index();
        let my_idx = self.proc_location.to_3d();
        let bounds = self.proc_location.bounds();

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
}
