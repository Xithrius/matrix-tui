use tui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::Style,
    text::Span,
    widgets::Paragraph,
};

use crate::ui::component::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderWidget {
    motd: String,
}

impl HeaderWidget {
    pub const fn new(motd: String) -> Self {
        Self { motd }
    }
}

impl Component for HeaderWidget {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [top] = Layout::vertical([Constraint::Length(1)]).areas(area);

        let motd_span = Span::styled(self.motd.clone(), Style::new().dim());
        let motd = Paragraph::new(motd_span).left_aligned();
        frame.render_widget(motd, top);
    }
}
