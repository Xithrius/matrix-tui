use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
};

use crate::{
    events::{Event, InternalEvent, LoginMode, Mode, SenderExt},
    ui::{component::Component, user_input::UserInputWidget},
};

pub struct PasswordPromptWidget {
    input: UserInputWidget,
    event_tx: Sender<Event>,

    password: Option<String>,
}

impl PasswordPromptWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let input = UserInputWidget::new(Some("Password"));

        Self {
            input,
            event_tx,
            password: None,
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }

    pub fn password(&self) -> Option<String> {
        self.password.clone()
    }
}

impl Component for PasswordPromptWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if !self.input.is_focused() {
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.input.set_focused(false);
                self.input.clear();
                self.event_tx
                    .send_into(InternalEvent::SwitchMode(Mode::Login(
                        LoginMode::SelectLoginChoice,
                    )))
                    .await?;
            }
            KeyCode::Enter => {
                let password = self.input.get_input();
                if password.is_empty() {
                    return Ok(());
                }
                self.password = Some(password.to_owned());

                self.input.clear();

                self.event_tx
                    .send_into(InternalEvent::SwitchMode(Mode::Login(LoginMode::Completed)))
                    .await?;
            }
            _ => {
                self.input.handle_key_event(key).await?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [_, top] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.input.draw(frame, top);
    }
}
