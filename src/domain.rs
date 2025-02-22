#[derive(Debug, Clone, Copy)]
pub struct Domain {
    pub xlo: f64,
    pub xhi: f64,
    pub ylo: f64,
    pub yhi: f64,
    pub zlo: f64,
    pub zhi: f64,
}

impl Domain {
    pub fn new() -> Self {
        Self {
            xlo: 0.0,
            xhi: 0.0,
            ylo: 0.0,
            yhi: 0.0,
            zlo: 0.0,
            zhi: 0.0,
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
}
