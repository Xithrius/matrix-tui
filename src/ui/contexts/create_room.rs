use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
};

use crate::ui::{
    action::{Action, FocusOpts, KeyEventResult},
    context::{Context, Keybinding},
    widgets::user_input::UserInputWidget,
};

pub struct CreateRoomContext {
    widget: UserInputWidget,
}

impl CreateRoomContext {
    pub fn new() -> Self {
        Self {
            widget: UserInputWidget::new(Some("Room Name")),
        }
    }

    // --- Domain API for execute_action ---

    /// Take the room name buffer, clearing it.
    pub fn take_name_buffer(&mut self) -> String {
        let name = self.widget.get_input().to_owned();
        self.widget.clear();
        name
    }
}

impl Context for CreateRoomContext {
    fn title(&self) -> &'static str {
        "Create room"
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![
            Keybinding::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
                Action::ConfirmCreateRoom,
                "Create room",
            ),
            Keybinding::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
                Action::PopContext,
                "Cancel",
            ),
        ]
    }

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyEventResult {
        self.widget.handle_key(key)
    }

    fn on_focus(&mut self, opts: FocusOpts) {
        if let Some(prefill) = opts.prefill {
            self.widget.set_input(&prefill);
        }
        self.widget.set_focused(true);
    }

    fn on_blur(&mut self) {
        self.widget.set_focused(false);
        self.widget.clear();
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let [_, input_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(area);

        self.widget.draw(frame, input_area);
    }
}
