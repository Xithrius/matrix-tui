use color_eyre::Result;
use crossbeam_channel::{Sender, unbounded};
use tracing::debug;
use tui::{
    DefaultTerminal, Frame,
    crossterm::{self, event::KeyEvent},
    layout::Rect,
};

use crate::{
    config::core::CoreConfig,
    event::{Event, EventHandler, InternalEvent, Mode},
    ui::{
        component::Component, header::HeaderWidget, input::InputWidget, messages::MessagesWidget,
    },
};

pub struct Ui {
    header: HeaderWidget,
    messages: MessagesWidget,
    input: InputWidget,
}

impl Ui {
    pub fn new(event_tx: Sender<Event>) -> Self {
        Self {
            header: HeaderWidget::new("Placeholder motd".to_string()),
            messages: MessagesWidget::new(event_tx.clone()),
            input: InputWidget::new(event_tx),
        }
    }
}

pub struct App {
    running: bool,
    events: EventHandler,

    mode: Mode,
    ui: Ui,
}

impl App {
    pub fn new(config: &CoreConfig) -> Self {
        let (event_tx, event_rx) = unbounded();

        let ui = Ui::new(event_tx.clone());

        let events = EventHandler::new(config, event_tx, event_rx);

        Self {
            running: true,
            events,
            mode: Mode::default(),
            ui,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame, frame.area()))?;
            self.handle_events()?;
        }

        Ok(())
    }

    pub fn handle_events(&mut self) -> Result<()> {
        match self.events.next()? {
            Event::Tick => self.tick(),
            Event::Crossterm(event) => match event {
                crossterm::event::Event::Key(key_event)
                    if key_event.kind == crossterm::event::KeyEventKind::Press =>
                {
                    self.handle_key_event(key_event)?;
                }
                _ => {}
            },
            Event::Internal(app_event) => match app_event {
                InternalEvent::SwitchMode(mode) => self.switch_mode(mode),
                InternalEvent::Quit => self.quit(),
                InternalEvent::SendMessage(message) => self.ui.messages.push(message),
            },
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

    pub fn switch_mode(&mut self, mode: Mode) {
        debug!("Switching to mode: {:?}", mode);

        match &mode {
            Mode::Input => self.ui.input.set_focused(true),
            Mode::Messages => {}
        }

        self.mode = mode;
    }
}

impl Component for App {
    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        debug!("Received key event: {:?}", key_event);

        match &self.mode {
            Mode::Messages => self.ui.messages.handle_key_event(key_event),
            Mode::Input => self.ui.input.handle_key_event(key_event),
        }
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.ui.header.draw(frame, area);
        self.ui.messages.draw(frame, area);
        self.ui.input.draw(frame, area);
    }
}
