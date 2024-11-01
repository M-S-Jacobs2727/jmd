use std::rc::Rc;

use crate::{utils::indices::Index, Container};

/// Neighbor list grid of bins
#[derive(Debug)]
pub(super) struct Grid {
    lo_corner: [f64; 3],
    bin_size: f64,
    neighbor_distance: f64,
    num_bins: [usize; 3],
    container: Rc<Container>,
}
impl Grid {
    pub(super) fn new(container: Rc<Container>, bin_size: f64, neighbor_distance: f64) -> Self {
        assert!(
            bin_size > 0.0,
            "Bin size should be positive, found {}",
            bin_size
        );
        assert!(
            neighbor_distance > 0.0,
            "Neighbor distance should be positive, found {}",
            neighbor_distance
        );
        let min_box_length = container.lx().min(container.ly()).min(container.lz());
        assert!(
            bin_size < 0.5 * min_box_length,
            "Bin size must be less than half the smallest box length, \
             found bin_size {} and smallest box length {}",
            bin_size,
            min_box_length
        );
        let buffer = 2.0 * neighbor_distance;
        let lo_corner = [
            container.xlo() - buffer,
            container.ylo() - buffer,
            container.zlo() - buffer,
        ];
        let num_bins: [usize; 3] = [
            ((container.lx() + 2.0 * buffer) / bin_size).ceil() as usize,
            ((container.ly() + 2.0 * buffer) / bin_size).ceil() as usize,
            ((container.lz() + 2.0 * buffer) / bin_size).ceil() as usize,
        ];
        Self {
            lo_corner,
            bin_size,
            neighbor_distance,
            num_bins,
            container,
        }
    }
    fn recompute(&mut self) {
        let buffer = 2.0 * self.neighbor_distance;
        self.lo_corner = [
            self.container.xlo() - buffer,
            self.container.ylo() - buffer,
            self.container.zlo() - buffer,
        ];
        self.num_bins = [
            ((self.container.lx() + 2.0 * buffer) / self.bin_size).ceil() as usize,
            ((self.container.ly() + 2.0 * buffer) / self.bin_size).ceil() as usize,
            ((self.container.lz() + 2.0 * buffer) / self.bin_size).ceil() as usize,
        ];
    }
    pub(super) fn bin_size(&self) -> f64 {
        self.bin_size
    }
    pub(super) fn set_bin_size(&mut self, bin_size: f64) {
        assert!(
            bin_size > 0.0,
            "Bin size should be positive, found {}",
            bin_size
        );
        self.bin_size = bin_size;
        self.recompute();
    }
    pub(super) fn set_neighbor_distance(&mut self, neighbor_distance: f64) {
        assert!(
            neighbor_distance > 0.0,
            "Neighbor distance should be positive, found {}",
            neighbor_distance
        );
        self.neighbor_distance = neighbor_distance;
        self.recompute();
    }
    pub(super) fn total_num_bins(&self) -> usize {
        self.num_bins[0] * self.num_bins[1] * self.num_bins[2]
    }
    // pub(super) fn lo_corner(&self) -> [f64; 3] {
    //     self.lo_corner
    // }
    // pub(super) fn hi_corner(&self) -> [f64; 3] {
    //     [
    //         self.lo_corner[0] + self.bin_size * self.num_bins[0] as f64,
    //         self.lo_corner[1] + self.bin_size * self.num_bins[1] as f64,
    //         self.lo_corner[2] + self.bin_size * self.num_bins[2] as f64,
    //     ]
    // }
    pub(super) fn num_bins(&self) -> [usize; 3] {
        self.num_bins
    }
    pub(super) fn coord_to_index(&self, coord: &[f64; 3]) -> Index {
        let inds = [
            ((coord[0] - self.lo_corner[0]) / self.bin_size).floor(),
            ((coord[1] - self.lo_corner[1]) / self.bin_size).floor(),
            ((coord[2] - self.lo_corner[2]) / self.bin_size).floor(),
        ];
        assert!(
            inds[0] >= 0.0 && inds[1] >= 0.0 && inds[2] >= 0.0,
            "Coordinates ({:?}) should be within grid ({:?}, {:?}",
            inds,
            self.lo_corner,
            [
                self.lo_corner[0] + self.num_bins[0] as f64 * self.bin_size,
                self.lo_corner[1] + self.num_bins[1] as f64 * self.bin_size,
                self.lo_corner[2] + self.num_bins[2] as f64 * self.bin_size,
            ]
        );
        Index::from_3d(
            &[inds[0] as usize, inds[1] as usize, inds[2] as usize],
            &self.num_bins(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::BC;

    use super::*;
    fn setup_grid() -> Grid {
        let container = Container::new(0.0, 10.0, 0.0, 10.0, 0.0, 10.0, BC::PP, BC::PP, BC::PP);
        Grid::new(Rc::new(container), 2.0, 3.0)
    }

    #[test]
    fn test_grid_basic() {
        let grid = setup_grid();
        assert_eq!(grid.num_bins(), [11usize, 11, 11]);
        assert_eq!(grid.lo_corner, [-6.0, -6.0, -6.0]);
    }

    #[test]
    fn test_coord_in_grid() {
        let grid = setup_grid();
        assert_eq!(
            grid.coord_to_index(&[1.0, 1.0, 1.0]).to_3d(),
            [3usize, 3, 3]
        );
        assert_eq!(
            grid.coord_to_index(&[-5.0, -5.0, -5.0]).to_3d(),
            [0usize, 0, 0]
        );
    }
}
