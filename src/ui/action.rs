#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ContextKey {
    // Fullscreen contexts
    Login,
    Recovery,
    // Static panels (always visible during a session)
    Sidebar,
    MessageList,
    MessageInput,
    // Overlay contexts
    MessageActions,
    CreateRoom,
    ConfirmDelete,
}

#[derive(Debug, Clone, Default)]
pub struct FocusOpts {
    pub selected_message: Option<usize>,
    pub prefill: Option<String>,
}

/// The result of a key dispatch attempt.
#[derive(Debug, Clone)]
pub enum KeyResult {
    /// A recognised keybinding or unbound handler produced an action.
    DoAction(Action),
    /// The key was consumed (e.g. text input) but produced no action.
    Consumed,
    /// The key was not handled by this context.
    NotConsumed,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    // --- Navigation ---
    /// Pop the top context off the stack (Escape).
    PopContext,
    /// Push an overlay or fullscreen context.
    PushContext(ContextKey, FocusOpts),

    // --- Panel focus cycling ---
    CycleFocusForward,
    CycleFocusBackward,
    FocusSidebar,
    FocusMessageList,
    FocusMessageInput,

    // --- Messaging ---
    SendMessage,
    /// Select the message at the given list index, opening `MessageActions`.
    SelectMessage(usize),
    ReplyToMessage(usize),
    EditMessage(usize),
    DeleteMessage(usize),
    ConfirmDeleteMessage(usize),
    ScrollMessagesUp,
    ScrollMessagesDown,

    // --- Rooms ---
    SelectRoom(String),
    OpenCreateRoom,
    ConfirmCreateRoom,
    ScrollRoomsUp,
    ScrollRoomsDown,

    // --- Auth ---
    SubmitLogin,

    // --- Recovery ---
    ProvideRecoveryKey(String),
    ConfirmRecoveryKeySaved,

    // --- Application ---
    Quit,
    Logout,
}
