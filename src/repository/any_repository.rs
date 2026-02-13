use core::fmt;

use anyhow::Result;
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum AnyRepositoryConfig {
    Local(super::local_repository::LocalRepositoryConfig),
}

impl fmt::Display for AnyRepositoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AnyRepositoryConfig::Local(config) => config.fmt(f),
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
            AnyRepository::Local(repository) => repository.is_file(&path),
        }
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        match self {
            AnyRepository::Local(repository) => repository.is_dir(&path),
        }
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<impl Iterator<Item = Result<RelativePathBuf>>> {
        match self {
            AnyRepository::Local(repository) => repository.read_dir(&path),
        }
    }

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        match self {
            AnyRepository::Local(repository) => repository.read_string(&path),
        }
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        match self {
            AnyRepository::Local(repository) => repository.write_string(&path, content),
        }
    }
}
