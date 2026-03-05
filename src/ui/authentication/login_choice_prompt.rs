use std::string::ToString;

use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tracing::debug;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, List, ListState},
};

use crate::{
    events::{Event, InternalEvent, LoginMode, Mode},
    matrix::login::LoginChoice,
    ui::component::Component,
};

pub struct LoginChoicePromptWidget {
    event_tx: Sender<Event>,
    login_choices: Vec<LoginChoice>,
    selected_login_choice: Option<LoginChoice>,

    list_state: ListState,
}

impl LoginChoicePromptWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
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
}

impl Component for LoginChoicePromptWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let index = self.list_state.selected();

        match key.code {
            KeyCode::Esc => {
                self.event_tx
                    .send(Event::Internal(InternalEvent::Quit))
                    .await?;
            }
            KeyCode::Up => {
                let index = index.unwrap_or(0).saturating_sub(1);
                self.list_state.select(Some(index));
            }
            KeyCode::Down => {
                if index.is_none() {
                    self.list_state.select(Some(0));
                    return Ok(());
                }

                let index = index.unwrap_or(0).saturating_add(1);
                self.list_state.select(Some(index));
            }
            KeyCode::Enter => {
                self.selected_login_choice = self.login_choices.get(index.unwrap_or(0)).cloned();
                debug!("Selected login choice: {:?}", self.selected_login_choice);
                self.event_tx
                    .send(Event::Internal(InternalEvent::SwitchMode(Mode::Login(
                        LoginMode::UsernamePrompt,
                    ))))
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let login_choices = self
            .login_choices
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>();
        let list = List::new(login_choices)
            .block(Block::bordered().title("Login choices"))
            .highlight_style(Style::new().reversed())
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
