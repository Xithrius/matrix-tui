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
    list_state: ListState,
    rooms: Vec<MatrixRoom>,
}

impl RoomNavigationWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            event_tx,
            list_state: ListState::default(),
            rooms: Vec::new(),
        }
    }

    pub fn push(&mut self, room: MatrixRoom) {
        self.rooms.push(room);
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
        let [left, _] =
            Layout::horizontal([Constraint::Length(20), Constraint::Percentage(100)]).areas(area);

        let entries: Vec<ListItem> = self
            .rooms
            .iter()
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

        frame.render_stateful_widget(list, left, &mut self.list_state);
    }
}
