use crate::{
    region::Rect,
    utils::{Axis, Direction},
};

/// Boundary conditions for simulation box.
///
/// P: Periodic (must be set for both sides)
/// F: Fixed boundary
/// S: Shrink-wrapped boundary
/// M: Shrink-wrapped boundary with a minimum
#[derive(Debug)]
pub enum BC {
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
impl BC {
    /// Check whether the boundary condition is periodic
    pub fn is_periodic(&self) -> bool {
        match self {
            BC::PP => true,
            _ => false,
        }
    }
}

/// Simulation box, represented by a rectangular box and boundary conditions
#[derive(Debug)]
pub struct Container {
    rect: Rect,
    bc: [BC; 3],
}
impl Container {
    // Creation

    /// Create a new container from boundary values and conditions
    pub fn new(
        xlo: f64,
        xhi: f64,
        ylo: f64,
        yhi: f64,
        zlo: f64,
        zhi: f64,
        xbc: BC,
        ybc: BC,
        zbc: BC,
    ) -> Self {
        let rect = Rect::new(xlo, xhi, ylo, yhi, zlo, zhi);
        Self {
            rect,
            bc: [xbc, ybc, zbc],
        }
    }
    /// Create a fully periodic container from a given rectangular box
    pub fn from_rect_periodic(rect: Rect) -> Self {
        Self {
            rect,
            bc: [BC::PP, BC::PP, BC::PP],
        }
    }

    // Getters
    /// Check whether the boundary condition along a given axis (X, Y, Z) is periodic
    pub fn is_periodic(&self, axis: Axis) -> bool {
        self.bc[axis.index()].is_periodic()
    }
    /// A reference to the rectangular box
    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    // Setters

    pub fn set_bound(&mut self, direction: Direction, bound: f64) {
        let opposite_bound = self.rect.get_bound(direction.opposite());
        if direction.is_lo() {
            assert!(
                bound < opposite_bound,
                "Given lower bound {:?} = {} should be less than the current upper bound {:?} = {}",
                direction,
                bound,
                direction.opposite(),
                opposite_bound,
            );
        } else {
            assert!(
                bound > opposite_bound,
                "Given upper bound {:?} = {} should be less than the current lower bound {:?} = {}",
                direction,
                bound,
                direction.opposite(),
                opposite_bound,
            );
        }
        self.rect.set_bound(direction, bound);
    }
    pub fn set_boundary_condition(&mut self, axis: Axis, bc: BC) {
        self.bc[axis.index()] = bc;
    }
}
