use crate::region::Region;

use super::Lattice;

#[derive(Debug)]
pub struct Cubic {
    a: f64,
}
impl Cubic {
    pub fn new(a: f64) -> Self {
        let s = Self { a };
        s.assert_positive();
        s
    }
    pub fn from_density(rho: f64) -> Self {
        let s = Self {
            a: (1.0 / rho).cbrt(),
        };
        s.assert_positive();
        s
    }
    fn assert_positive(&self) {
        assert!(
            self.a > 0.0,
            "Lattice constant should be positive, found {}",
            self.a
        );
    }
}
impl Lattice for Cubic {
    fn cell_lengths(&self) -> [f64; 3] {
        [self.a, self.a, self.a]
    }
    fn coords_within_region<R: Region>(&self, region: &R, origin: &[f64; 3]) -> Vec<[f64; 3]> {
        let bounding_box = region.bounding_box();
        let bblo = bounding_box.lo();
        let bbhi = bounding_box.hi();
        let lo = [
            ((bblo[0] - origin[0]) / self.a).floor() * self.a,
            ((bblo[1] - origin[1]) / self.a).floor() * self.a,
            ((bblo[2] - origin[2]) / self.a).floor() * self.a,
        ];
        let nlattice = [
            ((bbhi[0] - lo[0]) / self.a).floor() as usize,
            ((bbhi[1] - lo[1]) / self.a).floor() as usize,
            ((bbhi[2] - lo[2]) / self.a).floor() as usize,
        ];
        let mut coords: Vec<[f64; 3]> = Vec::new();
        coords.reserve(nlattice[0] * nlattice[1] * nlattice[2]);

        for i in 0..nlattice[0] {
            for j in 0..nlattice[1] {
                for k in 0..nlattice[2] {
                    coords.push([
                        lo[0] + self.a * i as f64,
                        lo[1] + self.a * j as f64,
                        lo[2] + self.a * k as f64,
                    ]);
                }
            }
        }
        coords
    }
}
