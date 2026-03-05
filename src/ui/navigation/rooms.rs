use std::collections::BTreeMap;

use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{
    crossterm::event::{KeyCode, KeyEvent},
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
};

use crate::{
    events::{Event, InternalEvent, Mode},
    matrix::models::MatrixRoom,
    ui::component::Component,
};

pub struct RoomNavigationWidget {
    event_tx: Sender<Event>,
    rooms: BTreeMap<String, MatrixRoom>,

    list_state: ListState,
}

impl RoomNavigationWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            rooms: BTreeMap::default(),

            list_state: ListState::default(),
        }
    }

    pub const fn ensure_initial_selection(&mut self) {
        let selected = self.list_state.selected();
        if selected.is_none() {
            self.list_state.select(Some(0));
        }
    }

    pub fn push_room(&mut self, room_id: String, room: MatrixRoom) {
        self.rooms.insert(room_id, room);
    }

    #[allow(dead_code)]
    pub fn remove_room(&mut self, room_id: &String) {
        self.rooms.remove(room_id);
    }
}

impl Component for RoomNavigationWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        let index = self.list_state.selected();

        match key.code {
            KeyCode::Esc => {
                self.event_tx
                    .send(Event::Internal(InternalEvent::SwitchMode(Mode::Messages)))
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
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let entries: Vec<ListItem> = self
            .rooms
            .values()
            .map(|room| {
                let line_item = room.name.clone().unwrap_or_else(|| room.id.clone());
                ListItem::new(vec![Line::from(line_item)])
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
