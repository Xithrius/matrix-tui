use color_eyre::Result;
use matrix_sdk::{
    Client, Room,
    deserialized_responses::TimelineEvent,
    event_handler::Ctx,
    ruma::events::{AnySyncTimelineEvent, room::message::OriginalSyncRoomMessageEvent},
    sync::{SyncResponse, Timeline},
};
use tokio::sync::mpsc::{Receiver, Sender};
use tracing::{error, info, warn};
use url::Url;

use super::event::{MatrixAction, MatrixEvent, MatrixNotification};
use crate::{
    config::CoreConfig,
    events::Event,
    matrix::{context::MatrixContext, login::LoginChoice},
};
use futures_util::StreamExt;
use matrix_sdk::{
    config::SyncSettings, ruma::api::client::session::get_login_types::v3::LoginType,
};

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

        let mut actor = MatrixThread::new(event_tx, action_rx, client, context);
        tokio::task::spawn(async move {
            if let Err(err) = actor.run().await {
                error!("Matrix runner ran into an issue: {}", err);
            }
        });

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

        info!("Available matrix login choices: {:?}", choices);

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

    async fn handle_matrix_action(&mut self, action: MatrixAction) -> Result<()> {
        match action {
            MatrixAction::SelectLogin(login_choice, login_credentials) => {
                login_choice.login(&self.client, login_credentials).await?;
            }
            MatrixAction::ChangeRoom(room_id) => {
                todo!()
            }
        }

        Ok(())
    }

    async fn handle_stream_timeline_event(&mut self, event: &TimelineEvent) -> Result<()> {
        let Ok(event) = event.raw().deserialize() else {
            warn!("Failed to deserialize timeline event: {:?}", event);
            return Ok(());
        };

        info!("Matrix task received sync timeline event {:?}", event);

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting matrix task");

        self.send_login_choices().await?;

        // Wait until we get a selected login action before continuing with regular event handling
        while let Some(action) = self.action_rx.recv().await {
            if let MatrixAction::SelectLogin(login_choice, login_credentials) = action {
                // TODO: Graceful retries for failed login attempts
                login_choice.login(&self.client, login_credentials).await?;
                break;
            }
        }

        self.add_event_handlers().await?;

        let client = self.client.clone();
        let mut sync_stream = {
            let settings = SyncSettings::default();
            let stream = client.sync_stream(settings).await;
            Box::pin(stream)
        };

        loop {
            tokio::select! {
                biased;

                Some(action) = self.action_rx.recv() => {
                    if let Err(err) = self.handle_matrix_action(action).await {
                        error!("Failed to handle matrix action: {}", err);
                    }
                },

                Some(Ok(response)) = sync_stream.next() => {
                    for room in response.rooms.joined.values() {
                        for e in &room.timeline.events {
                            if let Err(err) = self.handle_stream_timeline_event(e).await {
                                error!("Failed to handle timeline event: {}", err);
                            }
                        }
                    }
                }
            }
        }
    }
}
