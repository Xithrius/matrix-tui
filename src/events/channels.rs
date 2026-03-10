use color_eyre::Result;
use tokio::sync::mpsc::{Sender, error::SendError};

use crate::events::Event;

pub trait SenderExt<T> {
    async fn send_into(&self, value: impl Into<T>) -> Result<(), SendError<T>>;
}

impl SenderExt<Event> for Sender<Event> {
    async fn send_into(&self, value: impl Into<Event>) -> Result<(), SendError<Event>> {
        self.send(value.into()).await
    }
}
