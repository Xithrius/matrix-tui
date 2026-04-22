/// Sub-modes of the recovery flow, managed internally by `RecoveryWidget`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RecoveryMode {
    /// Prompt the user to enter their existing recovery key.
    EnterKey,
    /// Display a newly generated recovery key the user must record and confirm.
    ShowKey,
}
