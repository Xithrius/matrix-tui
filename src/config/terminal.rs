use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct TerminalConfig {
    pub frame_rate: f64,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self { frame_rate: 60.0 }
    }
}
