use anyhow::Result;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnyRepositoryConfig {
    Local(super::local_repository::LocalRepositoryConfig),
}

impl std::fmt::Display for AnyRepositoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AnyRepositoryConfig::Local(config) => write!(f, "{}", config),
        }
    }
}

#[derive(Debug)]
pub enum AnyRepository {
    Local(super::local_repository::LocalRepository),
}

impl super::Repository for AnyRepository {
    fn is_file(&self, path: &RelativePath) -> Result<bool> {
        match self {
            AnyRepository::Local(repository) => repository.is_file(path),
        }
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        match self {
            AnyRepository::Local(repository) => repository.is_dir(path),
        }
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<impl Iterator<Item = Result<RelativePathBuf>>> {
        match self {
            AnyRepository::Local(repository) => repository.read_dir(path),
        }
    }

    fn read_file(&self, path: &RelativePath) -> Result<impl std::io::Read> {
        match self {
            AnyRepository::Local(repository) => repository.read_file(path),
        }
    }

    fn write_file(&self, path: &RelativePath) -> Result<impl std::io::Write> {
        match self {
            AnyRepository::Local(repository) => repository.write_file(path),
        }
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        match self {
            AnyRepository::Local(repository) => repository.remove(path),
        }
    }
}
