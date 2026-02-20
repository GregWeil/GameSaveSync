use std::ffi::OsString;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Error, Ok, Result};

pub struct ParsePathReplacementErr;

#[derive(PartialEq, Debug)]
pub enum LinuxPathReplacement {
    Home,
}

impl FromStr for LinuxPathReplacement {
    type Err = ParsePathReplacementErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "<home>" | "$HOME" | "~" => Result::Ok(LinuxPathReplacement::Home),
            _ => Err(ParsePathReplacementErr),
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
            Result::Ok(replacement_type) => match replacement_type {
                LinuxPathReplacement::Home => std::env::home_dir()
                    .ok_or(Error::msg("failed to get home directory"))?
                    .into(),
            },
            Err(_) => OsString::from(segment),
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
    Ok(rewritten)
}
