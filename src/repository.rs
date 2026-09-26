use std::{io::Write, ops::Deref};

use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

pub mod kde_repository;
pub mod local_repository;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnyRepositoryConfig {
    Local(local_repository::LocalRepositoryConfig),
    KDE(kde_repository::KdeRepositoryConfig),
}

impl std::fmt::Display for AnyRepositoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnyRepositoryConfig::Local(config) => write!(f, "{}", config),
            AnyRepositoryConfig::KDE(config) => write!(f, "KDE KIO - {}", config),
        }
    }
}

pub trait Repository {
    fn is_file(&self, path: &RelativePath) -> Result<bool>;
    fn is_dir(&self, path: &RelativePath) -> Result<bool>;
    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>>;
    fn read_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Read>>;
    fn write_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Write>>;
    fn remove(&self, path: &RelativePath) -> Result<()>;

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        let file = self.read_file(path)?;
        std::io::read_to_string(file).with_context(|| format!("failed to read {path}"))
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        let mut file = self.write_file(path)?;
        file.write_all(content.as_ref())
            .with_context(|| format!("failed to write {path}"))
    }
}

impl Repository for Box<dyn Repository> {
    fn is_file(&self, path: &RelativePath) -> Result<bool> {
        self.deref().is_file(path)
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        self.deref().is_dir(path)
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>> {
        self.deref().read_dir(path)
    }

    fn read_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Read>> {
        self.deref().read_file(path)
    }

    fn write_file(&self, path: &RelativePath) -> Result<Box<dyn std::io::Write>> {
        self.deref().write_file(path)
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        self.deref().remove(path)
    }

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        self.deref().read_string(path)
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        self.deref().write_string(path, content)
    }
}

fn open_repository(config: &AnyRepositoryConfig) -> Result<Box<dyn Repository>> {
    match config {
        AnyRepositoryConfig::Local(local_config) => {
            Ok(Box::new(local_repository::open_repository(local_config)?))
        }
        AnyRepositoryConfig::KDE(kde_config) => {
            Ok(Box::new(kde_repository::open_repository(kde_config)?))
        }
    }
}

pub fn get_repository(config: &Option<AnyRepositoryConfig>) -> Result<Box<dyn Repository>> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    let repository = open_repository(config)?;
    if !repository.is_file(RelativePath::new("GameSaveSync.toml"))? {
        return Result::Err(Error::msg(format!(
            "Repository {config} has not been correctly initialized",
        )));
    }
    Ok(repository)
}

pub fn prepare_repository(config: &Option<AnyRepositoryConfig>) -> Result<()> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    let repository = open_repository(config)?;
    if !repository.is_file(RelativePath::new("GameSaveSync.toml"))? {
        if repository.read_dir(RelativePath::new(""))?.next().is_some() {
            return Result::Err(Error::msg(format!("Repository {config} should be empty")));
        }
        repository
            .write_string(RelativePath::new("GameSaveSync.toml"), "")
            .with_context(|| "failed to create repository")?;
    }
    Ok(())
}
