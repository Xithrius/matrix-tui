use crate::ui::{
    action::ContextKey,
    context::Context,
    contexts::{
        confirm_delete::ConfirmDeleteContext, create_room::CreateRoomContext, login::LoginContext,
        message_actions::MessageActionsContext, message_input::MessageInputContext,
        message_list::MessageListContext, recovery::RecoveryContext, room_list::RoomListContext,
    },
};

/// Owns every context for the full lifetime of the application.
/// The context manager and dispatch loop index into this registry by `ContextKey`.
pub struct ContextRegistry {
    pub login: LoginContext,
    pub recovery: RecoveryContext,
    pub sidebar: RoomListContext,
    pub message_list: MessageListContext,
    pub message_input: MessageInputContext,
    pub message_actions: MessageActionsContext,
    pub create_room: CreateRoomContext,
    pub confirm_delete: ConfirmDeleteContext,
}

impl ContextRegistry {
    pub fn new() -> Self {
        Self {
            login: LoginContext::new(),
            recovery: RecoveryContext::new(),
            sidebar: RoomListContext::new(),
            message_list: MessageListContext::new(),
            message_input: MessageInputContext::new(),
            message_actions: MessageActionsContext::new(),
            create_room: CreateRoomContext::new(),
            confirm_delete: ConfirmDeleteContext::new(),
        }
    }

    pub fn get(&self, key: ContextKey) -> &dyn Context {
        match key {
            ContextKey::Login => &self.login,
            ContextKey::Recovery => &self.recovery,
            ContextKey::Sidebar => &self.sidebar,
            ContextKey::MessageList => &self.message_list,
            ContextKey::MessageInput => &self.message_input,
            ContextKey::MessageActions => &self.message_actions,
            ContextKey::CreateRoom => &self.create_room,
            ContextKey::ConfirmDelete => &self.confirm_delete,
        }
    }

    pub fn get_mut(&mut self, key: ContextKey) -> &mut dyn Context {
        match key {
            ContextKey::Login => &mut self.login,
            ContextKey::Recovery => &mut self.recovery,
            ContextKey::Sidebar => &mut self.sidebar,
            ContextKey::MessageList => &mut self.message_list,
            ContextKey::MessageInput => &mut self.message_input,
            ContextKey::MessageActions => &mut self.message_actions,
            ContextKey::CreateRoom => &mut self.create_room,
            ContextKey::ConfirmDelete => &mut self.confirm_delete,
        }
    }
}
