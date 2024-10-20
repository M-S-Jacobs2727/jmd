use ndarray::{Array1, ArrayView1, Axis};

use crate::{utils::indices::Index, Container};

/// Neighbor list grid of bins
#[derive(Debug)]
pub struct Grid {
    lo_corner: ndarray::Array1<f64>,
    bin_size: f64,
    num_bins: ndarray::Array1<usize>,
}
impl Grid {
    pub fn new(container: &Container, bin_size: f64, cutoff_distance: f64) -> Self {
        assert!(
            bin_size > 0.0,
            "Bin size should be positive, found {}",
            bin_size
        );
        let min_box_length = container.lx().min(container.ly()).min(container.lz());
        assert!(
            bin_size < 0.5 * min_box_length,
            "Bin size must be less than half the smallest box length, \
             found bin_size {} and smallest box length {}",
            bin_size,
            min_box_length
        );
        let lo_corner = ndarray::arr1(&[
            container.xlo() - cutoff_distance,
            container.ylo() - cutoff_distance,
            container.zlo() - cutoff_distance,
        ]);
        let num_bins = ndarray::arr1(&[
            ((container.lx() + 2.0 * cutoff_distance) / bin_size).ceil() as usize,
            ((container.ly() + 2.0 * cutoff_distance) / bin_size).ceil() as usize,
            ((container.lz() + 2.0 * cutoff_distance) / bin_size).ceil() as usize,
        ]);
        Self {
            lo_corner,
            bin_size,
            num_bins,
        }
    }
    pub fn bin_size(&self) -> f64 {
        self.bin_size
    }
    pub fn total_num_bins(&self) -> usize {
        self.num_bins[0] * self.num_bins[1] * self.num_bins[2]
    }
    pub fn lo_corner<'a>(&'a self) -> ArrayView1<'a, f64> {
        self.lo_corner.view()
    }
    pub fn hi_corner(&self) -> Array1<f64> {
        &self.lo_corner + self.num_bins().mapv(|n| self.bin_size * n as f64)
    }
    pub fn num_bins<'a>(&'a self) -> ArrayView1<'a, usize> {
        self.num_bins.view()
    }
    pub fn bin_idx_to_3d_idx(&self, bin_idx: usize) -> [i32; 3] {
        assert!(
            bin_idx < self.total_num_bins(),
            "Bin index ({}) should be less than the total number of bins ({})",
            bin_idx,
            self.total_num_bins()
        );
        [
            (bin_idx / (self.num_bins[1] * self.num_bins[2])) as i32,
            (bin_idx / self.num_bins[2]) as i32,
            (bin_idx % self.num_bins[2]) as i32,
        ]
    }
    pub fn bin_idx_from_3d_idx(&self, inds: &[i32; 3]) -> usize {
        assert!(
            inds[0] >= 0 && inds[1] >= 0 && inds[2] >= 0,
            "3D bin indices ({}, {}, {}) should be positive",
            inds[0],
            inds[1],
            inds[2]
        );
        (inds[0] as usize) * self.num_bins[1] * self.num_bins[2]
            + (inds[1] as usize) * self.num_bins[2]
            + (inds[2] as usize)
    }
    pub fn coords_to_linear_indices<'a>(&self, coords: &ndarray::Array2<f64>) -> Array1<usize> {
        let num_bins = self.num_bins();
        assert_eq!(coords.len_of(Axis(1)), self.lo_corner.len_of(Axis(0)));

        let x = ((coords - &self.lo_corner) / self.bin_size).floor();
        let mut y: ndarray::Array2<usize> = ndarray::Array2::zeros(x.dim());
        ndarray::azip!((index (i, j), &x in &x) {y[[i, j]] = (x as usize) % num_bins[j]});

        y.axis_iter(Axis(0))
            .map(|yi| Index::from_3d(yi, self.num_bins()).idx())
            .collect()
    }
}
