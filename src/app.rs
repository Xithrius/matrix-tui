use color_eyre::Result;
use tokio::sync::mpsc::{Sender, channel};
use tracing::debug;
use tui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event as CrosstermEvent, KeyEvent, KeyEventKind},
};

use crate::{
    config::CoreConfig,
    events::{Event, EventHandler},
    matrix::{event::MatrixAction, handler::MatrixHandler},
    ui::{action::KeyEventResult, ui::UiManager},
};

pub struct App {
    pub(crate) running: bool,
    pub(crate) events: EventHandler,
    pub(crate) matrix_tx: Sender<MatrixAction>,
    pub(crate) ui: UiManager,
}

impl App {
    pub fn new(config: &CoreConfig) -> Result<Self> {
        let (event_tx, event_rx) = channel(100);
        let (matrix_tx, matrix_rx) = channel(100);

        let events = EventHandler::new(config, event_tx.clone(), event_rx);
        MatrixHandler::new(config, event_tx, matrix_rx)?;

        Ok(Self {
            running: true,
            events,
            matrix_tx,
            ui: UiManager::new(config),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    pub async fn handle_events(&mut self) -> Result<()> {
        let Some(event) = self.events.next().await else {
            return Ok(());
        };

        match event {
            Event::Tick => self.tick(),
            Event::Crossterm(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                self.handle_key_event(key).await?;
            }
            Event::Crossterm(_) => {}
            Event::Matrix(event) => self.handle_matrix_event(event).await?,
        }

        Ok(())
    }

    fn tick(&mut self) {
        self.ui.tick();
    }

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        debug!("Key event: {:?}", key);

        let action = self.ui.handle_key_event(key);

        if let KeyEventResult::DoAction(action) = action {
            self.execute_action(action).await
        } else {
            Ok(())
        }
    }

    fn draw(&mut self, frame: &mut Frame) {
        self.ui.draw(frame);
    }
}
