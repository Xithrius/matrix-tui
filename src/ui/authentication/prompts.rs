use color_eyre::Result;
use tokio::sync::mpsc::Sender;
use tui::{crossterm::event::KeyEvent, prelude::*};

use crate::{
    events::{Event, LoginMode},
    matrix::login::LoginChoice,
    ui::{
        authentication::{LoginChoicePromptWidget, PasswordPromptWidget, UsernamePromptWidget},
        component::Component,
    },
};

struct AuthenticationUI {
    login_choices_prompt: LoginChoicePromptWidget,
    username_prompt: UsernamePromptWidget,
    password_prompt: PasswordPromptWidget,
}

impl AuthenticationUI {
    fn new(event_tx: Sender<Event>) -> Self {
        let login_choices_prompt = LoginChoicePromptWidget::new(event_tx.clone());
        let username_prompt = UsernamePromptWidget::new(event_tx.clone());
        let password_prompt = PasswordPromptWidget::new(event_tx);

        Self {
            login_choices_prompt,
            username_prompt,
            password_prompt,
        }
    }
}

pub struct AuthenticationWidget {
    event_tx: Sender<Event>,
    ui: AuthenticationUI,

    login_mode: LoginMode,
}

impl AuthenticationWidget {
    pub fn new(event_tx: Sender<Event>) -> Self {
        let ui = AuthenticationUI::new(event_tx.clone());

        Self {
            event_tx,
            ui,
            login_mode: LoginMode::default(),
        }
    }

    pub fn set_login_mode(&mut self, mode: LoginMode) {
        self.login_mode = mode;
    }

    pub fn set_login_choices(&mut self, login_choices: Vec<LoginChoice>) {
        self.ui
            .login_choices_prompt
            .set_login_choices(login_choices);
    }
}

impl Component for AuthenticationWidget {
    async fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match self.login_mode {
            LoginMode::SelectLoginChoice => {
                self.ui.login_choices_prompt.handle_key_event(key).await?;
            }
            LoginMode::UsernamePrompt => {
                self.ui.username_prompt.handle_key_event(key).await?;
            }
            LoginMode::PasswordPrompt => {
                self.ui.password_prompt.handle_key_event(key).await?;
            }
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) {
        match self.login_mode {
            LoginMode::SelectLoginChoice => self.ui.login_choices_prompt.draw(frame, area),
            LoginMode::UsernamePrompt => self.ui.username_prompt.draw(frame, area),
            LoginMode::PasswordPrompt => self.ui.password_prompt.draw(frame, area),
        }
    }
}
