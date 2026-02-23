use color_eyre::Result;
use crossbeam_channel::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
};

use crate::{
    event::{Event, InternalEvent, Mode},
    ui::{component::Component, user_input::UserInputWidget},
};

pub struct InputWidget {
    input: UserInputWidget,
    event_tx: Sender<Event>,
}

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
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        if !self.input.is_focused() {
            return Ok(());
        }

        match key.code {
            KeyCode::Esc => {
                self.input.set_focused(false);
                self.input.clear();
                self.event_tx
                    .send(Event::Internal(InternalEvent::SwitchMode(Mode::Messages)))?;
            }
            KeyCode::Enter => {
                let message = self.input.get_input();
                self.event_tx
                    .send(Event::Internal(InternalEvent::SendMessage(
                        message.to_string(),
                    )))?;
                self.input.clear();
            }
            _ => {
                self.input.handle_key_event(key)?;
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
