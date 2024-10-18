#[derive(Clone, Copy, Debug)]
pub enum Axis {
    X,
    Y,
    Z,
}
impl Axis {
    pub fn index(&self) -> usize {
        match self {
            Axis::X => 0,
            Axis::Y => 1,
            Axis::Z => 2,
        }
    }
    pub fn direction(&self, hi: bool) -> Direction {
        match (self, hi) {
            (Axis::X, false) => Direction::Xlo,
            (Axis::X, true) => Direction::Xhi,
            (Axis::Y, false) => Direction::Ylo,
            (Axis::Y, true) => Direction::Yhi,
            (Axis::Z, false) => Direction::Zlo,
            (Axis::Z, true) => Direction::Zhi,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Xlo,
    Xhi,
    Ylo,
    Yhi,
    Zlo,
    Zhi,
}
impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Xlo => Direction::Xhi,
            Direction::Xhi => Direction::Xlo,
            Direction::Ylo => Direction::Yhi,
            Direction::Yhi => Direction::Ylo,
            Direction::Zlo => Direction::Zhi,
            Direction::Zhi => Direction::Zlo,
        }
    }
    pub fn axis(&self) -> Axis {
        match self {
            Direction::Xlo | Direction::Xhi => Axis::X,
            Direction::Ylo | Direction::Yhi => Axis::Y,
            Direction::Zlo | Direction::Zhi => Axis::Z,
        }
    }
    pub fn index(&self) -> usize {
        match self {
            Direction::Xlo => 0,
            Direction::Xhi => 1,
            Direction::Ylo => 2,
            Direction::Yhi => 3,
            Direction::Zlo => 4,
            Direction::Zhi => 5,
        }
    }
    pub fn is_lo(&self) -> bool {
        match self {
            Direction::Xlo | Direction::Ylo | Direction::Zlo => true,
            _ => false,
        }
    }
    pub fn is_hi(&self) -> bool {
        match self {
            Direction::Xlo | Direction::Ylo | Direction::Zlo => false,
            _ => true,
        }
    }
}
