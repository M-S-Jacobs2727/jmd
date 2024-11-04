use super::*;

#[derive(Clone, Copy, Debug)]
pub struct Basic {
    mass: f64,
}
impl Basic {
    pub fn new(mass: f64) -> Self {
        assert!(mass > 0.0, "Mass should be positive");
        Self { mass }
    }
}
impl AtomType for Basic {
    fn mass(&self) -> f64 {
        self.mass
    }
}
