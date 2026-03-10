use crate::matrix::{
    login::{LoginChoice, LoginCredentials},
    models::{MatrixMessage, MatrixRoom},
};

#[derive(Clone, Debug)]
pub enum MatrixEvent {
    Action(MatrixAction),
    Notification(MatrixNotification),
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
