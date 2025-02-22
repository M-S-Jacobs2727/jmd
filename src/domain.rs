#[derive(Debug, Clone, Copy)]
pub struct Domain {
    xlo: f64,
    xhi: f64,
    ylo: f64,
    yhi: f64,
    zlo: f64,
    zhi: f64,
}

impl Domain {
    pub fn new(xlo: f64, xhi: f64, ylo: f64, yhi: f64, zlo: f64, zhi: f64) -> Self {
        if xlo >= xhi || ylo >= yhi || zlo >= zhi {
            panic!("Invalid domain bounds");
        }
        Self {
            xlo,
            xhi,
            ylo,
            yhi,
            zlo,
            zhi,
        }
    }
    pub fn contains(&self, position: [f64; 3]) -> bool {
        position[0] >= self.xlo
            && position[0] <= self.xhi
            && position[1] >= self.ylo
            && position[1] <= self.yhi
            && position[2] >= self.zlo
            && position[2] <= self.zhi
    }

    pub fn set_bounds(&mut self, vec: Vec<(f64, f64)>) {
        self.xlo = vec[0].0;
        self.xhi = vec[0].1;
        self.ylo = vec[1].0;
        self.yhi = vec[1].1;
        self.zlo = vec[2].0;
        self.zhi = vec[2].1;
    }
    pub fn lx(&self) -> f64 {
        self.xhi - self.xlo
    }
    pub fn ly(&self) -> f64 {
        self.yhi - self.ylo
    }
    pub fn lz(&self) -> f64 {
        self.zhi - self.zlo
    }
    pub fn xlo(&self) -> f64 {
        self.xlo
    }
    pub fn xhi(&self) -> f64 {
        self.xhi
    }
    pub fn ylo(&self) -> f64 {
        self.ylo
    }
    pub fn yhi(&self) -> f64 {
        self.yhi
    }
    pub fn zlo(&self) -> f64 {
        self.zlo
    }
    pub fn zhi(&self) -> f64 {
        self.zhi
    }
}
