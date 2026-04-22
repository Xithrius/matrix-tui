/// Sub-modes of the login flow, managed internally by `AuthenticationWidget`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum LoginMode {
    #[default]
    SelectLoginChoice,
    UsernamePrompt,
    PasswordPrompt,
    Completed,
}

/// Sub-modes of the recovery flow, managed internally by `RecoveryWidget`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RecoveryMode {
    /// Prompt the user to enter their existing recovery key.
    EnterKey,
    /// Display a newly generated recovery key the user must record and confirm.
    ShowKey,
}
