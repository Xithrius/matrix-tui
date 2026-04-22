use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
};

use crate::{
    matrix::models::MatrixRoom,
    ui::{
        action::{Action, ContextKey, KeyResult},
        context::{Context, Keybinding, ViewName},
        widgets::rooms::RoomNavigationWidget,
    },
};

pub struct SidebarContext {
    widget: RoomNavigationWidget,
}

impl SidebarContext {
    pub fn new() -> Self {
        Self {
            widget: RoomNavigationWidget::new(),
        }
    }

    // --- Domain API for the notification path ---

    pub fn push_room(&mut self, room: MatrixRoom) {
        self.widget.push_room(room);
    }

    /// Mark a room as the currently active room (highlighted in the list).
    pub fn select_room(&mut self, room_id: &String) {
        self.widget.set_selected_room_id(room_id);
    }

    pub fn get_selected_room_id(&self) -> Option<String> {
        self.widget.get_selected_room_id()
    }

    pub fn clear(&mut self) {
        self.widget.clear();
    }

    // --- Domain API for execute_action ---

    pub fn scroll_up(&mut self) {
        self.widget.select_prev();
    }

    pub fn scroll_down(&mut self) {
        self.widget.select_next();
    }
}

impl Context for SidebarContext {
    fn key(&self) -> ContextKey {
        ContextKey::Sidebar
    }

    fn view(&self) -> ViewName {
        ViewName::Sidebar
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        vec![
            Keybinding::new(
                KeyCode::Up,
                KeyModifiers::NONE,
                Action::ScrollRoomsUp,
                "Scroll rooms up",
            ),
            Keybinding::new(
                KeyCode::Char('k'),
                KeyModifiers::NONE,
                Action::ScrollRoomsUp,
                "Scroll rooms up",
            ),
            Keybinding::new(
                KeyCode::Down,
                KeyModifiers::NONE,
                Action::ScrollRoomsDown,
                "Scroll rooms down",
            ),
            Keybinding::new(
                KeyCode::Char('j'),
                KeyModifiers::NONE,
                Action::ScrollRoomsDown,
                "Scroll rooms down",
            ),
            Keybinding::new(
                KeyCode::Tab,
                KeyModifiers::NONE,
                Action::CycleFocusForward,
                "Cycle focus forward",
            ),
            // Shift+Tab arrives as BackTab with no modifiers in crossterm
            Keybinding::new(
                KeyCode::BackTab,
                KeyModifiers::NONE,
                Action::CycleFocusBackward,
                "Cycle focus backward",
            ),
        ]
    }

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyResult {
        // Enter must be resolved at call time to embed the actual room id.
        if key.code == KeyCode::Enter && key.modifiers == KeyModifiers::NONE {
            return self
                .widget
                .highlighted_room_id()
                .map_or(KeyResult::Consumed, |id| {
                    KeyResult::DoAction(Action::SelectRoom(id))
                });
        }
        KeyResult::NotConsumed
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.widget.draw(frame, area);
    }
}
