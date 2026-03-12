use color_eyre::Result;
use tokio::sync::mpsc::{Sender, channel};
use tracing::{debug, error};
use tui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    config::CoreConfig,
    events::{Event, EventHandler, InternalEvent, LoginMode, Mode},
    matrix::{
        event::{MatrixAction, MatrixEvent, MatrixNotification},
        handler::MatrixHandler,
    },
    ui::{Component, Status, Ui},
};

pub struct App {
    running: bool,
    events: EventHandler,
    event_tx: Sender<Event>,
    matrix_tx: Sender<MatrixAction>,

    mode: Mode,
    ui: Ui,
}

impl App {
    pub fn new(config: &CoreConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel(100);
        let (matrix_tx, matrix_rx) = channel(100);
        let mode = Mode::default();

        let ui = Ui::new(event_tx.clone(), mode.clone());

        let events = EventHandler::new(config, event_tx.clone(), event_rx);
        MatrixHandler::new(config, event_tx.clone(), matrix_rx)?;

        Ok(Self {
            running: true,
            events,
            event_tx,
            matrix_tx,
            mode,
            ui,
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame, frame.area()))?;
            self.handle_events().await?;
        }

        Ok(())
    }

    async fn handle_crossterm_event(&mut self, event: CrosstermEvent) -> Result<()> {
        if let CrosstermEvent::Key(key_event) = event
            && key_event.kind == KeyEventKind::Press
        {
            self.handle_key_event(key_event).await?;
        }

        Ok(())
    }

    async fn handle_internal_event(&mut self, event: InternalEvent) -> Result<()> {
        match event {
            InternalEvent::SwitchMode(mode) => {
                self.switch_mode(mode).await?;
            }
            InternalEvent::Quit => {
                self.quit();
            }
            InternalEvent::SendMessage(content) => {
                // TODO: Add to app context and pass reference to messages UI
                let Some(room_id) = self.ui.navigation.rooms.get_selected_room_id() else {
                    error!("Could not find selected room ID when sending message");
                    return Ok(());
                };

                self.matrix_tx
                    .send(MatrixAction::SendMessage {
                        room_id,
                        message_body: content,
                    })
                    .await?;
            }
            InternalEvent::SwitchRoom(room_id) => {
                // TODO: Pass down the initial room ID as a reference instead of setting it everywhere
                self.ui.navigation.rooms.set_selected_room_id(&room_id);
                self.ui.messages.set_selected_room_id(room_id);
            }
        }

        Ok(())
    }

    async fn handle_matrix_event(&mut self, event: MatrixEvent) -> Result<()> {
        match event {
            MatrixEvent::Action(matrix_action) => {
                self.matrix_tx.send(matrix_action).await?;
            }
            MatrixEvent::Notification(matrix_notification) => {
                self.handle_matrix_notification(matrix_notification).await?;
            }
        }

        Ok(())
    }

    async fn handle_matrix_notification(&mut self, notification: MatrixNotification) -> Result<()> {
        match notification {
            MatrixNotification::RestoringSession => {
                self.ui
                    .status_line
                    .set_status(Status::Info("Restoring session...".to_string()));
            }
            MatrixNotification::SuccessfulSessionRestore => {
                self.switch_mode(Mode::Messages).await?;
                self.ui
                    .status_line
                    .set_status(Status::Info("Session restored successfully".to_string()));
            }
            MatrixNotification::LoginChoices(login_choices) => {
                self.ui.authentication.set_login_choices(login_choices);
                self.ui
                    .status_line
                    .set_status(Status::Info("Select login option".to_string()));
            }
            MatrixNotification::Message { room_id, message } => {
                // TODO: Add to app context and pass reference to messages UI
                self.ui.messages.push_message(&room_id, message);
            }
            MatrixNotification::SuccessfulLogin => {
                self.switch_mode(Mode::Messages).await?;
                self.ui
                    .status_line
                    .set_status(Status::Info("Login successful".to_string()));
            }
            MatrixNotification::LoginFailed => {
                self.switch_mode(Mode::Login(LoginMode::SelectLoginChoice))
                    .await?;
                self.ui
                    .status_line
                    .set_status(Status::Error("Login failed".to_string()));
            }
            MatrixNotification::KnownRooms(rooms) => {
                let Some(first_room) = rooms.first().map(|room| room.id.clone()) else {
                    return Ok(());
                };

                // TODO: Pass down the initial room ID as a reference instead of setting it everywhere
                self.ui.navigation.rooms.set_selected_room_id(&first_room);
                self.ui.messages.set_selected_room_id(first_room);

                for room in rooms {
                    let room_id = room.id.clone();
                    self.ui.navigation.rooms.push_room(room_id.clone(), room);

                    self.event_tx
                        .send(Event::Matrix(MatrixEvent::Action(
                            MatrixAction::GetRoomMessages(room_id),
                        )))
                        .await?;
                }
            }
            MatrixNotification::RoomMessages { room_id, messages } => {
                for message in messages {
                    self.ui.messages.push_message(&room_id, message);
                }
            }
        }

        Ok(())
    }

    pub async fn handle_events(&mut self) -> Result<()> {
        let Some(event) = self.events.next().await else {
            return Ok(());
        };

        match event {
            Event::Tick => {
                self.tick();
            }
            Event::Crossterm(event) => {
                self.handle_crossterm_event(event).await?;
            }
            Event::Internal(event) => {
                self.handle_internal_event(event).await?;
            }
            Event::Matrix(event) => {
                self.handle_matrix_event(event).await?;
            }
        }

        Ok(())
    }

    /// Handles the tick event of the terminal.
    ///
    /// The tick event is where you can update the state of your application with any logic that
    /// needs to be updated at a fixed frame rate. E.g. polling a server, updating an animation.
    #[allow(clippy::unused_self)]
    pub const fn tick(&self) {}

    pub const fn quit(&mut self) {
        self.running = false;
    }

    pub async fn switch_mode(&mut self, mode: Mode) -> Result<()> {
        debug!("Switching to mode: {:?}", mode);

        match &mode {
            Mode::Input => self.ui.input.set_focused(true),
            Mode::Login(login_mode) => {
                // TODO: Find where completed entering of credentials can be handled
                if matches!(login_mode, LoginMode::Completed)
                    && let Some(login_choice) = self.ui.authentication.selected_login_choice()
                {
                    let credentials = self.ui.authentication.get_login_credentials();
                    let matrix_action =
                        Event::Matrix(MatrixEvent::Action(MatrixAction::SelectLogin {
                            choice: login_choice,
                            credentials,
                        }));
                    self.event_tx.send(matrix_action).await?;
                }

                self.ui.authentication.set_login_mode(login_mode.clone());
            }
            Mode::Messages | Mode::RoomNavigation => {}
        }

        self.ui.header.set_mode(mode.clone());
        self.mode = mode;

        Ok(())
    }
}

impl Component for App {
    async fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        debug!("Received key event: {:?}", key_event);

        match &self.mode {
            Mode::Messages => self.ui.messages.handle_key_event(key_event).await,
            Mode::Input => self.ui.input.handle_key_event(key_event).await,
            Mode::Login(_) => self.ui.authentication.handle_key_event(key_event).await,
            Mode::RoomNavigation => self.ui.navigation.rooms.handle_key_event(key_event).await,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        if let Mode::Login(_) = self.mode {
            let [login_area, status_area] =
                Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)]).areas(area);

            self.ui.authentication.draw(frame, login_area);
            self.ui.status_line.draw(frame, status_area);
            return;
        }

        let [header_area, content_area, input_area, status_area] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(area);

        let [navigation_area, messages_area] =
            Layout::horizontal([Constraint::Length(30), Constraint::Percentage(100)])
                .areas(content_area);

        self.ui.header.draw(frame, header_area);
        self.ui.navigation.rooms.draw(frame, navigation_area);
        self.ui.messages.draw(frame, messages_area);
        self.ui.input.draw(frame, input_area);
        self.ui.status_line.draw(frame, status_area);
    }
}
