use crate::ui::{
    action::FocusOpts,
    context::{ContextKey, StackEntry},
    context_registry::ContextRegistry,
};

/// Tracks which context is currently active on two orthogonal axes:
/// - The push/pop stack for overlays and fullscreen contexts.
/// - Which main screen panel is focused.
pub struct ContextManager {
    stack: Vec<StackEntry>,
    /// Only meaningful when `stack.last()` is `StackEntry::MainScreen`.
    focused_main_panel: ContextKey,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            stack: vec![StackEntry::Fullscreen(ContextKey::Login)],
            focused_main_panel: ContextKey::MessageInput,
        }
    }

    /// Start at the login screen (called at app start or after logout).
    pub fn enter_login(&mut self, registry: &mut ContextRegistry) {
        if let Some(prev) = self.focused_ctx_key() {
            registry.get_mut(prev).on_blur();
        }
        self.stack.clear();
        self.stack.push(StackEntry::Fullscreen(ContextKey::Login));
        registry
            .get_mut(ContextKey::Login)
            .on_focus(FocusOpts::default());
    }

    /// Enter the main messaging session (called after successful login / encryption setup).
    pub fn enter_session(&mut self, registry: &mut ContextRegistry) {
        if let Some(prev) = self.focused_ctx_key() {
            registry.get_mut(prev).on_blur();
        }
        self.stack.clear();
        self.stack.push(StackEntry::MainScreen);
        self.focused_main_panel = ContextKey::MessageInput;
        registry
            .get_mut(ContextKey::MessageInput)
            .on_focus(FocusOpts::default());
    }

    /// Enter the recovery screen (fullscreen, replaces whatever is on the stack).
    pub fn enter_recovery(&mut self, registry: &mut ContextRegistry) {
        if let Some(prev) = self.focused_ctx_key() {
            registry.get_mut(prev).on_blur();
        }
        self.stack.clear();
        self.stack
            .push(StackEntry::Fullscreen(ContextKey::Recovery));
        registry
            .get_mut(ContextKey::Recovery)
            .on_focus(FocusOpts::default());
    }

    /// Push an overlay or fullscreen context onto the stack.
    pub fn push(&mut self, entry: StackEntry, opts: FocusOpts, registry: &mut ContextRegistry) {
        let key = match &entry {
            StackEntry::Overlay(k) | StackEntry::Fullscreen(k) => *k,
            StackEntry::MainScreen => return,
        };

        if let Some(prev) = self.focused_ctx_key() {
            registry.get_mut(prev).on_blur();
        }

        self.stack.push(entry);
        registry.get_mut(key).on_focus(opts);
    }

    /// Pop the top context. No-op if only the base `MainScreen` layer remains.
    pub fn pop(&mut self, registry: &mut ContextRegistry) {
        if self.stack.len() <= 1 {
            return;
        }

        if let Some(top_key) = self.focused_ctx_key() {
            registry.get_mut(top_key).on_blur();
        }

        self.stack.pop();

        if let Some(next_key) = self.focused_ctx_key() {
            registry.get_mut(next_key).on_focus(FocusOpts::default());
        }
    }

    /// Switch which main screen panel is focused without touching the stack.
    /// No-op if the top of the stack is not the `MainScreen` layer.
    pub fn activate_main_screen(&mut self, key: ContextKey, registry: &mut ContextRegistry) {
        if !matches!(self.stack.last(), Some(StackEntry::MainScreen)) {
            return;
        }
        registry.get_mut(self.focused_main_panel).on_blur();
        self.focused_main_panel = key;
        registry.get_mut(key).on_focus(FocusOpts::default());
    }

    /// The key of whichever context is currently focused.
    pub fn focused_ctx_key(&self) -> Option<ContextKey> {
        match self.stack.last()? {
            StackEntry::MainScreen => Some(self.focused_main_panel),
            StackEntry::Overlay(key) | StackEntry::Fullscreen(key) => Some(*key),
        }
    }

    /// Expose the stack for rendering (read-only).
    pub fn stack(&self) -> &[StackEntry] {
        &self.stack
    }

    /// Returns a display string for the current focus state (used by the header).
    pub fn mode_display(&self) -> &'static str {
        match self.stack.last() {
            Some(StackEntry::Fullscreen(ContextKey::Login)) => "Login",
            Some(StackEntry::Fullscreen(ContextKey::Recovery)) => "Recovery",
            Some(StackEntry::Overlay(ContextKey::MessageActions)) => "Message Actions",
            Some(StackEntry::Overlay(ContextKey::CreateRoom)) => "Create Room",
            Some(StackEntry::Overlay(ContextKey::ConfirmDelete)) => "Confirm Delete",
            Some(StackEntry::MainScreen) => match self.focused_main_panel {
                ContextKey::Sidebar => "Rooms",
                ContextKey::MessageList => "Messages",
                ContextKey::MessageInput => "Input",
                _ => "Session",
            },
            _ => "",
        }
    }
}

/// The three main screen panels that participate in Tab cycling.
const MAIN_SCREEN_CYCLE_ORDER: &[ContextKey] = &[
    ContextKey::Sidebar,
    ContextKey::MessageList,
    ContextKey::MessageInput,
];

pub fn cycle_main_screen_forward(current: ContextKey) -> ContextKey {
    let pos = MAIN_SCREEN_CYCLE_ORDER
        .iter()
        .position(|k| *k == current)
        .unwrap_or(0);
    MAIN_SCREEN_CYCLE_ORDER[(pos + 1) % MAIN_SCREEN_CYCLE_ORDER.len()]
}

pub fn cycle_main_screen_backward(current: ContextKey) -> ContextKey {
    let pos = MAIN_SCREEN_CYCLE_ORDER
        .iter()
        .position(|k| *k == current)
        .unwrap_or(0);
    MAIN_SCREEN_CYCLE_ORDER
        [(pos + MAIN_SCREEN_CYCLE_ORDER.len() - 1) % MAIN_SCREEN_CYCLE_ORDER.len()]
}
