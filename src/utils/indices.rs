#[derive(Clone, Copy, Debug)]
pub struct Index {
    idx: usize,
    bounds: [usize; 3],
}
impl Index {
    pub fn new() -> Self {
        Self {
            idx: 0,
            bounds: [0, 0, 0],
        }
    }
    pub fn from_3d(indices: &[usize; 3], bounds: &[usize; 3]) -> Self {
        let [x, y, z] = *indices;
        let [nx, ny, nz] = *bounds;
        if x >= nx || y >= ny || z >= nz {
            panic!("Out of bounds");
        }
        let idx = x * bounds[1] * bounds[2] + y * bounds[2] + z;
        Self {
            idx,
            bounds: [nx, ny, nz],
        }
    }
    pub fn to_3d(&self) -> [usize; 3] {
        let z = self.idx % self.bounds[2];
        let q = self.idx / self.bounds[2];
        let y = q % self.bounds[1];
        let x = q / self.bounds[1];
        [x, y, z]
    }
    pub fn idx(&self) -> usize {
        self.idx
    }
    pub fn bounds(&self) -> [usize; 3] {
        self.bounds
    }

    pub(crate) fn set_bounds(&mut self, bounds: [usize; 3]) {
        if self.idx >= bounds[0] * bounds[1] * bounds[2] {
            panic!("Out of bounds");
        }
        self.bounds = bounds;
    }

    pub(crate) fn set_idx(&mut self, idx: usize) {
        if idx > self.bounds[0] * self.bounds[1] * self.bounds[2] {
            panic!("Out of bounds");
        }
        self.idx = idx
    }
}
