use std::{fmt::format, process::Command};

use anyhow::{Context, Error, Result};
use regex::Regex;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct KdeRepositoryConfig {
    pub kio_url: RelativePathBuf,
}

impl std::fmt::Display for KdeRepositoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.kio_url)
    }
}

#[derive(Debug)]
pub struct KdeRepository {
    kio_url: RelativePathBuf,
}

const KIOCLIENT: &str = "kioclient";

lazy_static::lazy_static! {
    static ref RE_DOES_NOT_EXIST: Regex = Regex::new(r"^kioclient:\s+The file or folder .* does not exist." ).unwrap();
    static ref RE_FILE_TYPE: Regex = Regex::new(r"(?:^|\n)FILE_TYPE\s+(\d+)(?:\n|$)").unwrap();
}

fn get_file_type(kio_url: &RelativePath) -> Result<Option<String>> {
    let result = Command::new(KIOCLIENT)
        .arg("stat")
        .arg(kio_url.as_str())
        .output()
        .with_context(|| format!("failed to exec {KIOCLIENT} stat {kio_url}"))?;
    if !result.status.success() {
        let error = String::from_utf8(result.stderr)
            .with_context(|| format!("failed to read {KIOCLIENT} stat stderr"))?;
        if RE_DOES_NOT_EXIST.is_match(&error) {
            return Ok(None);
        }
        return Result::Err(Error::msg(error))
            .with_context(|| format!("{KIOCLIENT} stat returned {}", result.status));
    }
    let output = String::from_utf8(result.stdout)
        .with_context(|| format!("failed to read {KIOCLIENT} stat stdout"))?;
    let captures = match RE_FILE_TYPE.captures(&output) {
        Some(captures) => captures,
        None => {
            return Err(Error::msg(format!(
                "Did not find FILE_TYPE in {KIOCLIENT} stat stdout"
            )));
        }
    };
    Ok(Some(captures[1].into()))
}

impl super::Repository for KdeRepository {
    fn is_file(&self, path: &RelativePath) -> Result<bool> {
        match get_file_type(&self.kio_url.join(path))? {
            Some(ref file_type) if file_type == "0100000" => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        match get_file_type(&self.kio_url.join(path))? {
            Some(ref file_type) if file_type == "0040000" => Ok(true),
            _ => Ok(false),
        }
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>> {
        let result = Command::new(KIOCLIENT)
            .arg("ls")
            .arg(self.kio_url.join(path).as_str())
            .output()
            .with_context(|| format!("failed to exec {KIOCLIENT} ls"))?;
        if !result.status.success() {
            let error = String::from_utf8(result.stderr)
                .with_context(|| format!("failed to read {KIOCLIENT} ls stderr"))?;
            return Result::Err(Error::msg(error))
                .with_context(|| format!("{KIOCLIENT} ls returned {}", result.status));
        }
        let output = String::from_utf8(result.stdout)
            .with_context(|| format!("failed to read {KIOCLIENT} ls stdout"))?;
        let iterator = output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != ".")
            .map(|line| anyhow::Ok(RelativePathBuf::from(line)));
        Ok(Box::new(iterator.collect::<Vec<_>>().into_iter()))
    }

    fn read_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Read>> {
        todo!()
    }

    fn write_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Write>> {
        todo!()
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        let result = Command::new(KIOCLIENT)
            .arg("remove")
            .arg(self.kio_url.join(path).as_str())
            .output()
            .with_context(|| format!("failed to exec {KIOCLIENT} remove"))?;
        if !result.status.success() {
            let error = String::from_utf8(result.stderr)
                .with_context(|| format!("failed to read {KIOCLIENT} remove stderr"))?;
            if RE_DOES_NOT_EXIST.is_match(&error) {
                return Ok(());
            }
            return Result::Err(Error::msg(error))
                .with_context(|| format!("{KIOCLIENT} remove returned {}", result.status));
        }
        Ok(())
    }

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        let result = Command::new(KIOCLIENT)
            .arg("cat")
            .arg(self.kio_url.join(path).as_str())
            .output()
            .with_context(|| format!("failed to exec {KIOCLIENT} cat"))?;
        if !result.status.success() {
            let error = String::from_utf8(result.stderr)
                .with_context(|| format!("failed to read {KIOCLIENT} cat stderr"))?;
            return Result::Err(Error::msg(error))
                .with_context(|| format!("{KIOCLIENT} cat returned {}", result.status));
        }
        let output = String::from_utf8(result.stdout)
            .with_context(|| format!("failed to read {KIOCLIENT} cat stdout"))?;
        Ok(output)
    }
}

pub fn open_repository(config: &KdeRepositoryConfig) -> Result<KdeRepository> {
    let version = Command::new(KIOCLIENT)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to exec {KIOCLIENT} --version"))?;
    if !version.status.success() {
        return Result::Err(Error::msg(String::from_utf8(version.stderr)?))
            .with_context(|| format!("{KIOCLIENT} returned {}", version.status));
    }
    Ok(KdeRepository {
        kio_url: config.kio_url.clone(),
    })
}
