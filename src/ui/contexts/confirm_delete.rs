use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::Rect,
    widgets::{Block, BorderType, Paragraph},
};

use crate::ui::{
    action::{Action, FocusOpts},
    context::{Context, Keybinding},
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
        let paragraph = Paragraph::new("Delete this message?\n\n(Enter/y) Yes    (Esc/n) No")
            .block(
                Block::bordered()
                    .title("Confirm Delete")
                    .border_type(BorderType::Rounded),
            )
            .centered();
        frame.render_widget(paragraph, area);
    }
}
