use tui::{Frame, layout::Rect};

use crate::{
    matrix::login::{LoginChoice, LoginCredentials},
    ui::{
        action::{ContextKey, FocusOpts, KeyResult},
        authentication::AuthenticationWidget,
        context::{Context, Keybinding, ViewName},
    },
};

pub struct LoginContext {
    widget: AuthenticationWidget,
}

impl LoginContext {
    pub fn new() -> Self {
        Self {
            widget: AuthenticationWidget::new(),
        }
    }

    // --- Domain API for matrix notification handlers ---

    pub fn set_login_choices(&mut self, choices: Vec<LoginChoice>) {
        self.widget.set_login_choices(choices);
    }

    pub fn selected_login_choice(&self) -> Option<LoginChoice> {
        self.widget.selected_login_choice()
    }

    /// Take the completed login credentials out, clearing stored state.
    pub fn take_credentials(&mut self) -> Option<LoginCredentials> {
        self.widget.take_credentials()
    }
}

impl Context for LoginContext {
    fn key(&self) -> ContextKey {
        ContextKey::Login
    }

    fn view(&self) -> ViewName {
        ViewName::Fullscreen
    }

    fn is_fullscreen(&self) -> bool {
        true
    }

    fn keybindings(&self) -> Vec<Keybinding> {
        // All keys (including Enter at each step) are handled via handle_unbound_key
        // to allow sub-mode-aware logic. Global bindings cover Quit.
        vec![]
    }

    fn handle_unbound_key(&mut self, key: tui::crossterm::event::KeyEvent) -> KeyResult {
        self.widget.handle_key(key)
    }

    fn on_focus(&mut self, _opts: FocusOpts) {
        self.widget.on_focus();
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.widget.draw(frame, area);
    }
}
