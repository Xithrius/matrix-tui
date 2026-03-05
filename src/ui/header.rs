use tui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::Paragraph,
};

use crate::{events::Mode, ui::component::Component};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderWidget {
    motd: String,
    mode: Mode,
}

impl HeaderWidget {
    pub const fn new(motd: String, mode: Mode) -> Self {
        Self { motd, mode }
    }

    pub const fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }
}

impl Component for HeaderWidget {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let motd_span = Span::styled(self.motd.clone(), Style::new().dim());
        let motd = Paragraph::new(motd_span).left_aligned();
        frame.render_widget(motd, area);

        let mode_span = Span::styled(self.mode.to_string(), Style::new().dim());
        let mode = Paragraph::new(mode_span).right_aligned();
        frame.render_widget(mode, area);
    }
}
