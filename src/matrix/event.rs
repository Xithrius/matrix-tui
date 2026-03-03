use crate::{
    matrix::{login::LoginChoice, message::MatrixMessage},
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
    ChangeRoom(String),
}

#[derive(Clone, Debug)]
pub enum MatrixNotification {
    LoginChoices(Vec<LoginChoice>),
    SuccessfulLogin,
    Message(MatrixMessage),
}
