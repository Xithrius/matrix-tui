use std::fmt;

use tui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
};

use crate::ui::component::Component;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Error(String),
    Info(String),
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error(s) | Self::Info(s) => write!(f, "{s}"),
        }
    }
}

impl Status {
    const fn to_color(&self) -> Color {
        match self {
            Self::Error(_) => Color::Red,
            Self::Info(_) => Color::Blue,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusLineWidget {
    status: Status,
}

impl StatusLineWidget {
    pub const fn new(status: Status) -> Self {
        Self { status }
    }

    pub fn set_status(&mut self, status: Status) {
        self.status = status;
    }
}

impl Component for StatusLineWidget {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(self.status.to_color());

        let status_str_padded = format!(
            "{: <width$}",
            self.status.to_string(),
            width = area.width as usize
        );

        let status_span = Span::styled(status_str_padded, style);
        let status = Paragraph::new(status_span).left_aligned();
        frame.render_widget(status, area);
    }
}
