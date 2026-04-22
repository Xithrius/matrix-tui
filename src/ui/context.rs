use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
};

use crate::ui::action::{Action, ContextKey, FocusOpts, KeyResult};

/// The named terminal region a context renders into.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ViewName {
    Sidebar,
    Messages,
    Input,
    Overlay,
    Fullscreen,
}

/// A single entry in a context's declarative keybinding table.
pub struct Keybinding {
    pub key: KeyCode,
    pub mods: KeyModifiers,
    pub action: Action,
    /// Short human-readable description for the help bar / cheatsheet.
    #[allow(dead_code)]
    pub description: &'static str,
}

impl Keybinding {
    pub const fn new(
        key: KeyCode,
        mods: KeyModifiers,
        action: Action,
        description: &'static str,
    ) -> Self {
        Self {
            key,
            mods,
            action,
            description,
        }
    }

    pub fn matches(&self, event: KeyEvent) -> bool {
        self.key == event.code && self.mods == event.modifiers
    }
}

/// The core trait every UI context must implement.
#[allow(dead_code)]
///
/// A context is a singleton that owns one logical component's draw logic,
/// keybindings, and unbound key handling. It wraps a widget internally but
/// exposes only domain-appropriate methods to callers.
pub trait Context {
    fn key(&self) -> ContextKey;
    fn view(&self) -> ViewName;

    /// True if this context occupies the full terminal, hiding static panels.
    fn is_fullscreen(&self) -> bool {
        false
    }

    /// True if this context floats above the static panel layer.
    fn is_overlay(&self) -> bool {
        false
    }

    /// Declarative keybinding table. Checked first during dispatch.
    fn keybindings(&self) -> Vec<Keybinding>;

    /// Called when no keybinding matched. Override for text-input contexts
    /// and any other generic key handling. Returns `NotConsumed` by default.
    fn handle_unbound_key(&mut self, _key: KeyEvent) -> KeyResult {
        KeyResult::NotConsumed
    }

    /// Called when this context gains focus.
    fn on_focus(&mut self, _opts: FocusOpts) {}

    /// Called when this context loses focus.
    fn on_blur(&mut self) {}

    /// Draw this context into the given area.
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}
