use std::{fs, io::ErrorKind::NotFound};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::paths::config_dir;
use crate::repository::AnyRepositoryConfig;

const CONFIG_FILE: &str = "config.toml";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    pub repository: Option<AnyRepositoryConfig>,
}

pub fn load() -> Result<Config> {
    let path = config_dir()?.join(CONFIG_FILE);
    match fs::read_to_string(path) {
        Ok(ref file) => toml::from_str(file).with_context(|| "failed to parse config"),
        Err(ref e) if e.kind() == NotFound => Ok(Config::default()),
        Err(error) => Result::Err(error).with_context(|| "failed to read config"),
    }
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_dir()?.join(CONFIG_FILE);
    match path.parent() {
        Some(dir) => {
            fs::create_dir_all(dir).with_context(|| "failed to create config directory")?
        }
        None => unreachable!("config file should always have a parent path"),
    }
    let serialized =
        toml::to_string_pretty(config).with_context(|| "failed to serialize config")?;
    fs::write(path, serialized).with_context(|| "failed to write config")?;
    Ok(())
}
