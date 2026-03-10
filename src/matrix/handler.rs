use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use color_eyre::{Result, eyre::ContextCompat};
use futures_util::StreamExt;
use matrix_sdk::{
    Client, Room,
    config::SyncSettings,
    deserialized_responses::TimelineEventKind,
    event_handler::Ctx,
    room::MessagesOptions,
    ruma::{
        api::{
            Direction,
            client::{filter::FilterDefinition, session::get_login_types::v3::LoginType},
        },
        events::{
            AnyMessageLikeEvent, AnyTimelineEvent, MessageLikeEvent,
            room::message::{OriginalSyncRoomMessageEvent, RoomMessageEventContent},
        },
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
        models::{MatrixMessage, MatrixRoom},
        session::{ClientSession, FullSession, build_client, persist_sync_token},
    },
    utils::ChronoExt,
};

#[derive(Debug)]
pub struct MatrixHandler;

impl MatrixHandler {
    pub fn new(
        config: &CoreConfig,
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
    ) -> Result<Self> {
        let homeserver = Url::parse(&config.matrix.homeserver_url)?;

        let data_dir = get_data_dir().join("persist_session");
        let session_file = data_dir.join("session");

        let context = MatrixContext::new(event_tx.clone());

        let mut actor = MatrixThread::new(event_tx, action_rx, homeserver, session_file, context);
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
    homeserver: Url,
    client: Option<Client>,
    client_session: Option<ClientSession>,
    session_file: PathBuf,
    sync_token: Option<String>,
    context: MatrixContext,

    rooms: HashMap<String, Room>,
}

impl MatrixThread {
    fn new(
        event_tx: Sender<Event>,
        action_rx: Receiver<MatrixAction>,
        homeserver: Url,
        session_file: PathBuf,
        context: MatrixContext,
    ) -> Self {
        Self {
            event_tx,
            action_rx,
            homeserver,
            client: None,
            client_session: None,
            session_file,
            sync_token: None,
            context,
            rooms: HashMap::default(),
        }
    }

    async fn send_login_choices(&self, client: &Client) -> Result<()> {
        let mut choices = Vec::new();
        let login_types = client.matrix_auth().get_login_types().await?.flows;

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
        let client = self
            .client
            .as_ref()
            .context("Could not get client for sending login choices")?;

        client.add_event_handler_context(self.context.clone());

        client.add_event_handler(|event: OriginalSyncRoomMessageEvent, room: Room, context: Ctx<MatrixContext>| async move {
            if let Err(err) = context.on_room_message(event, room).await {
                error!("Failed to handle room message: {}", err);
            }
        });

        Ok(())
    }

    fn insert_rooms(&mut self, rooms: &[Room]) {
        for room in rooms {
            let room_id = room.room_id().to_string();
            self.rooms.entry(room_id).or_insert_with(|| room.clone());
        }
    }

    async fn send_message(&self, room_id: &String, message_body: &String) -> Result<()> {
        let Some(room) = self.rooms.get(room_id) else {
            return Ok(());
        };

        let content = RoomMessageEventContent::text_plain(message_body);

        room.send(content).await?;

        Ok(())
    }

    async fn handle_matrix_action(&mut self, action: &MatrixAction) -> Result<()> {
        let client = self
            .client
            .as_ref()
            .context("Could not get client for handling matrix action")?;

        match action {
            MatrixAction::StartRestoreSession => {
                todo!();
            }
            MatrixAction::StartLoggingIn => {
                todo!();
            }
            MatrixAction::SelectLogin { .. } => {}
            MatrixAction::GetRooms => {
                let rooms = client.rooms();
                self.insert_rooms(&rooms);
                let known_rooms: Vec<MatrixRoom> = rooms.iter().cloned().map(Into::into).collect();

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
            MatrixAction::GetRoomMessages(room_id) => {
                let rooms = client.rooms();
                let Some(room) = rooms.iter().find(|room| room.room_id() == room_id) else {
                    return Ok(());
                };

                let message_filter_options = MessagesOptions::new(Direction::Backward);

                let room_messages = room.messages(message_filter_options).await?;
                let mut messages = Vec::new();
                for event in &room_messages.chunk {
                    let deserialized_event = match &event.kind {
                        TimelineEventKind::Decrypted(decrypted_room_event) => {
                            decrypted_room_event.event.deserialize()?
                        }
                        TimelineEventKind::UnableToDecrypt { event, utd_info: _ }
                        | TimelineEventKind::PlainText { event } => {
                            event.deserialize()?.into_full_event(room.room_id().into())
                        }
                    };

                    let AnyTimelineEvent::MessageLike(AnyMessageLikeEvent::RoomMessage(
                        MessageLikeEvent::Original(m),
                    )) = deserialized_event
                    else {
                        continue;
                    };

                    let datetime = match m.origin_server_ts.origin_server_chrono() {
                        Ok(datetime) => datetime.format("%c"),
                        Err(err) => {
                            error!(
                                "Failed to convert origin server timestamp to datetime: {}",
                                err
                            );
                            continue;
                        }
                    };

                    let name = m.sender.localpart();
                    let content = m.content.body();

                    let message = MatrixMessage::new(
                        datetime.to_string(),
                        name.to_owned(),
                        content.to_owned(),
                    );
                    messages.push(message);
                }

                self.event_tx
                    .send(Event::Matrix(MatrixEvent::Notification(
                        MatrixNotification::RoomMessages {
                            room_id: room_id.clone(),
                            messages,
                        },
                    )))
                    .await?;
            }
        }

        Ok(())
    }

    async fn handle_sync_stream_response(&self, response: &SyncResponse) -> Result<()> {
        let sync_token = response.next_batch.clone();
        persist_sync_token(&self.session_file, sync_token).await?;

        Ok(())
    }

    async fn attempt_login(&mut self, data_dir: &Path) -> Result<()> {
        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::LoggingIn,
            )))
            .await?;

        let (client, client_session) = build_client(data_dir, self.homeserver.clone()).await?;

        self.send_login_choices(&client).await?;

        // Wait until we get a selected login action before continuing with regular event handling
        while let Some(action) = self.action_rx.recv().await {
            if let MatrixAction::SelectLogin {
                choice: login_choice,
                credentials: login_credentials,
            } = action
            {
                // TODO: Graceful retries for failed login attempts
                login_choice.login(&client, login_credentials).await?;
                self.event_tx
                    .send(Event::Matrix(MatrixEvent::Notification(
                        MatrixNotification::SuccessfulLogin,
                    )))
                    .await?;
                break;
            }
        }

        let matrix_auth = client.matrix_auth();

        // Persist the session to reuse it later.
        // This is not very secure, for simplicity. If the system provides a way of
        // storing secrets securely, it should be used instead.
        // Note that we could also build the user session from the login response.
        let user_session = matrix_auth
            .session()
            .context("Could not find Matrix session")?;
        let full_session = FullSession::new(client_session.clone(), user_session, None);
        let serialized_session = serde_json::to_string(&full_session)?;
        fs::write(self.session_file.clone(), serialized_session).await?;

        // After logging in, you might want to verify this session with another one (see
        // the `emoji_verification` example), or bootstrap cross-signing if this is your
        // first session with encryption, or if you need to reset cross-signing because
        // you don't have access to your old sessions (see the
        // `cross_signing_bootstrap` example).

        self.client = Some(client);
        self.client_session = Some(client_session);

        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::SuccessfulLogin,
            )))
            .await?;

        Ok(())
    }

    async fn attempt_session_restore(&mut self, session_file: &Path) -> Result<()> {
        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::RestoringSession,
            )))
            .await?;

        // The session was serialized as JSON in a file.
        let serialized_session = fs::read_to_string(session_file).await?;
        let FullSession {
            client_session,
            user_session,
            sync_token,
        } = serde_json::from_str(&serialized_session)?;

        info!("Restoring session for {}...", user_session.meta.user_id);

        // Build the client with the previous settings from the session.
        let client = Client::builder()
            .homeserver_url(client_session.homeserver.clone())
            .sqlite_store(
                client_session.db_path.clone(),
                Some(&client_session.passphrase),
            )
            .build()
            .await?;

        client.restore_session(user_session).await?;

        self.client = Some(client);
        self.client_session = Some(client_session);
        self.sync_token = sync_token;

        self.event_tx
            .send(Event::Matrix(MatrixEvent::Notification(
                MatrixNotification::SuccessfulSessionRestore,
            )))
            .await?;

        info!("Completed restoring session");

        Ok(())
    }

    async fn run(&mut self) -> Result<()> {
        info!("Starting matrix task");

        let data_dir = get_data_dir().join("persist_session");
        let session_file = data_dir.join("session");

        // TODO: Handle errors via sending information through the event sender
        if session_file.exists() {
            self.attempt_session_restore(&session_file).await?;
        } else {
            self.attempt_login(&data_dir).await?;
        }

        self.add_event_handlers()?;

        let client = self
            .client
            .clone()
            .context("Failed to get client after login or session restore")?;

        // Enable room members lazy-loading, it will speed up the initial sync a lot
        // with accounts in lots of rooms.
        // See <https://spec.matrix.org/v1.6/client-server-api/#lazy-loading-room-members>.
        let filter = FilterDefinition::with_lazy_loading();

        let mut settings = SyncSettings::default().filter(filter.into());

        let response = client.sync_once(settings.clone()).await?;
        let sync_token = response.next_batch.clone();
        settings = settings.token(sync_token.clone());
        persist_sync_token(&session_file, sync_token).await?;

        let rooms = client.rooms();
        self.insert_rooms(&rooms);
        let known_rooms: Vec<MatrixRoom> = rooms.iter().cloned().map(Into::into).collect();
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
