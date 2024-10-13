use crate::Container;

/// Neighbor list grid of bins
pub struct Grid {
    lo_corner: [f64; 3],
    hi_corner: [f64; 3],
    bin_size: f64,
    num_bins: [usize; 3],
}
impl Grid {
    pub fn new(container: &Container, bin_size: f64, cutoff_distance: f64) -> Self {
        assert!(bin_size > 0.0, "Bin size must be positive");
        assert!(
            bin_size < 0.5 * (container.lx().min(container.ly()).min(container.lz())),
            "Bin size must be less than half the smallest box length"
        );
        let lo_corner = [
            container.xlo() - cutoff_distance,
            container.ylo() - cutoff_distance,
            container.zlo() - cutoff_distance,
        ];
        let hi_corner = [
            container.xhi() + cutoff_distance,
            container.yhi() + cutoff_distance,
            container.zhi() + cutoff_distance,
        ];
        let mut num_bins: [usize; 3] = [0, 0, 0];
        for i in 0..3 {
            num_bins[i] += (hi_corner[i].max(0.0) / bin_size) as usize;
            num_bins[i] += (-lo_corner[i].min(0.0) / bin_size) as usize;
        }
        Self {
            lo_corner,
            hi_corner,
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
    pub fn num_bins(&self) -> &[usize; 3] {
        &self.num_bins
    }
    pub fn bin_idx_to_3d_idx(&self, bin_idx: usize) -> [i32; 3] {
        assert!(
            bin_idx < self.total_num_bins(),
            "Bin index should be less than the total number of bins"
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
            "3D bin indices should be positive"
        );
        (inds[0] as usize) * self.num_bins[1] * self.num_bins[2]
            + (inds[1] as usize) * self.num_bins[2]
            + (inds[2] as usize)
    }
    pub fn coord_to_3d_idx(&self, coord: &[f64; 3]) -> [i32; 3] {
        let mut inds: [i32; 3] = [0, 0, 0];
        let num_bins = self.num_bins();
        for i in 0..3 {
            inds[i] = ((((coord[i] - self.lo_corner[i]) / self.bin_size).floor() as i32)
                % num_bins[i] as i32) as i32;
        }
        inds
    }
}
