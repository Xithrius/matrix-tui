use crate::{
    matrix::{
        login::LoginChoice,
        models::{MatrixMessage, MatrixRoom},
    },
    ui::LoginCredentials,
};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum MatrixEvent {
    Action(MatrixAction),
    Notification(MatrixNotification),
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum MatrixAction {
    SelectLogin {
        choice: LoginChoice,
        credentials: Option<LoginCredentials>,
    },
    GetRooms,
    ChangeRoom(String),
    SendMessage {
        room_id: String,
        message_body: String,
    },
}

#[derive(Clone, Debug)]
pub enum MatrixNotification {
    LoginChoices(Vec<LoginChoice>),
    /// The login choice selected was successful in authentication,
    /// and the matrix task can now listen for more events besides logging in.
    SuccessfulLogin,
    /// All the rooms the matrix client knows about.
    ///
    /// This includes joined, invited, and left rooms.
    KnownRooms(Vec<MatrixRoom>),
    Message(MatrixMessage),
}
