use std::{
    collections::{BTreeMap, VecDeque},
    sync::LazyLock,
};

use tui::{
    Frame,
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Borders, Cell, Row, Table, TableState},
};

use crate::matrix::models::MatrixMessage;

static DATETIME_STYLE: LazyLock<Style> =
    LazyLock::new(|| Style::default().fg(Color::Rgb(173, 173, 184)));

pub struct MessagesWidget {
    table_state: TableState,
    messages: BTreeMap<String, VecDeque<MatrixMessage>>,
    selected_room_id: Option<String>,
    selected_room_messages: VecDeque<MatrixMessage>,
}

impl MessagesWidget {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default(),
            messages: BTreeMap::default(),
            selected_room_id: None,
            selected_room_messages: VecDeque::new(),
        }
    }

    pub fn set_active_room(&mut self, room_id: &String) {
        self.selected_room_id = Some(room_id.clone());
        self.selected_room_messages = self.messages.get(room_id).cloned().unwrap_or_default();
        self.table_state.select(None);
    }

    pub fn push_message(&mut self, room_id: &String, message: MatrixMessage) {
        self.messages
            .entry(room_id.clone())
            .or_default()
            .push_front(message.clone());

        if self
            .selected_room_id
            .as_deref()
            .is_some_and(|id| id == room_id)
        {
            self.selected_room_messages.push_back(message);
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.table_state = TableState::default();
        self.selected_room_id = None;
        self.selected_room_messages.clear();
    }

    /// Returns the list index of the currently highlighted message, if any.
    pub const fn highlighted_index(&self) -> Option<usize> {
        self.table_state.selected()
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let title = self.selected_room_id.as_deref().map_or_else(
            || "Messages".to_string(),
            |selected_room| {
                format!(
                    "Messages in {selected_room}: {}",
                    self.selected_room_messages.len()
                )
            },
        );

        let rows: Vec<Row> = self
            .selected_room_messages
            .iter()
            .map(|message| {
                let cells = vec![
                    Cell::from(message.datetime.format("%c").to_string()).style(*DATETIME_STYLE),
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
