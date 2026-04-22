use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout, Rect},
};

use crate::{
    matrix::login::{LoginChoice, LoginCredentials},
    ui::{
        action::{Action, ContextKey, FocusOpts, KeyResult},
        context::{Context, Keybinding, ViewName},
        widgets::user_input::UserInputWidget,
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum LoginMode {
    #[default]
    Username,
    Password,
}

pub struct LoginContext {
    username: UserInputWidget,
    password: UserInputWidget,
    login_choices: Vec<LoginChoice>,
    login_mode: LoginMode,
}

impl LoginContext {
    pub fn new() -> Self {
        Self {
            username: UserInputWidget::new(Some("Username")),
            password: UserInputWidget::new(Some("Password")),
            login_choices: Vec::new(),
            login_mode: LoginMode::default(),
        }
    }

    const fn set_login_mode(&mut self, mode: LoginMode) {
        self.username
            .set_focused(matches!(mode, LoginMode::Username));
        self.password
            .set_focused(matches!(mode, LoginMode::Password));
        self.login_mode = mode;
    }

    // --- Domain API for matrix notification handlers ---

    pub fn set_login_choices(&mut self, choices: Vec<LoginChoice>) {
        self.login_choices = choices;
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub const fn selected_login_choice(&self) -> Option<LoginChoice> {
        Some(LoginChoice::Password)
    }

    /// Take the completed login credentials out, clearing stored state.
    pub fn take_credentials(&mut self) -> Option<LoginCredentials> {
        let username = self.username.get_input().to_owned();
        let password = self.password.get_input().to_owned();

        if username.is_empty() || password.is_empty() {
            return None;
        }

        self.username.clear();
        self.password.clear();
        self.set_login_mode(LoginMode::Username);

        Some(LoginCredentials::Password { username, password })
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
        vec![
            Keybinding::new(KeyCode::Esc, KeyModifiers::NONE, Action::Quit, "Quit"),
            Keybinding::new(
                KeyCode::Enter,
                KeyModifiers::NONE,
                Action::SubmitLogin,
                "Submit login",
            ),
        ]
    }

    fn handle_unbound_key(&mut self, key: KeyEvent) -> KeyResult {
        match self.login_mode {
            LoginMode::Username => match key.code {
                KeyCode::Esc => KeyResult::DoAction(Action::Quit),
                KeyCode::Tab | KeyCode::BackTab => {
                    self.set_login_mode(LoginMode::Password);
                    KeyResult::Consumed
                }
                _ => self.username.handle_key(key),
            },
            LoginMode::Password => match key.code {
                KeyCode::Tab | KeyCode::BackTab => {
                    self.set_login_mode(LoginMode::Username);
                    KeyResult::Consumed
                }
                _ => self.password.handle_key(key),
            },
        }
    }

    fn on_focus(&mut self, _opts: FocusOpts) {
        self.set_login_mode(LoginMode::Username);
    }

    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let [_, username_area, password_area, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Fill(1),
        ])
        .areas(area);

        self.username.draw(frame, username_area);
        self.password.draw(frame, password_area);
    }
}
