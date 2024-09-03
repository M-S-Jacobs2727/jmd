use std::sync::Arc;

use crate::box_::Box_;

pub struct Bins {
    box_: Arc<Box_>,
    bin_size: f64,
}

impl Bins {
    pub fn new(bin_size: f64, box_: Arc<Box_>) -> Self {
        assert!(bin_size > 0.0, "Bin size must be positive");
        assert!(
            bin_size < (box_.lx().min(box_.ly()).min(box_.lz())),
            "Bin size must be less than the smallest box length"
        );
        Self { box_, bin_size }
    }
    pub fn total_num_bins(&self) -> usize {
        let n = self.num_bins();
        n[0] * n[1] * n[2]
    }
    pub fn num_bins(&self) -> [usize; 3] {
        [
            (self.box_.lx() / self.bin_size).ceil() as usize,
            (self.box_.ly() / self.bin_size).ceil() as usize,
            (self.box_.lz() / self.bin_size).ceil() as usize,
        ]
    }
    pub fn coord_to_bin_idx(&self, coord: &[f64; 3]) -> usize {
        let mut inds: [usize; 3] = [0, 0, 0];
        let box_lo = self.box_.lo();
        let num_bins = self.num_bins();
        for i in 0..3 {
            inds[i] = ((((coord[i] - box_lo[i]) / self.bin_size).floor() as i64)
                % num_bins[i] as i64) as usize;
        }
        inds[0] * num_bins[1] * num_bins[2] + inds[1] * num_bins[2] + inds[2]
    }
}
