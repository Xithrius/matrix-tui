use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
#[allow(dead_code)]
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
        Self::Login(LoginMode::default())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::Login(login_mode) => match login_mode {
                LoginMode::SelectLoginChoice => write!(f, "Select login choice"),
                LoginMode::UsernamePrompt => write!(f, "Username prompt"),
                LoginMode::PasswordPrompt => write!(f, "Password prompt"),
            },
            Mode::Messages => write!(f, "Messages"),
            Mode::Input => write!(f, "Input"),
        }
    }
}
