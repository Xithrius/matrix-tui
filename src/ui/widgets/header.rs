use tui::{Frame, layout::Rect, style::Style, text::Span, widgets::Paragraph};

use crate::{config::CoreConfig, ui::widgets::spinner::SpinnerWidget};

#[derive(Debug, Clone)]
pub struct HeaderWidget {
    motd: String,
    mode: String,
    spinner: SpinnerWidget,
}

impl HeaderWidget {
    pub const fn new(config: &CoreConfig, motd: String) -> Self {
        Self {
            motd,
            mode: String::new(),
            spinner: SpinnerWidget::new(config.terminal.frame_rate),
        }
    }

    pub const fn increment_spinner(&mut self) {
        if self.spinner.is_active() {
            self.spinner.increment();
        }
    }

    pub fn set_mode(&mut self, mode: String) {
        self.mode = mode;
    }

    /// Enable or disable the loading spinner (e.g. during session restore).
    pub const fn set_loading(&mut self, loading: bool) {
        self.spinner.set_active(loading);
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let motd_span = Span::styled(self.motd.clone(), Style::new().dim());
        let motd = Paragraph::new(motd_span).left_aligned();
        frame.render_widget(motd, area);

        let mode = if self.spinner.is_active() {
            let spinner_state = self.spinner.state();

            format!("{} {}", spinner_state, self.mode)
        } else {
            self.mode.clone()
        };

        let mode_span = Span::styled(mode, Style::new().dim());
        let mode = Paragraph::new(mode_span).right_aligned();
        frame.render_widget(mode, area);
    }
}
