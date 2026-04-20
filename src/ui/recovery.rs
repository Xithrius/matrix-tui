use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Paragraph},
};

use crate::{
    events::{Event, RecoveryMode},
    matrix::event::{MatrixAction, MatrixEvent},
    ui::{component::Component, user_input::UserInputWidget},
};

/// Prompts the user to enter their existing recovery key.
struct EnterRecoveryKeyWidget {
    input: UserInputWidget,
    event_tx: Sender<Event>,
}

impl EnterRecoveryKeyWidget {
    fn new(event_tx: Sender<Event>) -> Self {
        let input = UserInputWidget::new(Some("Recovery Key"));
        Self { input, event_tx }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }
}

impl Component for EnterRecoveryKeyWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Enter => {
                let key_str = self.input.get_input();
                if key_str.is_empty() {
                    return Ok(());
                }
                let recovery_key = key_str.to_owned();
                self.input.clear();

                self.event_tx
                    .send(Event::Matrix(MatrixEvent::Action(
                        MatrixAction::ProvideRecoveryKey(recovery_key),
                    )))
                    .await?;
            }
            _ => {
                self.input.handle_key_event(key).await?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [_, input_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.input.draw(frame, input_area);
    }
}

/// Displays a newly generated recovery key that the user must record before confirming.
struct ShowKeyWidget {
    recovery_key: Option<String>,
    event_tx: Sender<Event>,
}

impl ShowKeyWidget {
    const fn new(event_tx: Sender<Event>) -> Self {
        Self {
            recovery_key: None,
            event_tx,
        }
    }

    pub fn set_key(&mut self, key: String) {
        self.recovery_key = Some(key);
    }
}

impl Component for ShowKeyWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Enter {
            self.event_tx
                .send(Event::Matrix(MatrixEvent::Action(
                    MatrixAction::ConfirmRecoveryKeySaved,
                )))
                .await?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
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
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            enter_key: EnterRecoveryKeyWidget::new(event_tx.clone()),
            show_key: ShowKeyWidget::new(event_tx),
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
}

impl Component for RecoveryWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.mode {
            RecoveryMode::EnterKey => self.enter_key.handle_key_event(key).await,
            RecoveryMode::ShowKey => self.show_key.handle_key_event(key).await,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.mode {
            RecoveryMode::EnterKey => self.enter_key.draw(frame, area),
            RecoveryMode::ShowKey => self.show_key.draw(frame, area),
        }
    }
}
