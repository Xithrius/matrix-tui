use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{crossterm::event::KeyEvent, prelude::*};

use crate::{
    events::{Event, LoginMode},
    matrix::login::{LoginChoice, LoginCredentials},
    ui::{
        authentication::{LoginChoicePromptWidget, PasswordPromptWidget, UsernamePromptWidget},
        component::Component,
    },
};

struct AuthenticationUI {
    login_choice: LoginChoicePromptWidget,
    username: UsernamePromptWidget,
    password: PasswordPromptWidget,
}

impl AuthenticationUI {
    fn new(event_tx: Sender<Event>) -> Self {
        let login_choice_prompt = LoginChoicePromptWidget::new(event_tx.clone());
        let username_prompt = UsernamePromptWidget::new(event_tx.clone());
        let password_prompt = PasswordPromptWidget::new(event_tx);

        Self {
            login_choice: login_choice_prompt,
            username: username_prompt,
            password: password_prompt,
        }
    }
}

pub struct AuthenticationWidget {
    ui: AuthenticationUI,

    login_mode: LoginMode,
}

impl AuthenticationWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let ui = AuthenticationUI::new(event_tx);

        Self {
            ui,
            login_mode: LoginMode::default(),
        }
    }

    /// Unfocus the previous mode's user input widget, if the last mode was of an input widget.
    /// Then, focus the new mode's user input widget, if said mode is a user input widget,
    /// and override the previous mode.
    ///
    /// TODO: Word better later, because it's 2am now.
    pub const fn set_login_mode(&mut self, mode: LoginMode) {
        match self.login_mode {
            LoginMode::UsernamePrompt => self.ui.username.set_focused(false),
            LoginMode::PasswordPrompt => self.ui.password.set_focused(false),
            LoginMode::SelectLoginChoice | LoginMode::Completed => {}
        }

        match mode {
            LoginMode::UsernamePrompt => self.ui.username.set_focused(true),
            LoginMode::PasswordPrompt => self.ui.password.set_focused(true),
            LoginMode::SelectLoginChoice | LoginMode::Completed => {}
        }

        self.login_mode = mode;
    }

    pub fn set_login_choices(&mut self, login_choices: Vec<LoginChoice>) {
        self.ui.login_choice.set_login_choices(login_choices);
    }

    pub fn selected_login_choice(&self) -> Option<LoginChoice> {
        self.ui.login_choice.selected_login_choice()
    }

    pub fn get_login_credentials(&self) -> Option<LoginCredentials> {
        let login_choice = self.selected_login_choice()?;

        let LoginChoice::Password = login_choice else {
            return None;
        };

        let password = self.ui.password.password()?;
        let username = self.ui.username.username()?;

        let credentials = LoginCredentials::Password { username, password };

        Some(credentials)
    }
}

impl Component for AuthenticationWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.login_mode {
            LoginMode::SelectLoginChoice => {
                self.ui.login_choice.handle_key_event(key).await?;
            }
            LoginMode::UsernamePrompt => {
                self.ui.username.handle_key_event(key).await?;
            }
            LoginMode::PasswordPrompt => {
                self.ui.password.handle_key_event(key).await?;
            }
            // TODO: Handle login cancelling?
            LoginMode::Completed => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
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
