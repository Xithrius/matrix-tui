use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, List, ListState, Row, Table, TableState},
};

use crate::{
    events::{Event, InternalEvent, LoginMode, Mode},
    matrix::message::MatrixMessage,
    ui::component::Component,
};

pub struct AuthenticationWidget {
    event_tx: Sender<Event>,
    list_state: ListState,
    login_mode: LoginMode,
    selected_login_mode: Option<LoginMode>,
}

impl AuthenticationWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            list_state: ListState::default(),
            login_mode: LoginMode::default(),
            selected_login_mode: None,
        }
    }

    pub fn set_login_mode(&mut self, mode: LoginMode) {
        self.login_mode = mode;
    }

    pub async fn handle_login_choice_key_events(&mut self, key: KeyEvent) -> Result<()> {
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
                todo!()
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_username_prompt_key_events(&mut self, key: KeyEvent) -> Result<()> {
        Ok(())
    }

    pub async fn handle_password_prompt_key_events(&mut self, key: KeyEvent) -> Result<()> {
        Ok(())
    }
}

impl Component for AuthenticationWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.login_mode {
            LoginMode::SelectLoginChoice => self.handle_login_choice_key_events(key).await,
            LoginMode::UsernamePrompt => self.handle_username_prompt_key_events(key).await,
            LoginMode::PasswordPrompt => self.handle_password_prompt_key_events(key).await,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        // let [top] = Layout::vertical([Constraint::Percentage(100)]).areas(area);

        let items = ["Item 1", "Item 2", "Item 3"];
        let list = List::new(items)
            .block(Block::bordered().title("List"))
            .highlight_style(Style::new().reversed())
            .repeat_highlight_symbol(true);

        frame.render_stateful_widget(list, area, &mut self.list_state);

        // let rows: Vec<Row> = self
        //     .messages
        //     .iter()
        //     .map(|message| {
        //         Row::new(vec![
        //             // TODO: More attributes of the message
        //             Cell::from(message.name.clone()),
        //             Cell::from(message.content.clone()),
        //         ])
        //     })
        //     .collect();
        // let widths = [Constraint::Length(20), Constraint::Percentage(100)];
        // let table = Table::new(rows, widths)
        //     .block(
        //         Block::new()
        //             .title("Messages")
        //             .borders(Borders::ALL)
        //             .border_type(BorderType::Rounded),
        //     )
        //     .row_highlight_style(Style::new().reversed());

        // frame.render_stateful_widget(table, top, &mut self.table_state);
    }
}
