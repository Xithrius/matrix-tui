use color_eyre::Result;
use tokio::sync::mpsc::{Sender, channel};
use tracing::debug;
use tui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    config::CoreConfig,
    events::{Event, EventHandler},
    matrix::{event::MatrixAction, handler::MatrixHandler},
    ui::{
        action::{Action, KeyResult},
        context::Context,
        context_manager::StackEntry,
        ui::Ui,
    },
};

/// Global keybindings checked as a final fallthrough when the focused context
/// does not consume a key event.
const GLOBAL_KEYBINDINGS: &[(KeyCode, KeyModifiers, Action)] = &[
    (KeyCode::Char('q'), KeyModifiers::CONTROL, Action::Quit),
    (KeyCode::Char('c'), KeyModifiers::CONTROL, Action::Quit),
    (KeyCode::Char('l'), KeyModifiers::CONTROL, Action::Logout),
];

pub struct App {
    pub(crate) running: bool,
    pub(crate) events: EventHandler,
    pub(crate) matrix_tx: Sender<MatrixAction>,
    pub(crate) ui: Ui,
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
            ui: Ui::new(config),
        })
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    // --- Event loop ---

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
        self.ui.header.increment_spinner();
        self.ui.status_line.tick();
    }

    // --- Key dispatch ---

    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        debug!("Key event: {:?}", key);

        let Some(focused_key) = self.ui.ctx_mgr.current_key() else {
            return Ok(());
        };

        // Borrow the context, dispatch, then release the borrow before execute_action.
        let result = {
            let ctx = self.ui.registry.get_mut(focused_key);
            Self::dispatch(key, ctx)
        };

        if let KeyResult::DoAction(action) = result {
            self.execute_action(action).await?;
        }

        Ok(())
    }

    /// Three-phase dispatch: keybinding table -> unbound handler -> global bindings.
    fn dispatch(key: KeyEvent, ctx: &mut dyn Context) -> KeyResult {
        // Phase 1: declarative keybinding table
        for binding in ctx.keybindings() {
            if binding.matches(key) {
                return KeyResult::DoAction(binding.action);
            }
        }
        // Phase 2: context-specific unbound key handler
        match ctx.handle_unbound_key(key) {
            KeyResult::NotConsumed => {}
            other => return other,
        }
        // Phase 3: global keybindings (always available)
        for (code, mods, action) in GLOBAL_KEYBINDINGS {
            if key.code == *code && key.modifiers == *mods {
                return KeyResult::DoAction(action.clone());
            }
        }
        KeyResult::NotConsumed
    }

    // --- Rendering ---

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Update header mode text from manager state
        let mode_str = self.ui.ctx_mgr.mode_display().to_string();
        self.ui.header.set_mode(mode_str);

        match self.ui.ctx_mgr.stack().last() {
            Some(StackEntry::Fullscreen(_)) => {
                let [content_area, status_area] =
                    Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                        .areas(area);

                self.draw_focused_context(frame, content_area);
                self.ui.status_line.draw(frame, status_area);
            }
            Some(StackEntry::Static | StackEntry::Overlay(_)) => {
                let [header_area, content_area, status_area] = Layout::vertical([
                    Constraint::Length(1),
                    Constraint::Percentage(100),
                    Constraint::Length(1),
                ])
                .areas(area);

                let [sidebar_area, rest_area] =
                    Layout::horizontal([Constraint::Length(30), Constraint::Percentage(100)])
                        .areas(content_area);

                let [messages_area, input_area] =
                    Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)])
                        .areas(rest_area);

                self.ui.header.draw(frame, header_area);
                self.ui.registry.sidebar.draw(frame, sidebar_area);
                self.ui.registry.message_list.draw(frame, messages_area);
                self.ui.registry.message_input.draw(frame, input_area);

                // Render overlays on top
                for entry in self.ui.ctx_mgr.stack().to_vec() {
                    if let StackEntry::Overlay(key) = entry {
                        let overlay_area = centered_rect(60, 40, area);
                        self.ui.registry.get_mut(key).draw(frame, overlay_area);
                    }
                }

                self.ui.status_line.draw(frame, status_area);
            }
            None => {}
        }
    }

    fn draw_focused_context(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(key) = self.ui.ctx_mgr.current_key() {
            self.ui.registry.get_mut(key).draw(frame, area);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
