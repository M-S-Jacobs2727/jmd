use std::{
    sync::mpsc,
    thread::{self, ThreadId},
};
// TODO: Finish procs_in_box, integrate utils::indices
// TODO: Extract NeighborDirection to utils::direction, add functionality
use crate::{region::Rect, Box_, ThreadIds};

use super::worker::{Worker, M2W, W2M};

pub struct AtomInfo {
    pub ids: Vec<usize>,
    pub types: Vec<u32>,
    pub data: Vec<f64>,
}
impl AtomInfo {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            types: Vec::new(),
            data: Vec::new(),
        }
    }
}

fn multi_to_linear(idx: &[usize; 3], lengths: &[usize; 3]) -> usize {
    let [x, y, z] = &idx;
    let [nx, ny, nz] = &lengths;
    assert!(
        x < nx && y < ny && z < nz,
        "Multidimensional indices should be smaller than respective lengths"
    );
    x * ny * nz + y * nz + z
}
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

pub enum NeighborDirection {
    Xlo,
    Xhi,
    Ylo,
    Yhi,
    Zlo,
    Zhi,
}
impl NeighborDirection {
    pub fn opposite(&self) -> Self {
        match self {
            NeighborDirection::Xlo => NeighborDirection::Xhi,
            NeighborDirection::Xhi => NeighborDirection::Xlo,
            NeighborDirection::Ylo => NeighborDirection::Yhi,
            NeighborDirection::Yhi => NeighborDirection::Ylo,
            NeighborDirection::Zlo => NeighborDirection::Zhi,
            NeighborDirection::Zhi => NeighborDirection::Zlo,
        }
    }
}

// // This is given to each thread from the main thread
// pub struct DistributionInfo {
//     thread_ids: Vec<ThreadId>,
//     proc_dimensions: [usize; 3],
//     me: [usize; 3],
// }
// impl DistributionInfo {
//     pub fn new(thread_ids: Vec<ThreadId>, proc_dimensions: [usize; 3]) -> Self {
//         let idx = thread_ids
//             .iter()
//             .position(|t| *t == thread::current().id())
//             .expect("Current thread_id not found in given list!");
//         let me = linear_to_multi(idx, &proc_dimensions);
//         Self {
//             thread_ids,
//             proc_dimensions,
//             me,
//         }
//     }
//     pub fn xlo(&self) -> &ThreadId {
//         let xlo_idx = if self.me[0] == 0 {
//             self.proc_dimensions[0] - 1
//         } else {
//             self.me[0] - 1
//         };
//         let idx = multi_to_linear(&[xlo_idx, self.me[1], self.me[2]], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn xhi(&self) -> &ThreadId {
//         let xhi_idx = if self.me[0] == self.proc_dimensions[0] - 1 {
//             0
//         } else {
//             self.me[0] + 1
//         };
//         let idx = multi_to_linear(&[xhi_idx, self.me[1], self.me[2]], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn ylo(&self) -> &ThreadId {
//         let ylo_idx = if self.me[1] == 0 {
//             self.proc_dimensions[1] - 1
//         } else {
//             self.me[1] - 1
//         };
//         let idx = multi_to_linear(&[self.me[0], ylo_idx, self.me[2]], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn yhi(&self) -> &ThreadId {
//         let yhi_idx = if self.me[1] == self.proc_dimensions[1] - 1 {
//             0
//         } else {
//             self.me[1] + 1
//         };
//         let idx = multi_to_linear(&[self.me[0], yhi_idx, self.me[2]], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn zlo(&self) -> &ThreadId {
//         let zlo_idx = if self.me[2] == 0 {
//             self.proc_dimensions[2] - 1
//         } else {
//             self.me[2] - 1
//         };
//         let idx = multi_to_linear(&[self.me[0], self.me[1], zlo_idx], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn zhi(&self) -> &ThreadId {
//         let zhi_idx = if self.me[2] == self.proc_dimensions[2] - 1 {
//             0
//         } else {
//             self.me[2] + 1
//         };
//         let idx = multi_to_linear(&[self.me[0], self.me[1], zhi_idx], &self.proc_dimensions);
//         &self.thread_ids[idx]
//     }
//     pub fn thread_ids(&self) -> &Vec<ThreadId> {
//         &self.thread_ids
//     }
//     pub fn proc_dimensions(&self) -> &[usize; 3] {
//         &self.proc_dimensions
//     }
//     pub fn me(&self) -> &[usize; 3] {
//         &self.me
//     }
// }

pub struct NeighborProcs<AtomInfo> {
    xlo: Option<mpsc::Sender<AtomInfo>>,
    xhi: Option<mpsc::Sender<AtomInfo>>,
    ylo: Option<mpsc::Sender<AtomInfo>>,
    yhi: Option<mpsc::Sender<AtomInfo>>,
    zlo: Option<mpsc::Sender<AtomInfo>>,
    zhi: Option<mpsc::Sender<AtomInfo>>,
}
impl<AtomInfo> NeighborProcs<AtomInfo> {
    pub fn new() -> Self {
        Self {
            xlo: None,
            xhi: None,
            ylo: None,
            yhi: None,
            zlo: None,
            zhi: None,
        }
    }
    pub fn as_vec(&self) -> Vec<&Option<mpsc::Sender<AtomInfo>>> {
        vec![
            &self.xlo, &self.xhi, &self.ylo, &self.yhi, &self.zlo, &self.zhi,
        ]
    }
    pub fn set(&mut self, direction: NeighborDirection, sender: mpsc::Sender<AtomInfo>) {
        match direction {
            NeighborDirection::Xlo => self.xlo = Some(sender),
            NeighborDirection::Xhi => self.xhi = Some(sender),
            NeighborDirection::Ylo => self.ylo = Some(sender),
            NeighborDirection::Yhi => self.yhi = Some(sender),
            NeighborDirection::Zlo => self.zlo = Some(sender),
            NeighborDirection::Zhi => self.zhi = Some(sender),
        };
    }
}

fn all_factors(n: &usize) -> Vec<usize> {
    (2..n + 1).filter(|i| n % i == 0).collect()
}
// TODO: Finish this function
fn procs_in_box(nprocs: usize, lx: f64, ly: f64, lz: f64) -> [usize; 3] {
    // n1 * n2 * n3 = N, lx * ly * lz = V
    // ni = curt(N/V) * li
    let volume = lx * ly * lz;
    let fraction = nprocs as f64 / volume;
    let proc_estimates = [
        lx * fraction.cbrt(),
        ly * fraction.cbrt(),
        lz * fraction.cbrt(),
    ];
    let factors = all_factors(&nprocs);
    let i = factors
        .iter()
        .position(|f| proc_estimates[0] < *f as f64)
        .unwrap_or(factors.len() - 1);
    let [factor_lo, factor_hi] = if i == 0 {
        [factors[0], factors[1]]
    } else {
        [factors[i - 1], factors[i]]
    };
    todo!()
}

pub struct Domain {
    receiver: mpsc::Receiver<AtomInfo>,
    my_sender: mpsc::Sender<AtomInfo>,
    neighbor_procs: NeighborProcs<AtomInfo>,
    thread_ids: ThreadIds,
    subdomain: Rect,
    proc_dimensions: [usize; 3],
    my_idx: [usize; 3],
}
impl Domain {
    pub fn new() -> Self {
        let neighbor_procs: NeighborProcs<AtomInfo> = NeighborProcs::new();
        let (my_sender, receiver) = mpsc::channel();

        Self {
            receiver,
            my_sender,
            neighbor_procs,
            thread_ids: Vec::new(),
            subdomain: Rect::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            proc_dimensions: [0, 0, 0],
            my_idx: [0, 0, 0],
        }
    }
    pub fn init(&mut self, box_: &Box_, worker: &Worker) {
        self.thread_ids.clone_from(worker.thread_ids());

        let num_threads = self.thread_ids.len();
        self.proc_dimensions = procs_in_box(num_threads, box_.lx(), box_.ly(), box_.lz());

        self.my_idx = linear_to_multi(
            self.thread_ids
                .iter()
                .position(|&id| thread::current().id() == id)
                .unwrap(),
            &self.proc_dimensions,
        );
        self.reset_subdomain(box_);

        self.setup_neighbor(worker, NeighborDirection::Xlo, box_);
        self.setup_neighbor(worker, NeighborDirection::Xhi, box_);
        self.setup_neighbor(worker, NeighborDirection::Ylo, box_);
        self.setup_neighbor(worker, NeighborDirection::Yhi, box_);
        self.setup_neighbor(worker, NeighborDirection::Zlo, box_);
        self.setup_neighbor(worker, NeighborDirection::Zhi, box_);
    }
    fn setup_neighbor(&mut self, worker: &Worker, direction: NeighborDirection, box_: &Box_) {
        // Get index of neighbor (3d then 1d), if neighbor is present, send Option<mpsc::Sender> to main with proc idx, otherwise None and 0
        // Receive from main Option<mpsc::Sender> for opposite neighbor
        let idx = self.get_1d_neighbor(&self.my_idx, &direction, box_);
        let msg = match idx {
            Some(i) => (Some(self.my_sender.clone()), i),
            None => (None, 0),
        };
        worker.send(W2M::Sender(msg.0, msg.1)).unwrap();
        let msg = worker.recv();
        match msg {
            Ok(M2W::Sender(Some(sender))) => self.neighbor_procs.set(direction.opposite(), sender),
            Ok(M2W::Sender(None)) => {}
            Ok(_) => panic!("Invalid message"),
            _ => panic!("Disconnect error"),
        };
    }
    pub fn initialized(&self) -> bool {
        !self.thread_ids.is_empty()
    }
    pub fn reset_subdomain(&mut self, box_: &Box_) {
        let l = [
            box_.lx() / (self.proc_dimensions[0] as f64),
            box_.ly() / (self.proc_dimensions[1] as f64),
            box_.lz() / (self.proc_dimensions[2] as f64),
        ];
        let lo = box_.lo();
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
    pub fn clone_sender(&self) -> mpsc::Sender<AtomInfo> {
        self.my_sender.clone()
    }
    pub fn receiver(&self) -> &mpsc::Receiver<AtomInfo> {
        &self.receiver
    }
    pub fn neighbor_procs(&self) -> &NeighborProcs<AtomInfo> {
        &self.neighbor_procs
    }
    pub fn set_neighbor_proc(
        &mut self,
        direction: NeighborDirection,
        sender: mpsc::Sender<AtomInfo>,
    ) {
        self.neighbor_procs.set(direction, sender);
    }
    pub fn set_thread_ids(&mut self, thread_ids: Vec<ThreadId>) {
        self.thread_ids = thread_ids;
    }
    pub fn thread_ids(&self) -> &ThreadIds {
        &self.thread_ids
    }
    pub fn num_neighbors(&self) -> usize {
        self.neighbor_procs
            .as_vec()
            .iter()
            .filter(|&&p| (*p).is_some())
            .count()
    }
    pub fn receive(&self) -> Result<Option<AtomInfo>, mpsc::RecvError> {
        match self.receiver.try_recv() {
            Ok(t) => Ok(Some(t)),
            Err(e) => match e {
                mpsc::TryRecvError::Empty => Ok(None),
                mpsc::TryRecvError::Disconnected => Err(mpsc::RecvError),
            },
        }
    }
    pub fn send(
        &self,
        value: AtomInfo,
        neighbor: NeighborDirection,
    ) -> Result<(), mpsc::SendError<AtomInfo>> {
        let n = match neighbor {
            NeighborDirection::Xlo => self.neighbor_procs.xlo.as_ref(),
            NeighborDirection::Xhi => self.neighbor_procs.xhi.as_ref(),
            NeighborDirection::Ylo => self.neighbor_procs.ylo.as_ref(),
            NeighborDirection::Yhi => self.neighbor_procs.yhi.as_ref(),
            NeighborDirection::Zlo => self.neighbor_procs.zlo.as_ref(),
            NeighborDirection::Zhi => self.neighbor_procs.zhi.as_ref(),
        };
        match n {
            Some(s) => s.send(value),
            None => Ok(()),
        }
    }
    fn get_3d_neighbor(
        &self,
        my_idx: &[usize; 3],
        direction: &NeighborDirection,
        box_: &Box_,
    ) -> Option<[usize; 3]> {
        let (across_box, possible_neighbor) = match direction {
            NeighborDirection::Xlo => {
                if my_idx[0] == 0 {
                    (true, [self.proc_dimensions[0] - 1, my_idx[1], my_idx[2]])
                } else {
                    (false, [my_idx[0] - 1, my_idx[1], my_idx[2]])
                }
            }
            NeighborDirection::Xhi => {
                if my_idx[0] == self.proc_dimensions[0] - 1 {
                    (true, [0, my_idx[1], my_idx[2]])
                } else {
                    (false, [my_idx[0] + 1, my_idx[1], my_idx[2]])
                }
            }
            NeighborDirection::Ylo => {
                if my_idx[1] == 0 {
                    (true, [my_idx[0], self.proc_dimensions[1] - 1, my_idx[2]])
                } else {
                    (false, [my_idx[0], my_idx[1] - 1, my_idx[2]])
                }
            }
            NeighborDirection::Yhi => {
                if my_idx[1] == self.proc_dimensions[1] - 1 {
                    (true, [my_idx[0], 0, my_idx[2]])
                } else {
                    (false, [my_idx[0], my_idx[1] + 1, my_idx[2]])
                }
            }
            NeighborDirection::Zlo => {
                if my_idx[2] == 0 {
                    (true, [my_idx[0], my_idx[1], self.proc_dimensions[2] - 1])
                } else {
                    (false, [my_idx[0], my_idx[1], my_idx[2] - 1])
                }
            }
            NeighborDirection::Zhi => {
                if my_idx[2] == self.proc_dimensions[2] - 1 {
                    (true, [my_idx[0], my_idx[1], 0])
                } else {
                    (false, [my_idx[0], my_idx[1], my_idx[2] + 1])
                }
            }
        };
        if across_box {
            let periodic = match direction {
                NeighborDirection::Xlo => box_.x().pbc().is_periodic(),
                NeighborDirection::Xhi => box_.x().pbc().is_periodic(),
                NeighborDirection::Ylo => box_.y().pbc().is_periodic(),
                NeighborDirection::Yhi => box_.y().pbc().is_periodic(),
                NeighborDirection::Zlo => box_.z().pbc().is_periodic(),
                NeighborDirection::Zhi => box_.z().pbc().is_periodic(),
            };
            if periodic {
                Some(possible_neighbor)
            } else {
                None
            }
        } else {
            Some(possible_neighbor)
        }
    }
    fn get_1d_neighbor(
        &self,
        my_idx: &[usize; 3],
        direction: &NeighborDirection,
        box_: &Box_,
    ) -> Option<usize> {
        let idx3d = self.get_3d_neighbor(my_idx, direction, box_);
        match idx3d {
            Some(idx) => Some(multi_to_linear(&idx, &self.proc_dimensions)),
            None => None,
        }
    }
}
