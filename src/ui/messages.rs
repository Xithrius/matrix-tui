use std::{
    collections::{BTreeMap, VecDeque},
    sync::LazyLock,
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

static DATETIME_STYLE: LazyLock<Style> =
    LazyLock::new(|| Style::default().fg(Color::Rgb(173, 173, 184)));

pub struct MessagesWidget {
    event_tx: Sender<Event>,
    table_state: TableState,

    messages: BTreeMap<String, VecDeque<MatrixMessage>>,
    selected_room_id: Option<String>,
    selected_room_messages: Option<VecDeque<MatrixMessage>>,
}

impl MessagesWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            table_state: TableState::default(),
            messages: BTreeMap::default(),
            selected_room_id: None,
            selected_room_messages: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_selected_room_id(&self) -> Option<String> {
        self.selected_room_id.clone()
    }

    pub fn set_selected_room_id(&mut self, room_id: String) {
        self.selected_room_id = Some(room_id);

        let selected_room_messages = if let Some(selected_room_id) = self.selected_room_id.as_ref()
        {
            self.messages
                .get(selected_room_id)
                .cloned()
                .unwrap_or_default()
        } else {
            VecDeque::new()
        };

        self.selected_room_messages = Some(selected_room_messages);
    }

    pub fn push_message(&mut self, room_id: &String, message: MatrixMessage) {
        self.messages
            .entry(room_id.clone())
            .or_default()
            .push_front(message.clone());

        if self
            .selected_room_id
            .as_ref()
            .is_some_and(|selected_room_id| selected_room_id == room_id)
            && let Some(selected_room_messages) = self.selected_room_messages.as_mut()
        {
            selected_room_messages.push_back(message);
        }
    }
}

impl Component for MessagesWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let index = self.table_state.selected();

        let contains_ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        if contains_ctrl && key.code == KeyCode::Char('r') {
            self.event_tx
                .send(Event::Internal(InternalEvent::SwitchMode(
                    Mode::RoomNavigation,
                )))
                .await?;

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
        let selected_room_messages = self.selected_room_messages.clone().unwrap_or_default();

        let title = self.selected_room_id.clone().map_or_else(
            || "Messages".to_string(),
            |selected_room| {
                format!(
                    "Messages in {selected_room}: {}",
                    selected_room_messages.len()
                )
            },
        );

        let rows: Vec<Row> = selected_room_messages
            .iter()
            .map(|message| {
                let cells = vec![
                    Cell::from(message.datetime.clone()).style(*DATETIME_STYLE),
                    Cell::from(message.name.clone()),
                    Cell::from(message.content.clone()),
                ];
                Row::new(cells)
            })
            .collect();
        let widths = [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Percentage(100),
        ];
        let table = Table::new(rows, widths)
            .block(
                Block::new()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .row_highlight_style(Style::new().reversed());

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }
}
