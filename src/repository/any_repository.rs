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
    fn is_file<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>,
    {
        match self {
            AnyRepository::Local(repository) => repository.is_file(path),
        }
    }

    fn is_dir<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>,
    {
        match self {
            AnyRepository::Local(repository) => repository.is_dir(path),
        }
    }

    fn read_dir<P>(&self, path: P) -> Result<impl Iterator<Item = Result<RelativePathBuf>>>
    where
        P: AsRef<RelativePath>,
    {
        match self {
            AnyRepository::Local(repository) => repository.read_dir(path),
        }
    }

    fn read_string<P>(&self, path: P) -> Result<String>
    where
        P: AsRef<RelativePath>,
    {
        match self {
            AnyRepository::Local(repository) => repository.read_string(path),
        }
    }

    fn write_string<P>(&self, path: P, content: &str) -> Result<()>
    where
        P: AsRef<RelativePath>,
    {
        match self {
            AnyRepository::Local(repository) => repository.write_string(path, content),
        }
    }
}
