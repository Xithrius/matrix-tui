use crate::{
    events::Event,
    matrix::{
        login::{LoginChoice, LoginCredentials},
        models::{MatrixMessage, MatrixRoom},
    },
};

#[derive(Clone, Debug)]
pub enum MatrixEvent {
    Action(MatrixAction),
    Notification(MatrixNotification),
}

impl From<MatrixEvent> for Event {
    fn from(value: MatrixEvent) -> Self {
        Self::Matrix(value)
    }
}

#[derive(Clone, Debug)]
pub enum MatrixAction {
    StartLoggingIn,
    StartRestoreSession,
    SelectLogin {
        choice: LoginChoice,
        credentials: Option<LoginCredentials>,
    },
    #[allow(dead_code)]
    GetRooms,
    GetRoomMessages(String),
    SendMessage {
        room_id: String,
        message_body: String,
    },
}

impl From<MatrixAction> for Event {
    fn from(value: MatrixAction) -> Self {
        Self::Matrix(MatrixEvent::Action(value))
    }
}

#[derive(Clone, Debug)]
pub enum MatrixNotification {
    /// The user has selected [`MatrixAction::StartLoggingIn`]
    LoggingIn,
    LoginChoices(Vec<LoginChoice>),
    /// The selected login choice had no issues when authenticating
    SuccessfulLogin,

    /// Started restoring the session, after [`MatrixAction::StartRestoreSession`] was selected
    RestoringSession,
    /// Finished restoring the session without issue
    SuccessfulSessionRestore,

    /// All the rooms the matrix client knows about.
    ///
    /// This includes joined, invited, and left rooms.
    KnownRooms(Vec<MatrixRoom>),
    RoomMessages {
        room_id: String,
        messages: Vec<MatrixMessage>,
    },
    Message {
        room_id: String,
        message: MatrixMessage,
    },
}

impl From<MatrixNotification> for Event {
    fn from(value: MatrixNotification) -> Self {
        Self::Matrix(MatrixEvent::Notification(value))
    }
}
