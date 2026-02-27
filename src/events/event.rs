use tui::crossterm::event::Event as CrosstermEvent;

use crate::matrix::event::MatrixEvent;

#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling exernal systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Internal application events.
    Internal(InternalEvent),
    /// Matrix-SDK events
    Matrix(MatrixEvent),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    #[default]
    Messages,
    Input,
}

#[derive(Clone, Debug)]
pub enum InternalEvent {
    SwitchMode(Mode),
    SendMessage(String),
    Quit,
}
