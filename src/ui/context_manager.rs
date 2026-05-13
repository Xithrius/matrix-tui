use crate::ui::{
    action::{ContextKey, FocusOpts},
    context_registry::ContextRegistry,
};

#[derive(Debug, Clone)]
pub enum StackEntry {
    /// The normal in-session layer: sidebar + message list + input all visible.
    MainScreen,
    /// An overlay context floating above the static layer.
    Overlay(ContextKey),
    /// A fullscreen context that hides the static layer entirely.
    Fullscreen(ContextKey),
}

/// Tracks which context is currently active on two orthogonal axes:
/// - The push/pop stack for overlays and fullscreen contexts.
/// - Which static panel is focused when the top of the stack is `MainScreen`.
pub struct ContextManager {
    stack: Vec<StackEntry>,
    /// Only meaningful when `stack.last()` is `StackEntry::MainScreen`.
    focused_static: ContextKey,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            stack: vec![StackEntry::Fullscreen(ContextKey::Login)],
            focused_static: ContextKey::MessageInput,
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
        self.focused_static = ContextKey::MessageInput;
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

    /// Push an overlay (or fullscreen) context onto the stack.
    pub fn push(&mut self, key: ContextKey, opts: FocusOpts, registry: &mut ContextRegistry) {
        let is_fullscreen = registry.get(key).is_fullscreen();
        let entry = if is_fullscreen {
            StackEntry::Fullscreen(key)
        } else {
            StackEntry::Overlay(key)
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

    /// Switch which static panel is focused without touching the stack.
    /// No-op if the top of the stack is not the `MainScreen` layer.
    pub fn activate_static(&mut self, key: ContextKey, registry: &mut ContextRegistry) {
        if !matches!(self.stack.last(), Some(StackEntry::MainScreen)) {
            return;
        }
        registry.get_mut(self.focused_static).on_blur();
        self.focused_static = key;
        registry.get_mut(key).on_focus(FocusOpts::default());
    }

    /// The key of whichever context is currently focused.
    pub fn focused_ctx_key(&self) -> Option<ContextKey> {
        match self.stack.last()? {
            StackEntry::MainScreen => Some(self.focused_static),
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
            Some(StackEntry::MainScreen) => match self.focused_static {
                ContextKey::Sidebar => "Rooms",
                ContextKey::MessageList => "Messages",
                ContextKey::MessageInput => "Input",
                _ => "Session",
            },
            _ => "",
        }
    }
}

/// The three static panels that participate in Tab cycling.
const STATIC_CYCLE_ORDER: &[ContextKey] = &[
    ContextKey::Sidebar,
    ContextKey::MessageList,
    ContextKey::MessageInput,
];

pub fn cycle_static_forward(current: ContextKey) -> ContextKey {
    let pos = STATIC_CYCLE_ORDER
        .iter()
        .position(|k| *k == current)
        .unwrap_or(0);
    STATIC_CYCLE_ORDER[(pos + 1) % STATIC_CYCLE_ORDER.len()]
}

pub fn cycle_static_backward(current: ContextKey) -> ContextKey {
    let pos = STATIC_CYCLE_ORDER
        .iter()
        .position(|k| *k == current)
        .unwrap_or(0);
    STATIC_CYCLE_ORDER[(pos + STATIC_CYCLE_ORDER.len() - 1) % STATIC_CYCLE_ORDER.len()]
}
