use crate::constraint::Constraint;
use crate::system::System;

pub struct Engine {
    constraints: Vec<Box<dyn Constraint>>,
}

impl Engine {
    /// Adds a force to the engine.
    ///
    /// # Arguments
    ///
    /// * `force` - A force to be added to the engine.
    pub fn add_constraint(&mut self, force: Box<dyn Constraint>) {
        self.constraints.push(force);
    }
    /// Runs the engine for a specified number of steps.
    ///
    /// # Arguments
    ///
    /// * `steps` - The number of steps to run the engine for.
    pub fn run(&mut self, steps: usize, system: &mut System) {
        self.forward_communication(system);
        self.build_neighbor_list(0, system);
        self.compute_force(system);
        self.reverse_communication(system);
        self.output(0, system);
        for step in 1..=steps {
            self.pre_forward_communication(system);
            self.forward_communication(system);
            self.post_forward_communication(system);

            self.build_neighbor_list(step, system);

            self.pre_compute_force(system);
            self.compute_force(system);

            self.pre_reverse_communication(system);
            self.reverse_communication(system);
            self.post_reverse_communication(system);

            self.output(step, system);
        }
    }
    /// Iterates velocities and positions before forward communication.
    /// Also includes other events occuring before forward communication.
    fn pre_forward_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.pre_forward_communication(system);
        }
    }
    /// Forward communication of positions and velocities.
    fn forward_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.forward_communication(system);
        }
    }
    /// Events occuring after position and velocity communication.
    fn post_forward_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.post_forward_communication(system);
        }
    }
    /// Builds the neighbor list if the step is a multiple of the update frequency.
    fn build_neighbor_list(&mut self, step: usize, system: &mut System) {
        if step % system.neighbor_list.settings.update_every == 0 {
            system.generate_neighbor_list();
        }
    }
    /// Events occuring before force computation.
    fn pre_compute_force(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.pre_compute_force(system);
        }
    }
    /// Iterates velocities before reverse communication.
    /// Also includes other events occuring before reverse communication.
    fn pre_reverse_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.pre_reverse_communication(system);
        }
    }
    /// Reverse communication of forces.
    fn reverse_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.reverse_communication(system);
        }
    }
    /// Events occuring after reverse communication.
    fn post_reverse_communication(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.post_reverse_communication(system);
        }
    }
    fn compute_force(&mut self, system: &mut System) {
        for constraint in &mut self.constraints {
            constraint.compute_force(system);
        }
    }
    fn output(&self, step: usize, system: &mut System) {}
}
