use crate::matrix::{
    login::{LoginChoice, LoginCredentials},
    models::{MatrixMessage, MatrixRoom},
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum MatrixEvent {
    Action(MatrixAction),
    Notification(MatrixNotification),
}

#[derive(Clone, Debug)]
pub enum MatrixAction {
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
    #[allow(dead_code)]
    RestoringSession,
    #[allow(dead_code)]
    SuccessfulSessionRestore,
    LoginChoices(Vec<LoginChoice>),
    /// The login choice selected was successful in authentication,
    /// and the matrix task can now listen for more events besides logging in.
    SuccessfulLogin,
    LoginFailed,
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
