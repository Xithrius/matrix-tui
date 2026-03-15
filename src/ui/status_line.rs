use std::{
    fmt,
    time::{Duration, Instant},
};

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
    status: Option<Status>,
    expiration_time: Option<Instant>,
}

fn timeout_s_to_expiration(timeout_s: u64) -> Instant {
    Instant::now() + Duration::from_secs(timeout_s)
}

impl StatusLineWidget {
    pub fn new(status: Option<Status>, timeout_s: Option<u64>) -> Self {
        Self {
            status,
            expiration_time: timeout_s.map(timeout_s_to_expiration),
        }
    }

    pub fn set_status(&mut self, status: Status, timeout_s: Option<u64>) {
        self.status = Some(status);
        self.expiration_time = timeout_s.map(timeout_s_to_expiration);
    }
}

impl Component for StatusLineWidget {
    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let Some(status) = &self.status else {
            return;
        };

        let style = Style::new().bg(status.to_color());

        let status_str_padded = format!(
            "{: <width$}",
            status.to_string(),
            width = area.width as usize
        );

        let status_span = Span::styled(status_str_padded, style);
        let status = Paragraph::new(status_span).left_aligned();
        frame.render_widget(status, area);
    }
}
