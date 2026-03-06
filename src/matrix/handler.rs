use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use color_eyre::Result;
use futures_util::StreamExt;
use matrix_sdk::{
    Client, Room,
    config::SyncSettings,
    event_handler::Ctx,
    ruma::{
        api::client::{filter::FilterDefinition, session::get_login_types::v3::LoginType},
        events::room::message::{OriginalSyncRoomMessageEvent, RoomMessageEventContent},
        exports::serde_json,
    },
    sync::SyncResponse,
};
use tokio::{
    fs,
    sync::mpsc::{Receiver, Sender},
};
use tracing::{debug, error, info};
use url::Url;

use super::event::{MatrixAction, MatrixEvent, MatrixNotification};
use crate::{
    config::{CoreConfig, get_data_dir},
    events::Event,
    matrix::{
        context::MatrixContext,
        login::LoginChoice,
        models::MatrixRoom,
        session::{ClientSession, FullSession, build_client, persist_sync_token},
    },
};

#[derive(Debug)]
pub struct MatrixHandler;

impl MatrixHandler {
    pub async fn new(
        config: &CoreConfig,
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
    ) -> Result<Self> {
        let homeserver = Url::parse(&config.matrix.homeserver_url)?;

        let data_dir = get_data_dir().join("persist_session");
        let session_file = data_dir.join("session");
        let (client, client_session) = build_client(&data_dir, homeserver).await?;

        let context = MatrixContext::new(event_tx.clone());

        let mut actor = MatrixThread::new(
            event_tx,
            action_rx,
            client,
            client_session,
            session_file,
            context,
        );
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
    client_session: ClientSession,
    session_file: PathBuf,
    sync_token: Option<String>,
    context: MatrixContext,

    rooms: HashMap<String, Room>,
}

impl MatrixThread {
    fn new(
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
        client: Client,
        client_session: ClientSession,
        session_file: PathBuf,
        context: MatrixContext,
    ) -> Self {
        Self {
            event_tx,
            action_rx,
            client,
            client_session,
            session_file,
            sync_token: None,
            context,
            rooms: HashMap::default(),
        }
    }

    async fn send_login_choices(&self) -> Result<()> {
        let mut choices = Vec::new();
        let login_types = self.client.matrix_auth().get_login_types().await?.flows;

        for login_type in login_types {
            match login_type {
                LoginType::Password(_) => choices.push(LoginChoice::Password),
                LoginType::Sso(sso) => {
                    if sso.identity_providers.is_empty() {
                        choices.push(LoginChoice::Sso);
                    } else {
                        choices.extend(sso.identity_providers.into_iter().map(LoginChoice::SsoIdp));
                    }
                }
                _ => {}
            }
        }

        debug!("Available matrix login choices: {:?}", choices);

        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::LoginChoices(choices),
            )))
            .await?;

        Ok(())
    }

    #[allow(clippy::unnecessary_wraps)]
    fn add_event_handlers(&self) -> Result<()> {
        self.client.add_event_handler_context(self.context.clone());

        self.client.add_event_handler(|event: OriginalSyncRoomMessageEvent, room: Room, context: Ctx<MatrixContext>| async move {
            if let Err(err) = context.on_room_message(event, room).await {
                error!("Failed to handle room message: {}", err);
            }
        });

        Ok(())
    }

    async fn send_message(&self, room_id: &String, message_body: &String) -> Result<()> {
        let Some(room) = self.rooms.get(room_id) else {
            return Ok(());
        };

        let content = RoomMessageEventContent::text_plain(message_body);

        room.send(content).await?;

        Ok(())
    }

    async fn handle_matrix_action(&self, action: &MatrixAction) -> Result<()> {
        match action {
            MatrixAction::GetRooms => {
                let known_rooms: Vec<MatrixRoom> = self
                    .client
                    .rooms()
                    .iter()
                    .cloned()
                    .map(Into::into)
                    .collect();
                self.event_tx
                    .send(Event::Matrix(MatrixEvent::Notification(
                        MatrixNotification::KnownRooms(known_rooms),
                    )))
                    .await?;
            }
            MatrixAction::SendMessage {
                room_id,
                message_body,
            } => {
                self.send_message(room_id, message_body).await?;
            }
            _ => {}
        }

        Ok(())
    }

    async fn handle_sync_stream_response(&self, response: &SyncResponse) -> Result<()> {
        let sync_token = response.next_batch.clone();
        persist_sync_token(&self.session_file, sync_token).await?;

        Ok(())
    }

    async fn attempt_login(&mut self) -> Result<()> {
        self.send_login_choices().await?;

        // Wait until we get a selected login action before continuing with regular event handling
        while let Some(action) = self.action_rx.recv().await {
            if let MatrixAction::SelectLogin {
                choice: login_choice,
                credentials: login_credentials,
            } = action
            {
                // TODO: Graceful retries for failed login attempts
                login_choice.login(&self.client, login_credentials).await?;
                self.event_tx
                    .send(Event::Matrix(MatrixEvent::Notification(
                        MatrixNotification::SuccessfulLogin,
                    )))
                    .await?;
                break;
            }
        }

        let matrix_auth = self.client.matrix_auth();

        // Persist the session to reuse it later.
        // This is not very secure, for simplicity. If the system provides a way of
        // storing secrets securely, it should be used instead.
        // Note that we could also build the user session from the login response.
        let user_session = matrix_auth
            .session()
            .expect("A logged-in client should have a session");
        let full_session = FullSession::new(self.client_session.clone(), user_session, None);
        let serialized_session = serde_json::to_string(&full_session)?;
        fs::write(self.session_file.clone(), serialized_session).await?;

        // After logging in, you might want to verify this session with another one (see
        // the `emoji_verification` example), or bootstrap cross-signing if this is your
        // first session with encryption, or if you need to reset cross-signing because
        // you don't have access to your old sessions (see the
        // `cross_signing_bootstrap` example).

        Ok(())
    }

    async fn attempt_session_restore(&mut self, session_file: &Path) -> Result<()> {
        // The session was serialized as JSON in a file.
        let serialized_session = fs::read_to_string(session_file).await?;
        let FullSession {
            client_session,
            user_session,
            sync_token,
        } = serde_json::from_str(&serialized_session)?;

        // Build the client with the previous settings from the session.
        let client = Client::builder()
            .homeserver_url(client_session.homeserver.clone())
            .sqlite_store(
                client_session.db_path.clone(),
                Some(&client_session.passphrase),
            )
            .build()
            .await?;

        self.client_session = client_session;
        self.sync_token = sync_token;

        info!("Restoring session for {}…", user_session.meta.user_id);

        // Restore the Matrix user session.
        client.restore_session(user_session).await?;

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting matrix task");

        let data_dir = get_data_dir().join("persist_session");
        let session_file = data_dir.join("session");

        if session_file.exists() {
            self.attempt_session_restore(&session_file).await?;
        } else {
            self.attempt_login().await?;
        }

        self.add_event_handlers()?;

        let client = self.client.clone();

        // Enable room members lazy-loading, it will speed up the initial sync a lot
        // with accounts in lots of rooms.
        // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
        let filter = FilterDefinition::with_lazy_loading();

        let mut settings = SyncSettings::default().filter(filter.into());

        let response = client.sync_once(settings.clone()).await?;
        let sync_token = response.next_batch.clone();
        settings = settings.token(sync_token.clone());
        persist_sync_token(&session_file, sync_token).await?;

        let known_rooms: Vec<MatrixRoom> = client.rooms().iter().cloned().map(Into::into).collect();
        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::KnownRooms(known_rooms),
            )))
            .await?;

        let mut sync_stream = {
            let stream = client.sync_stream(settings).await;
            Box::pin(stream)
        };

        loop {
            tokio::select! {
                biased;

                Some(action) = self.action_rx.recv() => {
                    if let Err(err) = self.handle_matrix_action(&action).await {
                        error!("Failed to handle matrix action: {}", err);
                    }
                },
                Some(Ok(response)) = sync_stream.next() => {
                    // I don't know why. I don't want to know why. I shouldn't have to wonder why.
                    // Why does having a `.into()` get rid of the "Expected &SyncResponse, found &SyncResponse"
                    // error from rust analyzer, specifically in VSCode-like environments?
                    #[allow(clippy::useless_conversion)]
                    if let Err(err) = self.handle_sync_stream_response(&response.into()).await {
                        error!("Failed to handle matrix sync stream response: {}", err);
                    }
                }
            }
        }
    }
}
