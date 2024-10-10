use std::{sync::mpsc, thread::ThreadId};

use crate::ThreadIds;

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

// fn multi_to_linear(idx: &[usize; 3], lengths: &[usize; 3]) -> usize {
//     let [x, y, z] = &idx;
//     let [nx, ny, nz] = &lengths;
//     assert!(
//         x < nx && y < ny && z < nz,
//         "Multidimensional indices should be smaller than respective lengths"
//     );
//     x * ny * nz + y * nz + z
// }
// fn linear_to_multi(idx: usize, lengths: &[usize; 3]) -> [usize; 3] {
//     let [nx, ny, nz] = &lengths;
//     assert!(
//         nx * ny * nz > idx,
//         "Index should be smaller than total number"
//     );
//     let z = idx % nz;
//     let r = idx / nz;
//     let y = r % ny;
//     let x = r / ny;
//     [x, y, z]
// }

pub enum NeighborDirection {
    Xlo,
    Xhi,
    Ylo,
    Yhi,
    Zlo,
    Zhi,
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

pub struct NeighborProcs<T> {
    xlo: Option<mpsc::Sender<T>>,
    xhi: Option<mpsc::Sender<T>>,
    ylo: Option<mpsc::Sender<T>>,
    yhi: Option<mpsc::Sender<T>>,
    zlo: Option<mpsc::Sender<T>>,
    zhi: Option<mpsc::Sender<T>>,
}
impl<T> NeighborProcs<T> {
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
    pub fn as_vec(&self) -> Vec<&Option<mpsc::Sender<T>>> {
        vec![
            &self.xlo, &self.xhi, &self.ylo, &self.yhi, &self.zlo, &self.zhi,
        ]
    }
    pub fn set(&mut self, direction: NeighborDirection, sender: mpsc::Sender<T>) {
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

pub struct Communicator<T> {
    receiver: mpsc::Receiver<T>,
    my_sender: mpsc::Sender<T>,
    neighbor_procs: NeighborProcs<T>,
    thread_ids: ThreadIds,
}
impl<T> Communicator<T> {
    pub fn new() -> Self {
        let neighbor_procs: NeighborProcs<T> = NeighborProcs::new();
        let (my_sender, receiver) = mpsc::channel();

        Self {
            receiver,
            my_sender,
            neighbor_procs,
            thread_ids: Vec::new(),
        }
    }
    pub fn clone_sender(&self) -> mpsc::Sender<T> {
        self.my_sender.clone()
    }
    pub fn receiver(&self) -> &mpsc::Receiver<T> {
        &self.receiver
    }
    pub fn neighbor_procs(&self) -> &NeighborProcs<T> {
        &self.neighbor_procs
    }
    pub fn set_neighbor_proc(&mut self, direction: NeighborDirection, sender: mpsc::Sender<T>) {
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
    pub fn receive(&self) -> Result<Option<T>, mpsc::RecvError> {
        match self.receiver.try_recv() {
            Ok(t) => Ok(Some(t)),
            Err(e) => match e {
                mpsc::TryRecvError::Empty => Ok(None),
                mpsc::TryRecvError::Disconnected => Err(mpsc::RecvError),
            },
        }
    }
    pub fn send(&self, value: T, neighbor: NeighborDirection) -> Result<(), mpsc::SendError<T>> {
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
}
