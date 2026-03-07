use std::{
    collections::{BTreeMap, VecDeque},
    string::ToString,
};

use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
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

    selected_room_id: Option<String>,
    messages: BTreeMap<String, VecDeque<MatrixMessage>>,
}

impl MessagesWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            table_state: TableState::default(),
            selected_room_id: None,
            messages: BTreeMap::default(),
        }
    }

    #[allow(dead_code)]
    pub fn set_selected_room_id(&mut self, room_id: String) {
        self.selected_room_id = Some(room_id);
    }

    pub fn ensure_selected_room_id(&mut self) {
        if self.selected_room_id.is_some() {
            return;
        }

        let first_room_id = self
            .messages
            .keys()
            .collect::<Vec<&String>>()
            .first()
            .map(ToString::to_string);

        self.selected_room_id = first_room_id;
    }

    pub fn push_message(&mut self, room_id: String, message: MatrixMessage) {
        self.messages.entry(room_id).or_default().push_back(message);
    }
}

impl Component for MessagesWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let index = self.table_state.selected();

        let contains_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if contains_ctrl {
            if key.code == KeyCode::Char(' ') {
                self.event_tx
                    .send(Event::Internal(InternalEvent::SwitchMode(
                        Mode::RoomNavigation,
                    )))
                    .await?;
            }

            return Ok(());
        }

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
        let selected_room_id = self.selected_room_id.as_ref();
        let room_messages = if let Some(selected_room_id) = selected_room_id {
            self.messages
                .get(selected_room_id)
                .cloned()
                .unwrap_or_default()
        } else {
            VecDeque::new()
        };

        let rows: Vec<Row> = room_messages
            .iter()
            .map(|message| {
                let cells = vec![
                    Cell::from(message.name.clone()),
                    Cell::from(message.content.clone()),
                ];
                Row::new(cells)
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
