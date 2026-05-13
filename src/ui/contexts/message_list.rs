use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};

use crate::{
    matrix::models::MatrixMessage,
    ui::{
        action::{Action, KeyEventResult},
        context::{Context, ContextKey, Keybinding, ViewName},
        widgets::messages::MessagesWidget,
    },
};

pub struct MessageListContext {
    widget: MessagesWidget,
}

impl MessageListContext {
    pub fn new() -> Self {
        Self {
            widget: MessagesWidget::new(),
        }
    }

    // --- Domain API for the notification path ---

    pub fn push_message(&mut self, room_id: &String, message: MatrixMessage) {
        self.widget.push_message(room_id, message);
    }

    pub fn set_active_room(&mut self, room_id: &String) {
        self.widget.set_active_room(room_id);
    }

    pub fn clear(&mut self) {
        self.widget.clear();
    }
}

impl Context for MessageListContext {
    fn key(&self) -> ContextKey {
        ContextKey::MessageList
    }

    fn view(&self) -> ViewName {
        ViewName::Messages
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![
            Keybinding::new(
                KeyCode::Tab,
                KeyModifiers::NONE,
                Action::CycleFocusForward,
                "Cycle focus forward",
            ),
            Keybinding::new(
                KeyCode::BackTab,
                KeyModifiers::NONE,
                Action::CycleFocusBackward,
                "Cycle focus backward",
            ),
        ]
    }

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyEventResult {
        // Enter opens message actions for the highlighted message.
        if key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE {
            return self
                .widget
                .highlighted_index()
                .map_or(KeyEventResult::Consumed, |idx| {
                    KeyEventResult::DoAction(Action::SelectMessage(idx))
                });
        }
        KeyEventResult::NotConsumed
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.widget.draw(frame, area);
    }
}
