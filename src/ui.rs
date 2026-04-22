mod action;
mod authentication;
pub(crate) mod context;
pub(crate) mod context_manager;
mod context_registry;
pub mod contexts;
mod header;
mod messages;
mod navigation;
mod recovery;
mod spinner;
mod status_line;
mod user_input;

pub use action::{Action, ContextKey, FocusOpts, KeyResult};
pub use context_manager::{ContextManager, StackEntry};
pub use context_registry::ContextRegistry;
pub use header::HeaderWidget;
pub use status_line::{Status, StatusLineWidget};

use crate::config::CoreConfig;

pub struct Ui {
    pub registry: ContextRegistry,
    pub ctx_mgr: ContextManager,
    /// Always-visible title bar (not part of the focus system).
    pub header: HeaderWidget,
    /// Always-visible status bar (not part of the focus system).
    pub status_line: StatusLineWidget,
}

impl Ui {
    pub fn new(config: &CoreConfig) -> Self {
        Self {
            registry: ContextRegistry::new(),
            ctx_mgr: ContextManager::new(),
            header: HeaderWidget::new(config, "matrix-tui".to_string()),
            status_line: StatusLineWidget::new(
                Some(Status::Info("Launching...".to_string())),
                None,
            ),
        }
    }
}
