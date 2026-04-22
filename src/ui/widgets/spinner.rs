const SPINNERS: &[&str] = &["⠧", "⠏", "⠛", "⠹", "⠼", "⠶"];

const SPINNER_SECONDS_PER_REVOLUTION: f32 = 0.5;

#[derive(Debug, Clone, Default)]
pub struct SpinnerWidget {
    state: usize,
    active: bool,
    frame_duration: u64,
    tick_counter: u64,
}

impl SpinnerWidget {
    pub const fn new(fps: u64) -> Self {
        let frame_duration =
            ((SPINNER_SECONDS_PER_REVOLUTION * fps as f32) / SPINNERS.len() as f32) as u64;

        Self {
            state: 0,
            active: false,
            frame_duration,
            tick_counter: 0,
        }
    }

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
        self.tick_counter = self.tick_counter.wrapping_add(1);
        if self.tick_counter >= self.frame_duration {
            self.tick_counter = 0;
            self.state = self.state.saturating_add(1);
        }
    }
}
