use std::path::PathBuf;

use anyhow::{Error, Result};
use directories::ProjectDirs;

fn project_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("org", "GameSaveSync", "GameSaveSync")
        .ok_or_else(|| Error::msg("did not get project directories"))
}

pub fn config_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.config_local_dir();
    Ok(path.to_path_buf())
}

pub fn state_dir() -> Result<PathBuf> {
    let dirs = project_dirs()?;
    let path = dirs.state_dir().unwrap_or_else(|| dirs.data_local_dir());
    Ok(path.to_path_buf())
}

pub fn make_path_safe(value: &str) -> String {
    value
        .replace("?", "？")
        .replace(":", "꞉")
        .replace("*", "✳")
        .replace("|", "⏐")
        .replace("<", "＜")
        .replace(">", "＞")
        .replace("/", "⧸")
        .replace("\\", "⧹")
        .replace("\"", "＂")
}
