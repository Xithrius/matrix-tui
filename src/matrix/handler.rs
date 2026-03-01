use color_eyre::Result;
use matrix_sdk::{
    Client, Room,
    event_handler::Ctx,
    ruma::events::room::message::{OriginalSyncRoomMessageEvent, SyncRoomMessageEvent},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::error;
use url::Url;

use crate::{
    config::CoreConfig,
    events::Event,
    matrix::{context::MatrixContext, login::LoginChoice},
};
use matrix_sdk::{
    config::SyncSettings, ruma::api::client::session::get_login_types::v3::LoginType,
};

use super::event::{MatrixAction, MatrixEvent, MatrixNotification};

#[derive(Debug)]
pub struct MatrixHandler;

impl MatrixHandler {
    pub async fn new(
        config: &CoreConfig,
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
    ) -> Result<Self> {
        let homeserver_url = Url::parse(&config.matrix.homeserver_url)?;
        let client = Client::new(homeserver_url).await?;
        let context = MatrixContext::new(event_tx.clone());

        let actor = MatrixThread::new(event_tx, action_rx, client, context);
        tokio::spawn(async move { actor.run().await });

        Ok(Self {})
    }
}

struct MatrixThread {
    event_tx: Sender<Event>,
    action_rx: Receiver<MatrixAction>,
    client: Client,
    context: MatrixContext,
}

impl MatrixThread {
    const fn new(
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
        client: Client,
        context: MatrixContext,
    ) -> Self {
        Self {
            event_tx,
            action_rx,
            client,
            context,
        }
    }

    async fn send_login_choices(&self) -> Result<()> {
        let mut choices = Vec::new();
        let login_types = self.client.matrix_auth().get_login_types().await?.flows;

        for login_type in login_types {
            match login_type {
            LoginType::Password(_) => {
                choices.push(LoginChoice::Password)
            }
            LoginType::Sso(sso) => {
                if sso.identity_providers.is_empty() {
                    choices.push(LoginChoice::Sso)
                } else {
                    choices.extend(sso.identity_providers.into_iter().map(LoginChoice::SsoIdp))
                }
            }
            // This is used for SSO, so it's not a separate choice.
            LoginType::Token(_) |
            // This is only for application services, ignore it here.
            LoginType::ApplicationService(_) => {},
            // We don't support unknown login types.
            _ => {},
        }
        }

        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::LoginChoices(choices),
            )))
            .await?;

        Ok(())
    }

    async fn add_event_handlers(&self) -> Result<()> {
        self.client.add_event_handler_context(self.context.clone());

        // self.client.add_event_handler(
        //     |event: SyncRoomMessageEvent, context: Ctx<MatrixContext>| async move {},
        // );
        self.client.add_event_handler(|event: OriginalSyncRoomMessageEvent, room: Room, context: Ctx<MatrixContext>| async move {
            if let Err(err) = context.on_room_message(event, room).await {
                error!("Failed to handle room message: {}", err);
            }
        });

        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        self.send_login_choices().await?;

        while let Some(action) = self.action_rx.recv().await {
            if let MatrixAction::SelectLogin(login_choice) = action {
                login_choice.login(&self.client).await?;
                break;
            }
        }

        self.add_event_handlers().await?;
        self.client.sync(SyncSettings::new()).await?;

        Ok(())
    }

    async fn send(&self, event: Event) {
        let _ = self.event_tx.send(event).await;
    }
}
