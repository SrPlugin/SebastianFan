use std::fs;
use std::path::Path;

use sebastianfan_core::Config;

use crate::error::CliError;
use crate::store;

pub fn path(config_path: &Path) -> String {
    config_path.display().to_string()
}

pub fn show(config_path: &Path) -> Result<String, CliError> {
    let config = store::load(config_path)?;
    Ok(config.to_toml()?.trim_end().to_owned())
}

pub fn init(config_path: &Path, force: bool) -> Result<String, CliError> {
    if config_path.exists() && !force {
        return Err(CliError::AlreadyExists(config_path.to_path_buf()));
    }
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|source| CliError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(config_path, Config::default().to_toml()?).map_err(|source| CliError::Io {
        path: config_path.to_path_buf(),
        source,
    })?;
    Ok(format!("wrote {}", config_path.display()))
}
