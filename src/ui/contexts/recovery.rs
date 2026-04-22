use tui::{Frame, layout::Rect};

use crate::{
    events::RecoveryMode,
    ui::{
        action::{ContextKey, KeyResult},
        context::{Context, Keybinding, ViewName},
        widgets::recovery::RecoveryWidget,
    },
};

pub struct RecoveryContext {
    widget: RecoveryWidget,
}

impl RecoveryContext {
    pub fn new() -> Self {
        Self {
            widget: RecoveryWidget::new(),
        }
    }

    // --- Domain API for matrix notification handlers ---

    pub fn show_new_key(&mut self, key: String) {
        self.widget.set_recovery_key(key);
        self.widget.set_recovery_mode(RecoveryMode::ShowKey);
    }
}

impl Context for RecoveryContext {
    fn key(&self) -> ContextKey {
        ContextKey::Recovery
    }

    fn view(&self) -> ViewName {
        ViewName::Fullscreen
    }

    fn is_fullscreen(&self) -> bool {
        true
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![]
    }

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyResult {
        self.widget.handle_key(key)
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.widget.draw(frame, area);
    }
}
