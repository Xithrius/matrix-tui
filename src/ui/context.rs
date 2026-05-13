use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
};

use crate::ui::action::{Action, FocusOpts, KeyEventResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextKey {
    // Fullscreen contexts
    Login,
    Recovery,
    // MainScreen panels (always visible during a session)
    RoomList,
    MessageList,
    MessageInput,
    // Overlay contexts
    MessageActions,
    CreateRoom,
    ConfirmDelete,
}

/// Describes how a context is layered on the screen when it is pushed onto the stack.
#[derive(Debug, Clone)]
pub enum StackEntry {
    /// The normal in-session layer: room list + message list + input all visible.
    MainScreen,
    /// An overlay context floating above the static layer.
    Overlay(ContextKey),
    /// A fullscreen context that hides the static layer entirely.
    Fullscreen(ContextKey),
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
///
/// A context is a singleton that owns one logical component's draw logic,
/// keybindings, and unbound key handling. It wraps a widget internally but
/// exposes only domain-appropriate methods to callers.
///
/// Contexts have no knowledge of where they are rendered; that
/// responsibility belongs entirely to `UiManager` and `ContextManager`.
pub trait Context {
    /// Returns a display string as a title for the context.
    fn title(&self) -> &'static str;

    /// Declarative keybinding table. Checked first during dispatch.
    fn keybindings(&self) -> Vec<Keybinding>;

    /// Called when no keybinding matched. Override for text-input contexts
    /// and any other generic key handling. Returns `NotConsumed` by default.
    fn handle_unbound_key(&mut self, _key: KeyEvent) -> KeyEventResult {
        KeyEventResult::NotConsumed
    }

    /// Called when this context gains focus.
    fn on_focus(&mut self, _opts: FocusOpts) {}

    /// Called when this context loses focus.
    fn on_blur(&mut self) {}

    /// Draw this context into the given area.
    fn draw(&mut self, frame: &mut Frame, area: Rect);
}
