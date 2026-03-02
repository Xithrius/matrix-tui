use crate::{
    matrix::{login::LoginChoice, message::MatrixMessage},
    ui::LoginCredentials,
};

#[derive(Clone, Debug)]
pub enum MatrixEvent {
    Action(MatrixAction),
    Notification(MatrixNotification),
}

#[derive(Clone, Debug)]
pub enum MatrixAction {
    SelectLogin(LoginChoice, Option<LoginCredentials>),
    ChangeRoom(String),
}

#[derive(Clone, Debug)]
pub enum MatrixNotification {
    LoginChoices(Vec<LoginChoice>),
    Message(MatrixMessage),
}
