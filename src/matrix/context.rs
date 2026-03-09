use color_eyre::Result;
use matrix_sdk::{
    Room, RoomState,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
};
use tokio::sync::mpsc::Sender;

use crate::{
    events::Event,
    matrix::{
        event::{MatrixEvent, MatrixNotification},
        models::MatrixMessage,
    },
    utils::ChronoExt,
};

#[derive(Debug, Clone)]
pub struct MatrixContext {
    event_tx: Sender<Event>,
}

impl MatrixContext {
    pub const fn new(event_tx: Sender<Event>) -> Self {
        Self { event_tx }
    }

    pub async fn on_room_message(
        &self,
        event: OriginalSyncRoomMessageEvent,
        room: Room,
    ) -> Result<()> {
        // Only listen to joined rooms
        if room.state() != RoomState::Joined {
            return Ok(());
        }

        let MessageType::Text(text_message) = &event.content.msgtype else {
            return Ok(());
        };

        let datetime = event.origin_server_ts.origin_server_chrono()?;
        let formatted_datetime = datetime.format("%c").to_string();
        let name = event.sender.localpart();
        let content = text_message.body.clone();

        let message = MatrixMessage::new(formatted_datetime, name.to_owned(), content);

        let room_message_event =
            Event::Matrix(MatrixEvent::Notification(MatrixNotification::Message {
                room_id: room.room_id().to_string(),
                message,
            }));
        self.event_tx.send(room_message_event).await?;

        Ok(())
    }
}
