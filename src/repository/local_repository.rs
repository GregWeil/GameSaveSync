use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use super::Repository;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalRepositoryConfig {
    pub path: PathBuf,
}

impl std::fmt::Display for LocalRepositoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

#[derive(Debug)]
pub struct LocalRepository {
    path: PathBuf,
}

impl Repository for LocalRepository {
    fn is_file(&self, path: &RelativePath) -> Result<bool> {
        let path = path.to_path(&self.path);
        Ok(path.is_file())
    }

    fn is_dir(&self, path: &RelativePath) -> Result<bool> {
        let path = path.to_path(&self.path);
        Ok(path.is_dir())
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<impl Iterator<Item = Result<RelativePathBuf>>> {
        let path = path.to_path(&self.path);
        Ok(path
            .read_dir()
            .with_context(|| format!("failed to enumerate {}", path.display()))?
            .map(move |entry| -> Result<RelativePathBuf> {
                let entry = entry?;
                entry.path().relative_to(&path).with_context(|| {
                    format!(
                        "failed to enumerate {} in {}",
                        entry.file_name().display(),
                        path.display()
                    )
                })
            }))
    }

    fn read_file(&self, path: &RelativePath) -> Result<impl std::io::Read> {
        let path = path.to_path(&self.path);
        std::fs::File::open(&path).with_context(|| format!("failed to read {}", path.display()))
    }

    fn write_file(&self, path: &RelativePath) -> Result<impl std::io::Write> {
        let path = path.to_path(&self.path);
        match path.parent() {
            Some(parent) => std::fs::create_dir_all(parent)?,
            None => {}
        }
        std::fs::File::create(&path).with_context(|| format!("failed to read {}", path.display()))
    }

    fn remove(&self, path: &RelativePath) -> Result<()> {
        let path = path.to_path(&self.path);
        if path.is_file() {
            std::fs::remove_file(path)?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(path)?;
        }
        Ok(())
    }
}

pub fn open_repository(config: &LocalRepositoryConfig) -> Result<LocalRepository> {
    if !config.path.is_absolute() {
        return Result::Err(Error::msg(format!(
            "Path {} is not absolute",
            config.path.display()
        )));
    }
    if !config.path.exists() {
        return Result::Err(Error::msg(format!(
            "Path {} does not exist",
            config.path.display()
        )));
    }
    if !config.path.is_dir() {
        return Result::Err(Error::msg(format!(
            "Path {} is not a directory",
            config.path.display()
        )));
    }
    Ok(LocalRepository {
        path: config.path.clone(),
    })
}
