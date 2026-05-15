use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};

use crate::ui::{
    action::{Action, FocusOpts, KeyEventResult},
    context::{Context, Keybinding},
    widgets::UserInputWidget,
};

pub struct MessageInputContext {
    widget: UserInputWidget,
    /// Message being replied to, if any.
    pub reply_to: Option<usize>,
    /// Message being edited, if any.
    pub editing: Option<usize>,
}

impl MessageInputContext {
    pub fn new() -> Self {
        Self {
            widget: UserInputWidget::new(Some("Input")),
            reply_to: None,
            editing: None,
        }
    }

    // --- Domain API for execute_action ---

    /// Take the current buffer content (clearing it) for sending.
    pub fn take_buffer(&mut self) -> String {
        let text = self.widget.get_input().to_owned();
        self.widget.clear();
        self.reply_to = None;
        self.editing = None;
        text
    }

    /// Pre-fill the buffer (used when editing an existing message).
    #[allow(dead_code)]
    pub fn set_buffer(&mut self, text: &str) {
        self.widget.set_input(text);
    }
}

impl Context for MessageInputContext {
    fn title(&self) -> &'static str {
        "Message input"
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![
            Keybinding::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
                Action::SendMessage,
                "Send message",
            ),
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
        self.widget.handle_key(key)
    }

    fn on_focus(&mut self, _opts: FocusOpts) {
        self.widget.set_focused(true);
    }

    fn on_blur(&mut self) {
        self.widget.set_focused(false);
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        self.widget.draw(frame, area);
    }
}
