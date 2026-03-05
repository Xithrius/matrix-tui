use color_eyre::Result;
use tokio::sync::mpsc::{Sender, channel};
use tracing::debug;
use tui::{
    DefaultTerminal, Frame,
    crossterm::{event::Event as CrosstermEvent, event::KeyEvent, event::KeyEventKind},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    config::CoreConfig,
    events::{Event, EventHandler, InternalEvent, LoginMode, Mode},
    matrix::{
        event::{MatrixAction, MatrixEvent, MatrixNotification},
        handler::MatrixHandler,
    },
    ui::{
        AuthenticationWidget, Component, HeaderWidget, InputWidget, MessagesWidget,
        RoomNavigationWidget,
    },
};

pub struct Ui {
    header: HeaderWidget,
    messages: MessagesWidget,
    input: InputWidget,
    authentication: AuthenticationWidget,
    room_navigation: RoomNavigationWidget,
}

impl Ui {
    pub fn new(event_tx: Sender<Event>, mode: Mode) -> Self {
        Self {
            // TODO: Replace motd with something better
            header: HeaderWidget::new("matrix-tui".to_string(), mode),
            messages: MessagesWidget::new(event_tx.clone()),
            input: InputWidget::new(event_tx.clone()),
            authentication: AuthenticationWidget::new(event_tx.clone()),
            room_navigation: RoomNavigationWidget::new(event_tx),
        }
    }
}

pub struct App {
    config: CoreConfig,
    running: bool,
    events: EventHandler,
    event_tx: Sender<Event>,
    matrix_tx: Sender<MatrixAction>,

    mode: Mode,
    ui: Ui,
}

impl App {
    pub async fn new(config: &CoreConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel(100);
        let (matrix_tx, matrix_rx) = channel(100);
        let mode = Mode::default();

        let ui = Ui::new(event_tx.clone(), mode.clone());

        let events = EventHandler::new(config, event_tx.clone(), event_rx);
        MatrixHandler::new(config, event_tx.clone(), matrix_rx).await?;

        Ok(Self {
            config: config.clone(),
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
                let name = self.config.matrix.username.clone();
                self.ui.messages.push_user_message(name, content);
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
            MatrixNotification::LoginChoices(login_choices) => {
                self.ui.authentication.set_login_choices(login_choices);
            }
            MatrixNotification::Message(matrix_message) => {
                // TODO: Add to app context and pass reference to messages UI
                self.ui.messages.push(matrix_message);
            }
            MatrixNotification::SuccessfulLogin => {
                self.switch_mode(Mode::Messages).await?;
            }
            MatrixNotification::KnownRooms(rooms) => {
                for room in rooms {
                    self.ui.room_navigation.push_room(room.id.clone(), room);
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
            Mode::Messages => {}
            Mode::Input => self.ui.input.set_focused(true),
            Mode::Login(login_mode) => {
                // TODO: Find where completed entering of credentials can be handled
                if matches!(login_mode, LoginMode::Completed) {
                    if let Some(login_choice) = self.ui.authentication.selected_login_choice() {
                        let credentials = self.ui.authentication.get_login_credentials();
                        let matrix_action =
                            Event::Matrix(MatrixEvent::Action(MatrixAction::SelectLogin {
                                choice: login_choice,
                                credentials,
                            }));
                        self.event_tx.send(matrix_action).await?;
                    }
                }

                self.ui.authentication.set_login_mode(login_mode.clone());
            }
            Mode::RoomNavigation => {
                self.ui.room_navigation.ensure_initial_selection();
            }
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
            Mode::RoomNavigation => self.ui.room_navigation.handle_key_event(key_event).await,
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        if let Mode::Login(_) = self.mode {
            self.ui.authentication.draw(frame, area);
            return;
        }

        let [top, middle, bottom] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(100),
            Constraint::Length(3),
        ])
        .areas(area);

        let [left, right] =
            Layout::horizontal([Constraint::Length(30), Constraint::Percentage(100)]).areas(middle);

        self.ui.header.draw(frame, top);
        self.ui.room_navigation.draw(frame, left);
        self.ui.messages.draw(frame, right);
        self.ui.input.draw(frame, bottom);
    }
}
