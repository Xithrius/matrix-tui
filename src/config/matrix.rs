use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct MatrixConfig {
    pub homeserver_url: String,
    pub username: String,
}
