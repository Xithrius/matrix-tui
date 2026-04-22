use tui::{
    Frame,
    crossterm::event::KeyEvent,
    layout::{Constraint, Layout, Rect},
};

use crate::ui::{action::KeyResult, user_input::UserInputWidget};

pub struct UsernamePromptWidget {
    input: UserInputWidget,
    username: Option<String>,
}

impl UsernamePromptWidget {
    pub fn new() -> Self {
        Self {
            input: UserInputWidget::new(Some("Username")),
            username: None,
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }

    pub fn username(&self) -> Option<String> {
        self.username.clone()
    }

    pub fn has_input(&self) -> bool {
        !self.input.get_input().is_empty()
    }

    /// Store the current input as the confirmed username and clear the field.
    pub fn confirm(&mut self) {
        let value = self.input.get_input().to_owned();
        if !value.is_empty() {
            self.username = Some(value);
        }
        self.input.clear();
    }

    pub fn clear(&mut self) {
        self.username = None;
        self.input.clear();
    }

    /// Delegate text-input keys to the inner widget.
    pub fn handle_text_key(&mut self, key: KeyEvent) -> KeyResult {
        self.input.handle_key(key)
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let [_, top] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.input.draw(frame, top);
    }
}
