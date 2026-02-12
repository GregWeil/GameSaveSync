use std::path::PathBuf;

use anyhow::{Error, Result};
use directories::ProjectDirs;

fn project_dirs() -> Result<ProjectDirs> {
    match ProjectDirs::from("org", "GameSaveSync", "GameSaveSync") {
        Some(dirs) => Ok(dirs),
        None => Result::Err(Error::msg("did not get project directories")),
    }
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.config_local_dir();
    Ok(path.to_path_buf())
}

pub fn data_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.data_local_dir();
    Ok(path.to_path_buf())
}
