use color_eyre::Result;
use tokio::sync::mpsc::{Sender, channel};
use tracing::{debug, error};
use tui::{
    DefaultTerminal, Frame,
    crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    config::CoreConfig,
    events::{Event, EventHandler},
    matrix::{
        event::{MatrixAction, MatrixEvent, MatrixNotification},
        handler::MatrixHandler,
    },
    ui::{
        Action, ContextKey, FocusOpts, KeyResult, StackEntry, Status, Ui,
        context::Context,
        context_manager::{cycle_static_backward, cycle_static_forward},
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
    running: bool,
    events: EventHandler,
    matrix_tx: Sender<MatrixAction>,
    ui: Ui,
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

    // ─── Event loop ──────────────────────────────────────────────────────────

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

    // ─── Key dispatch ─────────────────────────────────────────────────────────

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

    /// Three-phase dispatch: keybinding table → unbound handler → global bindings.
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

    // ─── Action execution ────────────────────────────────────────────────────

    async fn execute_action(&mut self, action: Action) -> Result<()> {
        match action {
            // --- Application ---
            Action::Quit => {
                self.running = false;
            }
            Action::Logout => {
                self.matrix_tx.send(MatrixAction::Logout).await?;
                self.ui
                    .status_line
                    .set_status(Status::Info("Logging out...".to_string()), None);
                self.ui.registry.sidebar.clear();
                self.ui.registry.message_list.clear();
                self.ui.ctx_mgr.enter_login(&mut self.ui.registry);
            }

            // --- Navigation ---
            Action::PopContext => {
                self.ui.ctx_mgr.pop(&mut self.ui.registry);
            }
            Action::PushContext(key, opts) => {
                self.ui.ctx_mgr.push(key, opts, &mut self.ui.registry);
            }

            // --- Panel focus cycling ---
            Action::CycleFocusForward => {
                if let Some(current) = self.ui.ctx_mgr.current_key() {
                    let next = cycle_static_forward(current);
                    self.ui.ctx_mgr.activate_static(next, &mut self.ui.registry);
                }
            }
            Action::CycleFocusBackward => {
                if let Some(current) = self.ui.ctx_mgr.current_key() {
                    let prev = cycle_static_backward(current);
                    self.ui.ctx_mgr.activate_static(prev, &mut self.ui.registry);
                }
            }
            Action::FocusSidebar => {
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::Sidebar, &mut self.ui.registry);
            }
            Action::FocusMessageList => {
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::MessageList, &mut self.ui.registry);
            }
            Action::FocusMessageInput => {
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::MessageInput, &mut self.ui.registry);
            }

            // --- Messaging ---
            Action::SendMessage => {
                let text = self.ui.registry.message_input.take_buffer();
                if !text.trim().is_empty() {
                    let room_id = self.ui.registry.sidebar.get_selected_room_id();
                    if let Some(room_id) = room_id {
                        self.matrix_tx
                            .send(MatrixAction::SendMessage {
                                room_id,
                                message_body: text,
                            })
                            .await?;
                    } else {
                        error!("SendMessage: no room selected");
                    }
                }
            }
            Action::SelectMessage(idx) => {
                self.ui.registry.message_actions.selected_message = Some(idx);
                self.ui.ctx_mgr.push(
                    ContextKey::MessageActions,
                    FocusOpts {
                        selected_message: Some(idx),
                        ..Default::default()
                    },
                    &mut self.ui.registry,
                );
            }
            Action::ReplyToMessage(idx) => {
                self.ui.registry.message_input.reply_to = Some(idx);
                self.ui.ctx_mgr.pop(&mut self.ui.registry);
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::MessageInput, &mut self.ui.registry);
            }
            Action::EditMessage(idx) => {
                // Note: editing by index only; full edit support requires message IDs
                self.ui.registry.message_input.editing = Some(idx);
                self.ui.ctx_mgr.pop(&mut self.ui.registry);
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::MessageInput, &mut self.ui.registry);
            }
            Action::DeleteMessage(idx) => {
                self.ui.ctx_mgr.push(
                    ContextKey::ConfirmDelete,
                    FocusOpts {
                        selected_message: Some(idx),
                        ..Default::default()
                    },
                    &mut self.ui.registry,
                );
            }
            Action::ConfirmDeleteMessage(_idx) => {
                // Full delete support requires message IDs; pop overlays for now
                self.ui.ctx_mgr.pop(&mut self.ui.registry); // close ConfirmDelete
                self.ui.ctx_mgr.pop(&mut self.ui.registry); // close MessageActions
            }
            Action::ScrollMessagesUp => {
                todo!()
            }
            Action::ScrollMessagesDown => {
                todo!()
            }

            // --- Rooms ---
            Action::SelectRoom(id) => {
                self.ui.registry.sidebar.select_room(&id);
                self.ui.registry.message_list.set_active_room(&id);
                self.ui
                    .ctx_mgr
                    .activate_static(ContextKey::MessageInput, &mut self.ui.registry);
                // Request messages for the newly selected room
                self.matrix_tx
                    .send(MatrixAction::GetRoomMessages(id))
                    .await?;
            }
            Action::OpenCreateRoom => {
                self.ui.ctx_mgr.push(
                    ContextKey::CreateRoom,
                    FocusOpts::default(),
                    &mut self.ui.registry,
                );
            }
            Action::ConfirmCreateRoom => {
                let name = self.ui.registry.create_room.take_name_buffer();
                if !name.trim().is_empty() {
                    // Placeholder: room creation via matrix SDK not yet implemented
                    debug!("Create room: {name}");
                }
                self.ui.ctx_mgr.pop(&mut self.ui.registry);
            }
            Action::ScrollRoomsUp => {
                self.ui.registry.sidebar.scroll_up();
            }
            Action::ScrollRoomsDown => {
                self.ui.registry.sidebar.scroll_down();
            }

            // --- Auth ---
            Action::SubmitLogin => {
                let choice = self.ui.registry.login.selected_login_choice();
                let credentials = self.ui.registry.login.take_credentials();
                if let Some(choice) = choice {
                    self.matrix_tx
                        .send(MatrixAction::SelectLogin {
                            choice,
                            credentials,
                        })
                        .await?;
                }
            }

            // --- Recovery ---
            Action::ProvideRecoveryKey(key) => {
                self.matrix_tx
                    .send(MatrixAction::ProvideRecoveryKey(key))
                    .await?;
            }
            Action::ConfirmRecoveryKeySaved => {
                self.matrix_tx
                    .send(MatrixAction::ConfirmRecoveryKeySaved)
                    .await?;
            }
        }

        Ok(())
    }

    // ─── Matrix notification handling ────────────────────────────────────────

    async fn handle_matrix_event(&mut self, event: MatrixEvent) -> Result<()> {
        match event {
            MatrixEvent::Action(action) => {
                self.matrix_tx.send(action).await?;
            }
            MatrixEvent::Notification(notification) => {
                self.handle_matrix_notification(notification).await?;
            }
        }
        Ok(())
    }

    async fn handle_matrix_notification(&mut self, notification: MatrixNotification) -> Result<()> {
        match notification {
            MatrixNotification::RestoringSession => {
                self.ui
                    .status_line
                    .set_status(Status::Info("Restoring session...".to_string()), None);
                self.ui.header.set_loading(true);
                self.ui.header.set_mode("Restoring session".to_string());
            }
            MatrixNotification::SuccessfulSessionRestore => {
                self.ui.status_line.set_status(
                    Status::Info("Session restored, setting up encryption...".to_string()),
                    None,
                );
            }
            MatrixNotification::LoginChoices(choices) => {
                self.ui.registry.login.set_login_choices(choices);
                self.ui
                    .status_line
                    .set_status(Status::Info("Select login option".to_string()), None);
            }
            MatrixNotification::LoggingIn => {
                self.ui
                    .status_line
                    .set_status(Status::Info("Logging in...".to_string()), None);
            }
            MatrixNotification::SuccessfulLogin => {
                self.ui.status_line.set_status(
                    Status::Info("Login successful, setting up encryption...".to_string()),
                    None,
                );
            }
            MatrixNotification::LoginFailed => {
                self.ui.ctx_mgr.enter_login(&mut self.ui.registry);
                self.ui
                    .status_line
                    .set_status(Status::Error("Login failed".to_string()), Some(5));
            }
            MatrixNotification::NeedsRecoveryKey => {
                self.ui.ctx_mgr.enter_recovery(&mut self.ui.registry);
                self.ui.status_line.set_status(
                    Status::Info("Enter your recovery key to restore encryption".to_string()),
                    None,
                );
            }
            MatrixNotification::ShowNewRecoveryKey(key) => {
                self.ui.registry.recovery.show_new_key(key);
                // ctx_mgr is already in Recovery fullscreen; just update the widget.
            }
            MatrixNotification::EncryptionSetupComplete => {
                self.ui.header.set_loading(false);
                self.ui.ctx_mgr.enter_session(&mut self.ui.registry);
                self.ui
                    .status_line
                    .set_status(Status::Info("Encryption configured".to_string()), Some(5));
            }
            MatrixNotification::KnownRooms(rooms) => {
                let first_room = rooms.first().map(|r| r.id.clone());

                for room in rooms {
                    let room_id = room.id.clone();
                    self.ui.registry.sidebar.push_room(room);
                    self.matrix_tx
                        .send(MatrixAction::GetRoomMessages(room_id))
                        .await?;
                }

                if let Some(id) = first_room {
                    self.ui.registry.sidebar.select_room(&id);
                    self.ui.registry.message_list.set_active_room(&id);
                }
            }
            MatrixNotification::RoomMessages {
                room_id,
                mut messages,
            } => {
                messages.sort_by_key(|m| m.datetime);
                for message in messages {
                    self.ui
                        .registry
                        .message_list
                        .push_message(&room_id, message);
                }
            }
            MatrixNotification::Message { room_id, message } => {
                self.ui
                    .registry
                    .message_list
                    .push_message(&room_id, message);
            }
        }

        Ok(())
    }

    // ─── Rendering ───────────────────────────────────────────────────────────

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
                self.ui.registry.sidebar.render(frame, sidebar_area);
                self.ui.registry.message_list.render(frame, messages_area);
                self.ui.registry.message_input.render(frame, input_area);

                // Render overlays on top
                for entry in self.ui.ctx_mgr.stack().to_vec() {
                    if let StackEntry::Overlay(key) = entry {
                        let overlay_area = centered_rect(60, 40, area);
                        self.ui.registry.get_mut(key).render(frame, overlay_area);
                    }
                }

                self.ui.status_line.draw(frame, status_area);
            }
            None => {}
        }
    }

    fn draw_focused_context(&mut self, frame: &mut Frame, area: Rect) {
        if let Some(key) = self.ui.ctx_mgr.current_key() {
            self.ui.registry.get_mut(key).render(frame, area);
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
