use tui::{Frame, layout::Rect, style::Style, text::Span, widgets::Paragraph};

use crate::{
    config::CoreConfig,
    events::Mode,
    ui::{component::Component, spinner::SpinnerWidget},
};

#[derive(Debug, Clone)]
pub struct HeaderWidget {
    motd: String,
    mode: Mode,
    spinner: SpinnerWidget,
}

impl HeaderWidget {
    pub const fn new(config: &CoreConfig, motd: String, mode: Mode) -> Self {
        Self {
            motd,
            mode,
            spinner: SpinnerWidget::new(config.terminal.frame_rate),
        }
    }

    pub const fn increment_spinner(&mut self) {
        if self.spinner.is_active() {
            self.spinner.increment();
        }
    }

    pub const fn set_mode(&mut self, mode: Mode) {
        if matches!(mode, Mode::RestoringSession) {
            self.spinner.set_active(true);
        } else {
            self.spinner.set_active(false);
        }

        self.mode = mode;
    }
}

impl Component for HeaderWidget {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let motd_span = Span::styled(self.motd.clone(), Style::new().dim());
        let motd = Paragraph::new(motd_span).left_aligned();
        frame.render_widget(motd, area);

        let mode = if self.spinner.is_active() {
            let spinner_state = self.spinner.state();

            format!("{} {}", spinner_state, self.mode)
        } else {
            self.mode.to_string()
        };

        let mode_span = Span::styled(mode, Style::new().dim());
        let mode = Paragraph::new(mode_span).right_aligned();
        frame.render_widget(mode, area);
    }
}
