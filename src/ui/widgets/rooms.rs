use std::collections::BTreeMap;

use tui::{
    Frame,
    layout::Rect,
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use crate::matrix::models::MatrixRoom;

pub struct RoomsWidget {
    // TODO: BTreeMap is probably not needed here, use a vector
    rooms: BTreeMap<String, MatrixRoom>,

    list_state: ListState,
    selected_room_id: Option<String>,
}

impl RoomsWidget {
    pub fn new() -> Self {
        Self {
            rooms: BTreeMap::default(),
            list_state: ListState::default(),
            selected_room_id: None,
        }
    }

    pub fn get_selected_room_id(&self) -> Option<String> {
        self.selected_room_id.clone()
    }

    pub fn set_selected_room_id(&mut self, room_id: &String) {
        let selected_index = self.rooms.keys().position(|k| k.as_str() == room_id);
        self.list_state.select(selected_index);
        self.selected_room_id = Some(room_id.clone());
    }

    pub fn push_room(&mut self, room: MatrixRoom) {
        self.rooms.insert(room.id.clone(), room);
    }

    pub fn highlighted_room_id(&self) -> Option<String> {
        let selected = self.list_state.selected()?;
        self.rooms.keys().nth(selected).cloned()
    }

    pub fn clear(&mut self) {
        self.rooms.clear();
        self.list_state = ListState::default();
        self.selected_room_id = None;
    }

    pub fn select_prev(&mut self) {
        let idx = self.list_state.selected().unwrap_or(0).saturating_sub(1);
        self.list_state.select(Some(idx));
    }

    pub fn select_next(&mut self) {
        let len = self.rooms.len();
        if len == 0 {
            return;
        }
        if self.list_state.selected().is_none() {
            self.list_state.select(Some(0));
            return;
        }
        let idx = self
            .list_state
            .selected()
            .unwrap_or(0)
            .saturating_add(1)
            .min(len - 1);
        self.list_state.select(Some(idx));
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let entries: Vec<ListItem> = self
            .rooms
            .values()
            .map(|room| {
                let line_item = room.name.clone().unwrap_or_else(|| room.id.clone());
                let mut list_item = ListItem::new(vec![Line::from(line_item)]);

                let is_selected_room = self
                    .selected_room_id
                    .as_ref()
                    .is_some_and(|room_id| *room_id == room.id);
                if is_selected_room {
                    list_item = list_item.style(Style::default().bg(Color::LightGreen));
                }

                list_item
            })
            .collect();

        let block = Block::new()
            .title("Rooms")
            .borders(Borders::all())
            .border_type(BorderType::Rounded);
        let list = List::new(entries)
            .block(block)
            .highlight_style(Style::new().reversed());

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }
}
