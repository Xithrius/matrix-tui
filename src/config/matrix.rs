use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct MatrixConfig {
    server: Option<String>,
}
