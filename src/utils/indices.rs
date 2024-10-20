use ndarray::{self, s, Array1, ArrayView1};

type I = Array1<usize>;
type IV<'a> = ArrayView1<'a, usize>;

#[derive(Clone, Debug)]
pub struct Index {
    idx: usize,
    bounds: I,
}
impl Index {
    pub fn new(dim: usize) -> Self {
        Self {
            idx: 0,
            bounds: I::zeros(dim),
        }
    }
    pub fn from_3d(indices: IV, bounds: IV) -> Self {
        assert_eq!(indices.len(), bounds.len(), "Should be the same length");
        assert!(
            indices.indexed_iter().all(|(i, idx)| *idx <= bounds[i]),
            "Out of bounds"
        );
        let idx = indices
            .indexed_iter()
            .map(|(i, idx)| *idx * bounds.slice(s![i + 1..]).product())
            .sum();
        Self {
            idx,
            bounds: bounds.to_owned(),
        }
    }
    pub fn to_3d(&self) -> I {
        let mut inds = I::zeros(self.bounds.len());
        let mut tmp = self.idx;
        for i in 1..self.bounds.len() {
            let n = self.bounds.len() - i;
            inds[n] = tmp % self.bounds[n];
            tmp /= self.bounds[n];
        }
        inds[0] = tmp;
        inds
    }
    pub fn idx(&self) -> usize {
        self.idx.clone()
    }
    pub fn bounds(&self) -> IV {
        self.bounds.view()
    }

    pub(crate) fn set_bounds(&mut self, bounds: I) {
        assert!(self.idx < bounds.product(), "Out of bounds");
        self.bounds = bounds;
    }

    pub(crate) fn set_idx(&mut self, idx: usize) {
        assert!(idx < self.bounds.product(), "Out of bounds");
        self.idx = idx
    }
}
