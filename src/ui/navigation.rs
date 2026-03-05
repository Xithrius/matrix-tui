mod rooms;

use tokio::sync::mpsc::Sender;

use crate::{events::Event, ui::navigation::rooms::RoomNavigationWidget};

pub struct NavigationUI {
    pub rooms: RoomNavigationWidget,
}

impl NavigationUI {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let rooms = RoomNavigationWidget::new(event_tx);

        Self { rooms }
    }
}
