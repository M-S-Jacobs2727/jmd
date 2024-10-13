use crate::Error;

/// Three-dimensional index, convertible to a one-dimensional index
pub struct Index3D {
    x: usize,
    y: usize,
    z: usize,
}
impl Index3D {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y, z }
    }
    pub fn from_arr(idx: [usize; 3]) -> Self {
        Self {
            x: idx[0],
            y: idx[1],
            z: idx[2],
        }
    }
    pub fn to_1d(&self, lengths: [usize; 3]) -> Result<Index1D, Error> {
        if self.x < lengths[0] && self.y < lengths[1] && self.z < lengths[2] {
            Ok(Index1D::new(
                self.x * lengths[1] * lengths[2] + self.y * lengths[2] + self.z,
            ))
        } else {
            Err(Error::OtherError)
        }
    }
}
/// One-dimensional index, convertible to a three-dimensional index
pub struct Index1D {
    i: usize,
}
impl Index1D {
    pub fn new(i: usize) -> Self {
        Self { i }
    }
    pub fn to_3d(&self, lengths: [usize; 3]) -> Result<Index3D, Error> {
        if self.i < lengths[0] * lengths[1] * lengths[2] {
            let z = self.i % lengths[2];
            let r = self.i / lengths[2];
            let y = r % lengths[1];
            let x = r / lengths[1];
            Ok(Index3D::new(x, y, z))
        } else {
            Err(Error::OtherError)
        }
    }
}
