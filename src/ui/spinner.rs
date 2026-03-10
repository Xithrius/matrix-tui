const SPINNERS: &[&str] = &["|", "/", "-", "\\"];

#[derive(Debug, Clone, Default)]
pub struct SpinnerWidget {
    state: usize,
    active: bool,
}

impl SpinnerWidget {
    pub const fn is_active(&self) -> bool {
        self.active
    }

    pub const fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn state(&self) -> &str {
        SPINNERS.get(self.state % SPINNERS.len()).unwrap_or(&"")
    }

    pub const fn increment(&mut self) {
        self.state = self.state.saturating_add(1);
    }
}
