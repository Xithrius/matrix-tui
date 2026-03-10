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

pub struct UsernamePromptWidget {
    input: UserInputWidget,
    event_tx: Sender<Event>,

    username: Option<String>,
}

impl UsernamePromptWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let input = UserInputWidget::new(Some("Username"));

        Self {
            input,
            event_tx,
            username: None,
        }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }

    pub fn username(&self) -> Option<String> {
        self.username.clone()
    }
}

impl Component for UsernamePromptWidget {
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
                let username = self.input.get_input();
                if username.is_empty() {
                    return Ok(());
                }
                self.username = Some(username.to_owned());

                self.input.clear();

                self.event_tx
                    .send_into(InternalEvent::SwitchMode(Mode::Login(
                        LoginMode::PasswordPrompt,
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
        let [_, top] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.input.draw(frame, top);
    }
}
