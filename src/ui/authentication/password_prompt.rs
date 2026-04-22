use tui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
};

use crate::ui::{action::KeyResult, user_input::UserInputWidget};

pub struct PasswordPromptWidget {
    input: UserInputWidget,
    password: Option<String>,
}

impl PasswordPromptWidget {
    pub fn new() -> Self {
        Self {
            input: UserInputWidget::new(Some("Password")),
            password: None,
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }

    pub fn password(&self) -> Option<String> {
        self.password.clone()
    }

    pub fn has_input(&self) -> bool {
        !self.input.get_input().is_empty()
    }

    pub fn confirm(&mut self) {
        let value = self.input.get_input().to_owned();
        if !value.is_empty() {
            self.password = Some(value);
        }
        self.input.clear();
    }

    pub fn clear(&mut self) {
        self.password = None;
        self.input.clear();
    }

    pub fn handle_text_key(&mut self, key: KeyEvent) -> KeyResult {
        self.input.handle_key(key)
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let [_, top] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.input.draw(frame, top);
    }
}
