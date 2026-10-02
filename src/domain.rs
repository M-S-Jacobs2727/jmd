use crate::region::Rect;

#[derive(Debug, Clone, Copy)]
pub struct Domain {
    region: Rect,
    boundary_conditions: [BoundaryCondition; 3],
}
#[derive(Debug, Clone, Copy)]
pub enum BoundaryCondition {
    Periodic,
    Fixed(f64, f64),
    ShrinkWrapped,
    MinimumShrinkWrapped(f64, f64),
}

impl Domain {
    pub fn new(region: Rect, boundary_conditions: [BoundaryCondition; 3]) -> Self {
        for boundary_condition in boundary_conditions {
            if let BoundaryCondition::Fixed(min, max) = boundary_condition {
                if min >= max {
                    panic!("Invalid boundary condition: min >= max");
                }
            }
            if let BoundaryCondition::MinimumShrinkWrapped(min, max) = boundary_condition {
                if min >= max {
                    panic!("Invalid boundary condition: min >= max");
                }
            }
        }
        Self {
            region,
            boundary_conditions,
        }
    }
    pub fn region(&self) -> &Rect {
        &self.region
    }
    pub fn boundary_conditions(&self) -> &[BoundaryCondition; 3] {
        &self.boundary_conditions
    }
    pub fn is_periodic(&self, direction: usize) -> bool {
        if direction > 2 {
            panic!("Invalid direction: {}", direction);
        }
        self.boundary_conditions[direction].is_periodic()
    }
    pub fn xlo(&self) -> f64 {
        self.region.xlo()
    }
    pub fn ylo(&self) -> f64 {
        self.region.ylo()
    }
    pub fn zlo(&self) -> f64 {
        self.region.zlo()
    }
    pub fn xhi(&self) -> f64 {
        self.region.xhi()
    }
    pub fn yhi(&self) -> f64 {
        self.region.yhi()
    }
    pub fn zhi(&self) -> f64 {
        self.region.zhi()
    }
    pub fn lx(&self) -> f64 {
        self.region.lx()
    }
    pub fn ly(&self) -> f64 {
        self.region.ly()
    }
    pub fn lz(&self) -> f64 {
        self.region.lz()
    }
}

impl BoundaryCondition {
    pub fn is_periodic(&self) -> bool {
        matches!(self, BoundaryCondition::Periodic)
    }
}
