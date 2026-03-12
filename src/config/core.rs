use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::config::{matrix::MatrixConfig, persistence::load_config, terminal::TerminalConfig};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct CoreConfig {
    pub terminal: TerminalConfig,
    pub matrix: MatrixConfig,
}

impl CoreConfig {
    pub fn new(frame_rate: u64) -> Result<Self> {
        let mut config = load_config()?;

        config.terminal.frame_rate = frame_rate;

        Ok(config)
    }
}
