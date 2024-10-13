/// Settings for computing and updating a neighbor list
pub struct UpdateSettings {
    pub every: usize,
    pub delay: usize,
    pub check: bool,
    last_update_step: usize,
}

impl UpdateSettings {
    pub fn new() -> Self {
        Self {
            every: 1,
            delay: 0,
            check: true,
            last_update_step: 0,
        }
    }
    pub fn should_update_neighbors(&self, step: usize) -> bool {
        (step % self.every == 0) && (step - self.last_update_step >= self.delay)
    }
}
