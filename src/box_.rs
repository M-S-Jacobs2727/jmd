pub enum PBC {
    PP,
    FF,
    FM,
    FS,
    MF,
    MM,
    MS,
    SF,
    SM,
    SS,
}
impl PBC {
    pub fn is_periodic(&self) -> bool {
        match self {
            PBC::PP => true,
            _ => false,
        }
    }
}

pub struct Bounds {
    lo: f64,
    hi: f64,
    pbc: PBC,
}
impl Bounds {
    pub fn new(lo: f64, hi: f64, pbc: PBC) -> Self {
        if hi <= lo {
            panic!("Lower bounds should be less than upper bounds");
        }
        Self { lo, hi, pbc }
    }
    pub fn lo(&self) -> f64 {
        self.lo
    }
    pub fn hi(&self) -> f64 {
        self.hi
    }
    pub fn pbc(&self) -> &PBC {
        &self.pbc
    }
}

pub struct Box_ {
    x: Bounds,
    y: Bounds,
    z: Bounds,
}

impl Box_ {
    pub fn new(
        xlo: f64,
        xhi: f64,
        ylo: f64,
        yhi: f64,
        zlo: f64,
        zhi: f64,
        xpbc: PBC,
        ypbc: PBC,
        zpbc: PBC,
    ) -> Self {
        Self {
            x: Bounds {
                lo: xlo,
                hi: xhi,
                pbc: xpbc,
            },
            y: Bounds {
                lo: ylo,
                hi: yhi,
                pbc: ypbc,
            },
            z: Bounds {
                lo: zlo,
                hi: zhi,
                pbc: zpbc,
            },
        }
    }
    pub fn lx(&self) -> f64 {
        self.x.lo - self.x.hi
    }
    pub fn ly(&self) -> f64 {
        self.y.lo - self.y.hi
    }
    pub fn lz(&self) -> f64 {
        self.z.lo - self.z.hi
    }
}
