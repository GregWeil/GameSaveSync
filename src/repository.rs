use std::{ops::Deref, path::Path};

use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

pub mod kde_repository;
pub mod local_repository;

const REPOSITORY_FILE: &str = "GameSaveSync.toml";

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

pub enum RepositoryPathMetadata {
    File,
    Directory,
    Other,
    DoesNotExist,
}

pub trait Repository {
    fn metadata(&self, path: &RelativePath) -> Result<RepositoryPathMetadata>;
    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>>;
    fn read_string(&self, path: &RelativePath) -> Result<String>;
    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()>;
    fn download_file(&self, repository_path: &RelativePath, local_path: &Path) -> Result<()>;
    fn upload_file(&self, local_path: &Path, repository_path: &RelativePath) -> Result<()>;
    fn remove(&self, path: &RelativePath) -> Result<()>;
}

impl Repository for Box<dyn Repository> {
    fn metadata(&self, path: &RelativePath) -> Result<RepositoryPathMetadata> {
        self.deref().metadata(path)
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>> {
        self.deref().read_dir(path)
    }

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        self.deref().read_string(path)
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        self.deref().write_string(path, content)
    }

    fn download_file(&self, repository_path: &RelativePath, local_path: &Path) -> Result<()> {
        self.deref().download_file(repository_path, local_path)
    }

    fn upload_file(&self, local_path: &Path, repository_path: &RelativePath) -> Result<()> {
        self.deref().upload_file(local_path, repository_path)
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        self.deref().remove(path)
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
    if !matches!(
        repository.metadata(RelativePath::new(REPOSITORY_FILE))?,
        RepositoryPathMetadata::File
    ) {
        return Result::Err(Error::msg(format!(
            "Repository {config} has not been correctly initialized",
        )));
    }
    Ok(repository)
}

pub fn prepare_repository(config: &Option<AnyRepositoryConfig>) -> Result<()> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    let repository = open_repository(config)?;
    if !matches!(
        repository.metadata(RelativePath::new(REPOSITORY_FILE))?,
        RepositoryPathMetadata::File
    ) {
        if repository.read_dir(RelativePath::new(""))?.next().is_some() {
            return Result::Err(Error::msg(format!("Repository {config} should be empty")));
        }
        repository
            .write_string(RelativePath::new(REPOSITORY_FILE), "")
            .with_context(|| "failed to create repository")?;
    }
    Ok(())
}
