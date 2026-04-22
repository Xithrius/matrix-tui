use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Paragraph},
};

use super::user_input::UserInputWidget;
use crate::{
    events::RecoveryMode,
    ui::action::{Action, KeyResult},
};

/// Prompts the user to enter their existing recovery key.
struct EnterRecoveryKeyWidget {
    input: UserInputWidget,
}

impl EnterRecoveryKeyWidget {
    fn new() -> Self {
        Self {
            input: UserInputWidget::new(Some("Recovery Key")),
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }

    fn get_input(&self) -> &str {
        self.input.get_input()
    }

    fn clear(&mut self) {
        self.input.clear();
    }

    /// Delegate text-input keys to the inner widget.
    fn handle_text_key(&mut self, key: KeyEvent) -> KeyResult {
        self.input.handle_key(key)
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let [_, input_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);
        self.input.draw(frame, input_area);
    }
}

/// Displays a newly generated recovery key that the user must record before confirming.
struct ShowKeyWidget {
    recovery_key: Option<String>,
}

impl ShowKeyWidget {
    const fn new() -> Self {
        Self { recovery_key: None }
    }

    fn set_key(&mut self, key: String) {
        self.recovery_key = Some(key);
    }

    fn draw(&self, frame: &mut Frame, area: Rect) {
        let key_display = self
            .recovery_key
            .as_deref()
            .unwrap_or("Generating recovery key...");

        let [_, key_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(5)]).areas(area);

        let paragraph = Paragraph::new(format!(
            "{key_display}\n\nPress Enter to confirm you have saved this key."
        ))
        .block(
            Block::bordered()
                .title("Save Your Recovery Key")
                .border_type(BorderType::Rounded),
        )
        .wrap(tui::widgets::Wrap { trim: false });

        frame.render_widget(paragraph, key_area);
    }
}

pub struct RecoveryWidget {
    enter_key: EnterRecoveryKeyWidget,
    show_key: ShowKeyWidget,
    mode: RecoveryMode,
}

impl RecoveryWidget {
    pub fn new() -> Self {
        Self {
            enter_key: EnterRecoveryKeyWidget::new(),
            show_key: ShowKeyWidget::new(),
            mode: RecoveryMode::EnterKey,
        }
    }

    pub const fn set_recovery_mode(&mut self, mode: RecoveryMode) {
        match self.mode {
            RecoveryMode::EnterKey => self.enter_key.set_focused(false),
            RecoveryMode::ShowKey => {}
        }

        match mode {
            RecoveryMode::EnterKey => self.enter_key.set_focused(true),
            RecoveryMode::ShowKey => {}
        }

        self.mode = mode;
    }

    pub fn set_recovery_key(&mut self, key: String) {
        self.show_key.set_key(key);
    }

    /// Synchronous key handler. Returns `DoAction(ProvideRecoveryKey(key))`
    /// when the user submits the entry key, or `DoAction(ConfirmRecoveryKeySaved)`
    /// on the show-key screen.
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self.mode {
            RecoveryMode::EnterKey => match key.code {
                KeyCode::Enter => {
                    let key_str = self.enter_key.get_input().to_owned();
                    if key_str.is_empty() {
                        KeyResult::Consumed
                    } else {
                        self.enter_key.clear();
                        KeyResult::DoAction(Action::ProvideRecoveryKey(key_str))
                    }
                }
                _ => self.enter_key.handle_text_key(key),
            },
            RecoveryMode::ShowKey => match key.code {
                KeyCode::Enter => KeyResult::DoAction(Action::ConfirmRecoveryKeySaved),
                _ => KeyResult::NotConsumed,
            },
        }
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        match self.mode {
            RecoveryMode::EnterKey => self.enter_key.draw(frame, area),
            RecoveryMode::ShowKey => self.show_key.draw(frame, area),
        }
    }
}
