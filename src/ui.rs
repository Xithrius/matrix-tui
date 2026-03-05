mod authentication;
mod component;
mod header;
mod input;
mod messages;
mod navigation;
mod user_input;

use tokio::sync::mpsc::Sender;

pub use crate::ui::component::Component;
use crate::{
    events::{Event, Mode},
    ui::{
        authentication::AuthenticationWidget, header::HeaderWidget, input::InputWidget,
        messages::MessagesWidget, navigation::NavigationUI,
    },
};

pub struct Ui {
    pub header: HeaderWidget,
    pub messages: MessagesWidget,
    pub input: InputWidget,
    pub authentication: AuthenticationWidget,
    pub navigation: NavigationUI,
}

impl Ui {
    pub fn new(event_tx: Sender<Event>, mode: Mode) -> Self {
        Self {
            // TODO: Replace motd with something better
            header: HeaderWidget::new("matrix-tui".to_string(), mode),
            messages: MessagesWidget::new(event_tx.clone()),
            input: InputWidget::new(event_tx.clone()),
            authentication: AuthenticationWidget::new(event_tx.clone()),
            navigation: NavigationUI::new(event_tx),
        }
    }
}
