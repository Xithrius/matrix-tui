use std::{env, path::PathBuf, sync::LazyLock};

use color_eyre::Result;
use config::{Config, File};
use directories::ProjectDirs;
use tracing::error;

use crate::config::core::CoreConfig;

static BINARY_NAME: LazyLock<String> = LazyLock::new(|| env!("CARGO_PKG_NAME").to_lowercase());
static DATA_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_DATA", BINARY_NAME.clone()))
        .ok()
        .map(PathBuf::from)
});
static CONFIG_DIR: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    env::var(format!("{}_CONFIG", BINARY_NAME.clone()))
        .ok()
        .map(PathBuf::from)
});

const CONFIG_FILE_NAME: &str = "config.toml";

fn project_directory() -> Option<ProjectDirs> {
    ProjectDirs::from("com", &BINARY_NAME, env!("CARGO_PKG_NAME"))
}

#[allow(unused)]
pub fn get_data_dir() -> PathBuf {
    DATA_DIR.clone().unwrap_or_else(|| {
        project_directory().map_or_else(
            || PathBuf::from(".").join(".data"),
            |proj_dirs| proj_dirs.data_local_dir().to_path_buf(),
        )
    })
}

pub fn get_config_dir() -> PathBuf {
    CONFIG_DIR.clone().unwrap_or_else(|| {
        project_directory().map_or_else(
            || PathBuf::from(".").join(".config"),
            |proj_dirs| proj_dirs.config_local_dir().to_path_buf(),
        )
    })
}

pub(super) fn load_config() -> Result<CoreConfig> {
    let path = get_config_dir().join(CONFIG_FILE_NAME);

    let config = match Config::builder()
        .add_source(File::with_name(&path.to_string_lossy()))
        .build()
    {
        Ok(config) => config.try_deserialize::<CoreConfig>()?,
        Err(err) => {
            error!("Failed to build config: {}, going off of defaults.", err);
            CoreConfig::default()
        }
    };

    Ok(config)
}
