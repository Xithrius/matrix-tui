use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    widgets::{Block, BorderType, List, ListItem},
};

use crate::ui::{
    action::{Action, ContextKey, FocusOpts},
    context::{Context, Keybinding, ViewName},
};

pub struct MessageActionsContext {
    pub selected_message: Option<usize>,
}

impl MessageActionsContext {
    pub const fn new() -> Self {
        Self {
            selected_message: None,
        }
    }
}

impl Context for MessageActionsContext {
    fn key(&self) -> ContextKey {
        ContextKey::MessageActions
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
                KeyCode::Char('r'),
                KeyModifiers::NONE,
                Action::ReplyToMessage(idx),
                "Reply to message",
            ),
            Keybinding::new(
                KeyCode::Char('e'),
                KeyModifiers::NONE,
                Action::EditMessage(idx),
                "Edit message",
            ),
            Keybinding::new(
                KeyCode::Char('d'),
                KeyModifiers::NONE,
                Action::DeleteMessage(idx),
                "Delete message",
            ),
            Keybinding::new(
                KeyCode::Esc,
                KeyModifiers::NONE,
                Action::PopContext,
                "Close",
            ),
        ]
    }

    fn on_focus(&mut self, opts: FocusOpts) {
        self.selected_message = opts.selected_message;
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        let overlay = centered_rect(40, 10, area);
        let items = vec![
            ListItem::new("(r) Reply"),
            ListItem::new("(e) Edit"),
            ListItem::new("(d) Delete"),
            ListItem::new("(Esc) Close"),
        ];
        let list = List::new(items).block(
            Block::bordered()
                .title("Message Actions")
                .border_type(BorderType::Rounded),
        );
        frame.render_widget(list, overlay);
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
