use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
#[allow(dead_code)]
pub enum LoginMode {
    #[default]
    SelectLoginChoice,
    UsernamePrompt,
    PasswordPrompt,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Login(LoginMode),
    Messages,
    Input,
    RoomNavigation,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Login(LoginMode::default())
    }
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Login(login_mode) => match login_mode {
                LoginMode::SelectLoginChoice => write!(f, "Select login choice"),
                LoginMode::UsernamePrompt => write!(f, "Username prompt"),
                LoginMode::PasswordPrompt => write!(f, "Password prompt"),
                LoginMode::Completed => write!(f, "Completed"),
            },
            Self::Messages => write!(f, "Messages"),
            Self::Input => write!(f, "Input"),
            Self::RoomNavigation => write!(f, "Room navigation"),
        }
    }
}
