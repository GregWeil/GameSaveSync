use std::io::Write;

use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};

pub mod any_repository;
pub mod local_repository;

pub use any_repository::{AnyRepository, AnyRepositoryConfig};

pub trait Repository {
    fn is_file(&self, path: &RelativePath) -> Result<bool>;
    fn is_dir(&self, path: &RelativePath) -> Result<bool>;
    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<impl Iterator<Item = Result<RelativePathBuf>>>;
    fn read_file(&self, path: &RelativePath) -> Result<impl std::io::Read>;
    fn write_file(&self, path: &RelativePath) -> Result<impl std::io::Write>;
    fn remove(&self, path: &RelativePath) -> Result<()>;
}

pub trait RepositoryExt {
    fn read_string(&self, path: &RelativePath) -> Result<String>;
    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()>;
}

impl<T: Repository> RepositoryExt for T {
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

fn open_repository(config: &AnyRepositoryConfig) -> Result<AnyRepository> {
    let repository = match config {
        AnyRepositoryConfig::Local(local_config) => {
            AnyRepository::Local(local_repository::open_repository(local_config)?)
        }
    };
    Ok(repository)
}

pub fn get_repository(config: &Option<AnyRepositoryConfig>) -> Result<AnyRepository> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    let repository = open_repository(config)?;
    if !repository.is_file(RelativePath::new("GameSaveSync.toml"))? {
        return Result::Err(Error::msg(format!(
            "Repository {} has not been correctly initialized",
            config
        )));
    }
    Ok(repository)
}

pub fn prepare_repository(config: &Option<AnyRepositoryConfig>) -> Result<()> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    let repository = open_repository(config)?;
    if !repository.is_file(RelativePath::new("GameSaveSync.toml"))? {
        if repository.read_dir(RelativePath::new(""))?.next().is_some() {
            return Result::Err(Error::msg(format!("Repository {} should be empty", config)));
        }
        repository
            .write_string(RelativePath::new("GameSaveSync.toml"), "")
            .with_context(|| "failed to create repository")?;
    }
    Ok(())
}
