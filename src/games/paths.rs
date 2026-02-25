use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Error, Ok, Result};

#[derive(PartialEq, Debug)]
pub enum LinuxPathReplacement {
    Home,
}

const LINUX_PATH_REPLACEMENTS: [LinuxPathReplacement; 1] = [LinuxPathReplacement::Home];

impl FromStr for LinuxPathReplacement {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<home>" | "$HOME" | "~" => Result::Ok(LinuxPathReplacement::Home),
            _ => Err(()),
        }
    }
}

impl LinuxPathReplacement {
    fn to_path(&self) -> Result<OsString> {
        match self {
            LinuxPathReplacement::Home => Ok(std::env::home_dir()
                .ok_or(Error::msg("failed to get home directory"))?
                .into()),
        }
    }
}

pub fn rewrite_path(path: &str) -> Result<PathBuf> {
    let mut rewritten = OsString::with_capacity(path.len());
    let mut first = true;
    for segment in path.split(std::path::is_separator) {
        if !first {
            rewritten.push(std::path::MAIN_SEPARATOR_STR);
        }
        first = false;
        let replacement = match LinuxPathReplacement::from_str(segment) {
            Result::Ok(replacement_type) => replacement_type.to_path()?,
            Result::Err(_) => OsString::from(segment),
        };
        rewritten.push(replacement);
    }
    let rewritten = PathBuf::from(rewritten);
    if !rewritten.is_absolute() {
        return Err(Error::msg(format!(
            "path {} is not absolute",
            rewritten.display()
        )));
    }
    for replacement_type in LINUX_PATH_REPLACEMENTS {
        let replacement = match replacement_type.to_path() {
            Result::Ok(replacement) => PathBuf::from(replacement),
            Result::Err(_) => continue,
        };
        if replacement.starts_with(&rewritten) {
            return Err(Error::msg(format!("path {} is too broad", path)));
        }
    }
    Ok(rewritten)
}
