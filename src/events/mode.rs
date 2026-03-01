#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum LoginMode {
    #[default]
    SelectLoginChoice,
    UsernamePrompt,
    PasswordPrompt,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Login(LoginMode),
    Messages,
    Input,
}

impl Default for Mode {
    fn default() -> Self {
        // Self::Login(LoginMode::default())
        Self::Messages
    }
}

// TODO: Replace with fmt::Display impl
impl ToString for Mode {
    fn to_string(&self) -> String {
        let msg = match self {
            Mode::Login(login_mode) => match login_mode {
                LoginMode::SelectLoginChoice => "Select login choice",
                LoginMode::UsernamePrompt => "Username prompt",
                LoginMode::PasswordPrompt => "Password prompt",
            },
            Mode::Messages => "Messages",
            Mode::Input => "Input",
        };

        msg.to_string()
    }
}
