use crate::{
    config::CoreConfig,
    ui::{
        context_manager::ContextManager,
        context_registry::ContextRegistry,
        widgets::{
            header::HeaderWidget,
            status_line::{Status, StatusLineWidget},
        },
    },
};

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
