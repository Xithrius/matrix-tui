use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    events::RecoveryMode,
    ui::{
        action::{Action, ContextKey, FocusOpts, KeyEventResult},
        context::{Context, Keybinding, ViewName},
        widgets::user_input::UserInputWidget,
    },
};

pub struct RecoveryContext {
    key_input: UserInputWidget,
    recovery_key: Option<String>,
    mode: RecoveryMode,
}

impl RecoveryContext {
    pub fn new() -> Self {
        Self {
            key_input: UserInputWidget::new(Some("Recovery Key")),
            recovery_key: None,
            mode: RecoveryMode::EnterKey,
        }
    }

    // --- Domain API for matrix notification handlers ---

    pub fn show_new_key(&mut self, key: String) {
        self.recovery_key = Some(key);
        self.mode = RecoveryMode::ShowKey;
    }
}

impl Context for RecoveryContext {
    fn key(&self) -> ContextKey {
        ContextKey::Recovery
    }

    fn view(&self) -> ViewName {
        ViewName::Fullscreen
    }

    fn is_fullscreen(&self) -> bool {
        true
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![]
    }

    fn handle_unbound_key(&mut self, key: KeyEvent) -> KeyEventResult {
        match self.mode {
            RecoveryMode::EnterKey => match key.code {
                KeyCode::Enter => {
                    let key_str = self.key_input.get_input().to_owned();
                    if key_str.is_empty() {
                        KeyEventResult::Consumed
                    } else {
                        self.key_input.clear();
                        KeyEventResult::DoAction(Action::ProvideRecoveryKey(key_str))
                    }
                }
                _ => self.key_input.handle_key(key),
            },
            RecoveryMode::ShowKey => match key.code {
                KeyCode::Enter => KeyEventResult::DoAction(Action::ConfirmRecoveryKeySaved),
                _ => KeyEventResult::NotConsumed,
            },
        }
    }

    fn on_focus(&mut self, _opts: FocusOpts) {
        self.key_input.set_focused(true);
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            RecoveryMode::EnterKey => {
                let [_, input_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(3),
                    Constraint::Fill(1),
                ])
                .areas(area);
                self.key_input.draw(frame, input_area);
            }
            RecoveryMode::ShowKey => {
                let key_display = self
                    .recovery_key
                    .as_deref()
                    .unwrap_or("Generating recovery key...");

                let [_, key_area, _] = Layout::vertical([
                    Constraint::Fill(1),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                ])
                .areas(area);

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
    }
}
