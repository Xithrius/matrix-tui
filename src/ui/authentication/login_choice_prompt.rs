use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, List, ListState},
};

use crate::{matrix::login::LoginChoice, ui::action::KeyResult};

pub struct LoginChoicePromptWidget {
    login_choices: Vec<LoginChoice>,
    selected_login_choice: Option<LoginChoice>,
    list_state: ListState,
}

impl LoginChoicePromptWidget {
    pub fn new() -> Self {
        Self {
            login_choices: Vec::default(),
            selected_login_choice: None,
            list_state: ListState::default(),
        }
    }

    pub fn set_login_choices(&mut self, login_choices: Vec<LoginChoice>) {
        self.login_choices = login_choices;
        self.list_state.select_first();
    }

    pub fn selected_login_choice(&self) -> Option<LoginChoice> {
        self.selected_login_choice.clone()
    }

    /// Confirm the currently highlighted choice and store it.
    pub fn confirm_selection(&mut self) {
        let index = self.list_state.selected().unwrap_or(0);
        self.selected_login_choice = self.login_choices.get(index).cloned();
    }

    /// Handle list navigation keys. Returns `Consumed` or `NotConsumed`.
    pub fn handle_nav_key(&mut self, key: KeyEvent) -> KeyResult {
        let len = self.login_choices.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                let idx = self.list_state.selected().unwrap_or(0).saturating_sub(1);
                self.list_state.select(Some(idx));
                KeyResult::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if len == 0 {
                    return KeyResult::Consumed;
                }
                let idx = self
                    .list_state
                    .selected()
                    .unwrap_or(0)
                    .saturating_add(1)
                    .min(len - 1);
                self.list_state.select(Some(idx));
                KeyResult::Consumed
            }
            _ => KeyResult::NotConsumed,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let login_choices = self
            .login_choices
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();

        let block = Block::bordered()
            .title("Login choices")
            .border_type(BorderType::Rounded);
        let list = List::new(login_choices)
            .block(block)
            .highlight_style(Style::new().reversed())
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
