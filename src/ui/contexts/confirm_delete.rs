use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, Paragraph},
};

use crate::ui::{
    action::{Action, ContextKey, FocusOpts},
    context::{Context, Keybinding, ViewName},
};

pub struct ConfirmDeleteContext {
    pub selected_message: Option<usize>,
}

impl ConfirmDeleteContext {
    pub const fn new() -> Self {
        Self {
            selected_message: None,
        }
    }
}

impl Context for ConfirmDeleteContext {
    fn key(&self) -> ContextKey {
        ContextKey::ConfirmDelete
    }

    fn view(&self) -> ViewName {
        ViewName::Overlay
    }

    fn is_overlay(&self) -> bool {
        true
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        let idx = self.selected_message.unwrap_or(0);
        vec![
            Keybinding::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
                Action::ConfirmDeleteMessage(idx),
                "Confirm delete",
            ),
            Keybinding::new(
                KeyCode::Char('y'),
                KeyModifiers::NONE,
                Action::ConfirmDeleteMessage(idx),
                "Confirm delete",
            ),
            Keybinding::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
                Action::PopContext,
                "Cancel",
            ),
            Keybinding::new(
                KeyCode::Char('n'),
                KeyModifiers::NONE,
                Action::PopContext,
                "Cancel",
            ),
        ]
    }

    fn on_focus(&mut self, opts: FocusOpts) {
        self.selected_message = opts.selected_message;
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let overlay = centered_rect(40, 20, area);
        let paragraph = Paragraph::new("Delete this message?\n\n(Enter/y) Yes    (Esc/n) No")
            .block(
                Block::bordered()
                    .title("Confirm Delete")
                    .border_type(BorderType::Rounded),
            )
            .centered();
        frame.render_widget(paragraph, overlay);
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
