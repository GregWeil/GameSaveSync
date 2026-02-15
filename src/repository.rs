use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};

pub mod any_repository;
pub mod local_repository;

pub use any_repository::{AnyRepository, AnyRepositoryConfig};

pub trait Repository {
    fn is_file<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>;
    fn is_dir<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>;
    fn read_dir<P>(&self, path: P) -> Result<impl Iterator<Item = Result<RelativePathBuf>>>
    where
        P: AsRef<RelativePath>;
    fn read_string<P>(&self, path: P) -> Result<String>
    where
        P: AsRef<RelativePath>;
    fn write_string<P>(&self, path: P, content: &str) -> Result<()>
    where
        P: AsRef<RelativePath>;
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
