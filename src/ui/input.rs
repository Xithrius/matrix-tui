use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
};

use crate::{
    events::{Event, InternalEvent, Mode, SenderExt},
    ui::{component::Component, user_input::UserInputWidget},
};

pub struct InputWidget {
    input: UserInputWidget,
    event_tx: Sender<Event>,
}

// TODO: Rename to something better
impl InputWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let input = UserInputWidget::new(Some("Input"));

        Self { input, event_tx }
    }

    pub const fn set_focused(&mut self, focused: bool) {
        self.input.set_focused(focused);
    }
}

impl Component for InputWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if !self.input.is_focused() {
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.input.set_focused(false);
                self.input.clear();
                self.event_tx
                    .send_into(InternalEvent::SwitchMode(Mode::Messages))
                    .await?;
            }
            KeyCode::Enter => {
                let message = self.input.get_input();
                self.event_tx
                    .send_into(InternalEvent::SendMessage(message.to_string()))
                    .await?;
                self.input.clear();
            }
            _ => {
                self.input.handle_key_event(key).await?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.input.draw(frame, area);
    }
}
