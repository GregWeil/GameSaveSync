use std::{fs, io::ErrorKind::NotFound, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::paths::config_dir;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct RepositoryConfig {
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub repository: Option<RepositoryConfig>,
}

fn config_path() -> Result<PathBuf> {
    let dir = config_dir()?;
    Ok(dir.join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    match fs::read_to_string(path) {
        Ok(ref file) => toml::from_str(file).with_context(|| "failed to parse config"),
        Err(ref e) if e.kind() == NotFound => Ok(Config::default()),
        Err(error) => Result::Err(error).with_context(|| "failed to read config"),
    }
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    match path.parent() {
        Some(dir) => {
            fs::create_dir_all(dir).with_context(|| "failed to create config directory")?
        }
        None => {}
    }
    let serialized =
        toml::to_string_pretty(config).with_context(|| "failed to serialize config")?;
    fs::write(path, serialized).with_context(|| "failed to write config")?;
    Ok(())
}
