use crate::matrix::login::LoginChoice;

#[derive(Clone, Debug)]
pub enum MatrixEvent {
    LoginChoices(Vec<LoginChoice>),
}
