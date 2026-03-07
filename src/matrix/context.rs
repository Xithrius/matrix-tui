use tokio::sync::mpsc::Sender;

use crate::{
    events::Event,
    matrix::{
        event::{MatrixEvent, MatrixNotification},
        models::MatrixMessage,
    },
};
use color_eyre::{Result, eyre::ContextCompat};
use matrix_sdk::{
    Room, RoomState,
    ruma::events::room::message::{MessageType, OriginalSyncRoomMessageEvent},
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

        let MessageType::Text(msgtype) = &event.content.msgtype else {
            return Ok(());
        };

        let member = room
            .get_member(&event.sender)
            .await?
            .context("The room member doesn't exist")?;
        let name = member.name();

        let message = MatrixMessage::new(name.to_owned(), msgtype.body.clone());

        let room_message_event =
            Event::Matrix(MatrixEvent::Notification(MatrixNotification::Message {
                room_id: room.room_id().to_string(),
                message,
            }));
        self.event_tx.send(room_message_event).await?;

        Ok(())
    }
}
