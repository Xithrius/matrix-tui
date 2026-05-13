use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout},
};

use crate::{
    config::CoreConfig,
    ui::{
        action::{Action, KeyEventResult},
        context::{Context, StackEntry},
        context_manager::ContextManager,
        widgets::{HeaderWidget, StatusLineWidget, status_line::Status},
    },
};

/// Global keybindings checked as a final fallthrough when the focused context
/// does not consume a key event.
const GLOBAL_KEYBINDINGS: &[(KeyCode, KeyModifiers, Action)] = &[
    (KeyCode::Char('q'), KeyModifiers::CONTROL, Action::Quit),
    (KeyCode::Char('c'), KeyModifiers::CONTROL, Action::Quit),
    (KeyCode::Char('l'), KeyModifiers::CONTROL, Action::Logout),
];

pub struct UiManager {
    pub ctx_mgr: ContextManager,
    /// Always-visible title bar (not part of the focus system).
    pub header: HeaderWidget,
    /// Always-visible status bar (not part of the focus system).
    pub status_line: StatusLineWidget,
}

impl UiManager {
    pub fn new(config: &CoreConfig) -> Self {
        Self {
            ctx_mgr: ContextManager::new(),
            header: HeaderWidget::new(config, "matrix-tui".to_string()),
            status_line: StatusLineWidget::new(
                Some(Status::Info("Launching...".to_string())),
                None,
            ),
        }
    }

    pub fn tick(&mut self) {
        self.header.increment_spinner();
        self.status_line.tick();
    }

    pub fn handle_key_event(&mut self, key: KeyEvent) -> KeyEventResult {
        let Some(focused_ctx_key) = self.ctx_mgr.focused_ctx_key() else {
            return KeyEventResult::NotConsumed;
        };

        let ctx = self.ctx_mgr.registry.get_mut(focused_ctx_key);

        // Phase 1: declarative keybinding table
        for binding in ctx.keybindings() {
            if binding.matches(key) {
                return KeyEventResult::DoAction(binding.action);
            }
        }

        // Phase 2: context-specific unbound key handler
        match ctx.handle_unbound_key(key) {
            KeyEventResult::NotConsumed => {}
            other => return other,
        }

        // Phase 3: global keybindings
        for (code, mods, action) in GLOBAL_KEYBINDINGS {
            if key.code == *code && key.modifiers == *mods {
                return KeyEventResult::DoAction(action.clone());
            }
        }

        // No keybinding matched
        KeyEventResult::NotConsumed
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        // Update header mode text from manager state
        let mode_str = self.ctx_mgr.mode_display().to_string();
        self.header.set_mode(mode_str);

        match self.ctx_mgr.stack().last() {
            Some(StackEntry::Fullscreen(_)) => {
                let [content_area, status_area] =
                    Layout::vertical([Constraint::Percentage(100), Constraint::Length(1)])
                        .areas(area);

                if let Some(key) = self.ctx_mgr.focused_ctx_key() {
                    self.ctx_mgr.registry.get_mut(key).draw(frame, content_area);
                }

                self.status_line.draw(frame, status_area);
            }
            Some(StackEntry::MainScreen | StackEntry::Overlay(_)) => {
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

                self.header.draw(frame, header_area);
                self.ctx_mgr.registry.room_list.draw(frame, sidebar_area);
                self.ctx_mgr
                    .registry
                    .message_list
                    .draw(frame, messages_area);
                self.ctx_mgr.registry.message_input.draw(frame, input_area);

                // Render overlays on top
                for entry in self.ctx_mgr.stack().to_vec() {
                    if let StackEntry::Overlay(key) = entry {
                        let [_, overlay_area, _] = Layout::vertical([
                            Constraint::Fill(1),
                            Constraint::Fill(3),
                            Constraint::Fill(1),
                        ])
                        .areas(area);

                        let [_, overlay_area, _] = Layout::horizontal([
                            Constraint::Fill(1),
                            Constraint::Fill(3),
                            Constraint::Fill(1),
                        ])
                        .areas(overlay_area);

                        self.ctx_mgr.registry.get_mut(key).draw(frame, overlay_area);
                    }
                }

                self.status_line.draw(frame, status_area);
            }
            None => {}
        }
    }
}
