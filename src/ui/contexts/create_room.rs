use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType},
};

use crate::ui::{
    action::{Action, ContextKey, FocusOpts, KeyResult},
    context::{Context, Keybinding, ViewName},
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
    fn key(&self) -> ContextKey {
        ContextKey::CreateRoom
    }

    fn view(&self) -> ViewName {
        ViewName::Overlay
    }

    fn is_overlay(&self) -> bool {
        true
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

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyResult {
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
        let overlay = centered_rect(50, 30, area);
        let [_, input_area] =
            Layout::vertical([Constraint::Percentage(100), Constraint::Length(3)]).areas(overlay);

        // Draw a backdrop block for the overlay
        frame.render_widget(
            Block::bordered()
                .title("Create Room")
                .border_type(BorderType::Rounded),
            overlay,
        );

        self.widget.draw(frame, input_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(vertical[1])[1]
}
