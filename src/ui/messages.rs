use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};

use crate::{
    events::{Event, InternalEvent, Mode},
    ui::component::Component,
};

pub struct MessagesWidget {
    event_tx: Sender<Event>,
    table_state: TableState,
    messages: Vec<String>,
}

impl MessagesWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            table_state: TableState::default(),
            messages: Vec::new(),
        }
    }

    pub fn push(&mut self, message: String) {
        self.messages.push(message);
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
        let [_, top, _] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ])
        .areas(area);

        let rows: Vec<Row> = self
            .messages
            .iter()
            .map(|message| {
                Row::new(vec![
                    // TODO: More attributes of the message
                    Cell::from(message.to_string()),
                ])
            })
            .collect();
        let widths = [Constraint::Length(45), Constraint::Percentage(100)];
        let table = Table::new(rows, widths)
            .block(
                Block::new()
                    .title("Messages")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .row_highlight_style(Style::new().reversed());

        frame.render_stateful_widget(table, top, &mut self.table_state);
    }
}
