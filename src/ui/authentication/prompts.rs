use tui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    events::LoginMode,
    matrix::login::{LoginChoice, LoginCredentials},
    ui::{
        action::{Action, KeyResult},
        authentication::{LoginChoicePromptWidget, PasswordPromptWidget, UsernamePromptWidget},
    },
};

struct AuthenticationUI {
    login_choice: LoginChoicePromptWidget,
    username: UsernamePromptWidget,
    password: PasswordPromptWidget,
}

impl AuthenticationUI {
    fn new() -> Self {
        Self {
            login_choice: LoginChoicePromptWidget::new(),
            username: UsernamePromptWidget::new(),
            password: PasswordPromptWidget::new(),
        }
    }
}

pub struct AuthenticationWidget {
    ui: AuthenticationUI,
    login_mode: LoginMode,
}

impl AuthenticationWidget {
    pub fn new() -> Self {
        Self {
            ui: AuthenticationUI::new(),
            login_mode: LoginMode::default(),
        }
    }

    /// Transition to a new sub-mode, updating focused state on sub-widgets.
    const fn set_login_mode(&mut self, mode: LoginMode) {
        // Blur the outgoing widget
        match self.login_mode {
            LoginMode::UsernamePrompt => self.ui.username.set_focused(false),
            LoginMode::PasswordPrompt => self.ui.password.set_focused(false),
            LoginMode::SelectLoginChoice | LoginMode::Completed => {}
        }
        // Focus the incoming widget
        match mode {
            LoginMode::UsernamePrompt => self.ui.username.set_focused(true),
            LoginMode::PasswordPrompt => self.ui.password.set_focused(true),
            LoginMode::SelectLoginChoice | LoginMode::Completed => {}
        }

        self.login_mode = mode;
    }

    /// Called when the context gains focus (e.g. app start or after logout).
    pub const fn on_focus(&mut self) {
        self.set_login_mode(LoginMode::SelectLoginChoice);
    }

    pub fn set_login_choices(&mut self, login_choices: Vec<LoginChoice>) {
        self.ui.login_choice.set_login_choices(login_choices);
    }

    pub fn selected_login_choice(&self) -> Option<LoginChoice> {
        self.ui.login_choice.selected_login_choice()
    }

    /// Take the completed credentials out (clears stored state).
    pub fn take_credentials(&mut self) -> Option<LoginCredentials> {
        let login_choice = self.selected_login_choice()?;

        let credentials = match login_choice {
            LoginChoice::Password => {
                let username = self.ui.username.username()?;
                let password = self.ui.password.password()?;
                Some(LoginCredentials::Password { username, password })
            }
            LoginChoice::Sso | LoginChoice::SsoIdp(_) => None,
        };

        // Reset for the next login attempt
        self.ui.username.clear();
        self.ui.password.clear();
        self.set_login_mode(LoginMode::SelectLoginChoice);

        credentials
    }

    /// Synchronous key handler. Manages login sub-mode transitions internally
    /// and returns `DoAction(SubmitLogin)` when credentials are ready.
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self.login_mode {
            LoginMode::SelectLoginChoice => match key.code {
                KeyCode::Esc => KeyResult::DoAction(Action::Quit),
                KeyCode::Enter => {
                    self.ui.login_choice.confirm_selection();
                    self.set_login_mode(LoginMode::UsernamePrompt);
                    KeyResult::Consumed
                }
                _ => self.ui.login_choice.handle_nav_key(key),
            },
            LoginMode::UsernamePrompt => match key.code {
                KeyCode::Esc => {
                    self.set_login_mode(LoginMode::SelectLoginChoice);
                    KeyResult::Consumed
                }
                KeyCode::Enter => {
                    if self.ui.username.has_input() {
                        self.ui.username.confirm();
                        self.set_login_mode(LoginMode::PasswordPrompt);
                    }
                    KeyResult::Consumed
                }
                _ => self.ui.username.handle_text_key(key),
            },
            LoginMode::PasswordPrompt => match key.code {
                KeyCode::Esc => {
                    self.ui.username.clear();
                    self.set_login_mode(LoginMode::SelectLoginChoice);
                    KeyResult::Consumed
                }
                KeyCode::Enter => {
                    if self.ui.password.has_input() {
                        self.ui.password.confirm();
                        self.set_login_mode(LoginMode::Completed);
                        KeyResult::DoAction(Action::SubmitLogin)
                    } else {
                        KeyResult::Consumed
                    }
                }
                _ => self.ui.password.handle_text_key(key),
            },
            LoginMode::Completed => KeyResult::Consumed,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.login_mode {
            LoginMode::SelectLoginChoice => {
                self.ui.login_choice.draw(frame, area);
            }
            LoginMode::UsernamePrompt => {
                self.ui.username.draw(frame, area);
            }
            LoginMode::PasswordPrompt | LoginMode::Completed => {
                self.ui.password.draw(frame, area);
            }
        }
    }
}
