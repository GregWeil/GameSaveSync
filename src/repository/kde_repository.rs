use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Error, Result};
use regex::Regex;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize, Debug)]
pub struct KdeRepositoryConfig {
    pub protocol: Option<String>,
    pub path: PathBuf,
}

fn kio_url(protocol: &Option<String>, path: &Path) -> String {
    let empty = String::new();
    let protocol = protocol.as_ref().unwrap_or(&empty);
    let path = path.to_str().unwrap_or(&empty);
    protocol.to_owned() + path
}

impl std::fmt::Display for KdeRepositoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", kio_url(&self.protocol, &self.path))
    }
}

#[derive(Debug)]
pub struct KdeRepository {
    pub protocol: Option<String>,
    pub path: PathBuf,
}

const KIOCLIENT: &str = "kioclient";

lazy_static::lazy_static! {
    static ref RE_DOES_NOT_EXIST: Regex = Regex::new(r"(?:^|\n)kioclient:\s+The file or folder .* does not exist.(?:\n|$)").unwrap();
    static ref RE_FILE_TYPE: Regex = Regex::new(r"(?:^|\n)FILE_TYPE\s+(\d+)(?:\n|$)").unwrap();
}

fn get_file_type(kio_url: &str) -> Result<Option<String>> {
    let result = Command::new(KIOCLIENT)
        .arg("stat")
        .arg(kio_url)
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

fn copy_file(
    source_protocol: &Option<String>,
    source_path: &Path,
    destination_protocol: &Option<String>,
    destination_path: &Path,
) -> Result<()> {
    let source = kio_url(source_protocol, source_path);
    let destination = kio_url(destination_protocol, destination_path);
    let result = Command::new(KIOCLIENT)
        .arg("copy")
        .arg("--overwrite")
        .arg(&source)
        .arg(&destination)
        .output()
        .with_context(|| format!("failed to exec {KIOCLIENT} copy"))?;
    if !result.status.success() {
        let error = String::from_utf8(result.stderr)
            .with_context(|| format!("failed to read {KIOCLIENT} copy stderr"))?;
        return Result::Err(Error::msg(error))
            .with_context(|| format!("{KIOCLIENT} remove returned {}", result.status));
    }
    Ok(())
}

impl super::Repository for KdeRepository {
    fn is_file(&self, path: &RelativePath) -> Result<bool> {
        let path = kio_url(&self.protocol, &path.to_path(&self.path));
        match get_file_type(&path)? {
            Some(ref file_type) if file_type == "0100000" => Ok(true),
            _ => Ok(false),
        }
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        let path = kio_url(&self.protocol, &path.to_path(&self.path));
        match get_file_type(&path)? {
            Some(ref file_type) if file_type == "0040000" => Ok(true),
            _ => Ok(false),
        }
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>> {
        let path = kio_url(&self.protocol, &path.to_path(&self.path));
        let result = Command::new(KIOCLIENT)
            .arg("ls")
            .arg(path)
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

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        let path = kio_url(&self.protocol, &path.to_path(&self.path));
        let result = Command::new(KIOCLIENT)
            .arg("cat")
            .arg(path)
            .output()
            .with_context(|| format!("failed to exec {KIOCLIENT} cat"))?;
        if !result.status.success() {
            let error = String::from_utf8(result.stderr)
                .with_context(|| format!("failed to read {KIOCLIENT} cat stderr"))?;
            return Result::Err(Error::msg(error))
                .with_context(|| format!("{KIOCLIENT} cat returned {}", result.status));
        }
        let content = String::from_utf8(result.stdout)
            .with_context(|| format!("failed to read {KIOCLIENT} cat stdout"))?;
        Ok(content)
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        let file = match path.file_name() {
            Some(name) => NamedTempFile::with_suffix(&name)
                .with_context(|| format!("failed to create a temp file to write {name}"))?,
            None => {
                NamedTempFile::new().with_context(|| "failed to create a temp file to write")?
            }
        };
        std::fs::write(&file, &content).with_context(|| "failed to write to temp file")?;
        self.upload_file(file.path(), &path)?;
        Ok(())
    }

    fn download_file(&self, repository_path: &RelativePath, local_path: &Path) -> Result<()> {
        copy_file(
            &self.protocol,
            &repository_path.to_path(&self.path),
            &None,
            &local_path,
        )
        .with_context(|| format!("failed to copy to {}", local_path.display()))?;
        Ok(())
    }

    fn upload_file(&self, local_path: &Path, repository_path: &RelativePath) -> Result<()> {
        copy_file(
            &None,
            &local_path,
            &self.protocol,
            &repository_path.to_path(&self.path),
        )
        .with_context(|| format!("failed to copy from {}", local_path.display()))?;
        Ok(())
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        let path = kio_url(&self.protocol, &path.to_path(&self.path));
        let result = Command::new(KIOCLIENT)
            .arg("remove")
            .arg(&path)
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
}

pub fn open_repository(config: &KdeRepositoryConfig) -> Result<KdeRepository> {
    match &config.protocol {
        Some(protocol) => {
            if protocol.is_empty() {
                return Result::Err(Error::msg(format!(
                    "invalid configuration: protocol must not be empty if specified"
                )));
            }
            if config.path.is_absolute() {
                return Result::Err(Error::msg(format!(
                    "invalid configuration: path must not be absolute if a protocol is specified"
                )));
            }
        }
        None => {
            if !config.path.is_absolute() {
                return Result::Err(Error::msg(format!(
                    "invalid configuration: path must be absolute if no protocol is specified"
                )));
            }
        }
    }
    let version = Command::new(KIOCLIENT)
        .arg("--version")
        .output()
        .with_context(|| format!("failed to exec {KIOCLIENT} --version"))?;
    if !version.status.success() {
        return Result::Err(Error::msg(String::from_utf8(version.stderr)?))
            .with_context(|| format!("{KIOCLIENT} returned {}", version.status));
    }
    Ok(KdeRepository {
        protocol: config.protocol.clone(),
        path: config.path.clone(),
    })
}
