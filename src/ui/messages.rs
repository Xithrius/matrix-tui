use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};

use crate::{
    events::{Event, InternalEvent, Mode},
    matrix::models::MatrixMessage,
    ui::component::Component,
};

pub struct MessagesWidget {
    event_tx: Sender<Event>,
    table_state: TableState,
    messages: Vec<MatrixMessage>,
}

impl MessagesWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            table_state: TableState::default(),
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, message: MatrixMessage) {
        self.messages.push(message);
    }

    pub fn push_user_message(&mut self, name: String, content: String) {
        let message = MatrixMessage::new(name, content);
        self.push(message);
    }

    #[allow(dead_code)]
    pub fn push_system_message(&mut self, content: String) {
        let message = MatrixMessage::new("System".to_string(), content);
        self.push(message);
    }
}

impl Component for MessagesWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let index = self.table_state.selected();

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.event_tx
                    .send(Event::Internal(InternalEvent::Quit))
                    .await?;
            }
            KeyCode::Up => {
                let index = index.unwrap_or(0).saturating_sub(1);
                self.table_state.select(Some(index));
            }
            KeyCode::Down => {
                if index.is_none() {
                    self.table_state.select(Some(0));
                    return Ok(());
                }

                let index = index.unwrap_or(0).saturating_add(1);
                self.table_state.select(Some(index));
            }
            KeyCode::Char('i') => {
                self.event_tx
                    .send(Event::Internal(InternalEvent::SwitchMode(Mode::Input)))
                    .await?;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let rows: Vec<Row> = self
            .messages
            .iter()
            .map(|message| {
                Row::new(vec![
                    Cell::from(message.name.clone()),
                    Cell::from(message.content.clone()),
                ])
            })
            .collect();
        let widths = [Constraint::Length(20), Constraint::Percentage(100)];
        let table = Table::new(rows, widths)
            .block(
                Block::new()
                    .title("Messages")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .row_highlight_style(Style::new().reversed());

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}
