use core::fmt;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use super::Repository;

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalRepositoryConfig {
    pub path: PathBuf,
}

impl fmt::Display for LocalRepositoryConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.path.display().fmt(f)
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

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        let path = path.to_path(&self.path);
        std::fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        let path = path.to_path(&self.path);
        std::fs::create_dir_all(&path)?;
        std::fs::write(&path, &content)?;
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
