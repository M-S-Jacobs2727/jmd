// TODO: add is_periodic function to Box, taking NeighborDirection
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
    // pub fn subdomain(&self, distribution_info: &DistributionInfo) -> Rect {
    //     let lx = self.lx() / (distribution_info.proc_dimensions()[0] as f64);
    //     let ly = self.ly() / (distribution_info.proc_dimensions()[1] as f64);
    //     let lz = self.lz() / (distribution_info.proc_dimensions()[2] as f64);
    //     let me = distribution_info.me();
    //     Rect::new(
    //         lx * (me[0] as f64),
    //         lx * (me[0] as f64 + 1f64),
    //         ly * (me[1] as f64),
    //         ly * (me[1] as f64 + 1f64),
    //         lz * (me[2] as f64),
    //         lz * (me[2] as f64 + 1f64),
    //     )
    // }
    pub fn x(&self) -> &Bounds {
        &self.x
    }
    pub fn y(&self) -> &Bounds {
        &self.y
    }
    pub fn z(&self) -> &Bounds {
        &self.z
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
    pub fn lo(&self) -> [f64; 3] {
        [self.x.lo, self.y.lo, self.z.lo]
    }
    pub fn hi(&self) -> [f64; 3] {
        [self.x.hi, self.y.hi, self.z.hi]
    }
    pub fn xlo(&self) -> f64 {
        self.x.lo
    }
    pub fn xhi(&self) -> f64 {
        self.x.hi
    }
    pub fn ylo(&self) -> f64 {
        self.y.lo
    }
    pub fn yhi(&self) -> f64 {
        self.y.hi
    }
    pub fn zlo(&self) -> f64 {
        self.z.lo
    }
    pub fn zhi(&self) -> f64 {
        self.z.hi
    }
}
