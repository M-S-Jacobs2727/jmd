// TODO: integrate utils::indices
use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};

use super::{
    message::Message,
    worker::{Worker, M2W, W2M},
    AdjacentProcs,
};
use crate::{region::Rect, utils::Direction, Container, NeighborList};

/// Transform a linear index of a L*M*N vector to a 3D index of a LxMxN array
fn multi_to_linear(idx: &[usize; 3], lengths: &[usize; 3]) -> usize {
    let [x, y, z] = &idx;
    let [nx, ny, nz] = &lengths;
    assert!(
        x < nx && y < ny && z < nz,
        "Multidimensional indices should be smaller than respective lengths"
    );
    x * ny * nz + y * nz + z
}
/// Transform a 3D index of a LxMxN array to a linear index of a L*M*N vector
fn linear_to_multi(idx: usize, lengths: &[usize; 3]) -> [usize; 3] {
    let [nx, ny, nz] = &lengths;
    assert!(
        nx * ny * nz > idx,
        "Index should be smaller than total number"
    );
    let z = idx % nz;
    let r = idx / nz;
    let y = r % ny;
    let x = r / ny;
    [x, y, z]
}

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
    receiver: mpsc::Receiver<Message>,
    my_sender: mpsc::Sender<Message>,
    worker: Option<Box<&'a Worker>>,
    procs: AdjacentProcs,
    thread_ids: Vec<ThreadId>,
    subdomain: Rect,
    proc_dimensions: [usize; 3],
    my_idx: [usize; 3],
}
impl<'a> Domain<'a> {
    pub fn new() -> Self {
        let neighbor_procs: AdjacentProcs = AdjacentProcs::new();
        let (my_sender, receiver) = mpsc::channel();

        Self {
            receiver,
            my_sender,
            worker: None,
            procs: neighbor_procs,
            thread_ids: Vec::new(),
            subdomain: Rect::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0),
            proc_dimensions: [0, 0, 0],
            my_idx: [0, 0, 0],
        }
    }
    pub fn init(&mut self, container: &Container, worker: Box<&'a Worker>) {
        self.thread_ids.clone_from(worker.thread_ids());
        self.worker = Some(worker);

        let num_threads = self.thread_ids.len();
        self.proc_dimensions =
            procs_in_box(num_threads, container.lx(), container.ly(), container.lz());

        self.my_idx = linear_to_multi(
            self.thread_ids
                .iter()
                .position(|&id| thread::current().id() == id)
                .unwrap(),
            &self.proc_dimensions,
        );
        self.reset_subdomain(container);

        self.setup_neighbor(Direction::Xlo, container);
        self.setup_neighbor(Direction::Xhi, container);
        self.setup_neighbor(Direction::Ylo, container);
        self.setup_neighbor(Direction::Yhi, container);
        self.setup_neighbor(Direction::Zlo, container);
        self.setup_neighbor(Direction::Zhi, container);
    }
    pub fn subdomain(&self) -> &Rect {
        &self.subdomain
    }
    fn worker(&self) -> &Box<&'a Worker> {
        self.worker.as_ref().expect("Must init")
    }
    fn setup_neighbor(&mut self, direction: Direction, container: &Container) {
        // Get index of neighbor (3d then 1d), if neighbor is present, send Option<mpsc::Sender> to main with proc idx, otherwise None and 0
        // Receive from main Option<mpsc::Sender> for opposite neighbor
        let idx = self.get_1d_neighbor(&self.my_idx, direction.clone(), container);
        let msg = match idx {
            Some(i) => (Some(self.my_sender.clone()), i),
            None => (None, 0),
        };
        self.worker().send(W2M::Sender(msg.0, msg.1)).unwrap();
        let msg = self.worker().recv();
        match msg {
            Ok(M2W::Sender(Some(sender))) => self.procs.set(direction.opposite(), sender),
            Ok(M2W::Sender(None)) => {}
            Ok(_) => panic!("Invalid message"),
            _ => panic!("Disconnect error"),
        };
    }
    pub fn initialized(&self) -> bool {
        !self.thread_ids.is_empty()
    }
    pub fn reset_subdomain(&mut self, container: &Container) {
        let l = [
            container.lx() / (self.proc_dimensions[0] as f64),
            container.ly() / (self.proc_dimensions[1] as f64),
            container.lz() / (self.proc_dimensions[2] as f64),
        ];
        let lo = container.lo();
        let sdlo = [
            lo[0] + l[0] * self.my_idx[0] as f64,
            lo[1] + l[1] * self.my_idx[1] as f64,
            lo[2] + l[2] * self.my_idx[2] as f64,
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

    pub fn clone_sender(&self) -> mpsc::Sender<Message> {
        self.my_sender.clone()
    }
    pub fn receiver(&self) -> &mpsc::Receiver<Message> {
        &self.receiver
    }
    pub fn neighbor_procs(&self) -> &AdjacentProcs {
        &self.procs
    }
    pub fn set_neighbor_proc(&mut self, direction: Direction, sender: mpsc::Sender<Message>) {
        self.procs.set(direction, sender);
    }
    pub fn set_thread_ids(&mut self, thread_ids: Vec<ThreadId>) {
        self.thread_ids = thread_ids;
    }
    pub fn thread_ids(&self) -> &Vec<ThreadId> {
        &self.thread_ids
    }
    pub fn num_neighbors(&self) -> usize {
        self.procs
            .as_vec()
            .iter()
            .filter(|&&p| (*p).is_some())
            .count()
    }
    pub fn receive(&self) -> Result<Message, mpsc::RecvError> {
        self.receiver.recv()
    }
    pub fn send(
        &self,
        value: Message,
        neighbor: Direction,
    ) -> Result<(), mpsc::SendError<Message>> {
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
    fn get_3d_neighbor(
        &self,
        my_idx: &[usize; 3],
        direction: Direction,
        container: &Container,
    ) -> Option<[usize; 3]> {
        let axis_index = direction.axis().index();
        let across_box = if direction.is_lo() {
            my_idx[axis_index] == 0
        } else {
            my_idx[axis_index] == self.proc_dimensions[axis_index] - 1
        };
        let possible_neighbor = match (across_box, direction.is_lo()) {
            (false, false) => {
                let mut idx = my_idx.clone();
                idx[axis_index] += 1;
                idx
            }
            (false, true) => {
                let mut idx = my_idx.clone();
                idx[axis_index] -= 1;
                idx
            }
            (true, false) => {
                let mut idx = my_idx.clone();
                idx[axis_index] = 0;
                idx
            }
            (true, true) => {
                let mut idx = my_idx.clone();
                idx[axis_index] = self.proc_dimensions[axis_index] - 1;
                idx
            }
        };
        if across_box && !container.is_periodic(direction) {
            None
        } else {
            Some(possible_neighbor)
        }
    }
    fn get_1d_neighbor(
        &self,
        my_idx: &[usize; 3],
        direction: Direction,
        container: &Container,
    ) -> Option<usize> {
        let idx3d = self.get_3d_neighbor(my_idx, direction, container);
        match idx3d {
            Some(idx) => Some(multi_to_linear(&idx, &self.proc_dimensions)),
            None => None,
        }
    }
}
