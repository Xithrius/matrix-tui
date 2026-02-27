use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct MatrixConfig {
    homeserver_url: Option<String>,
}
