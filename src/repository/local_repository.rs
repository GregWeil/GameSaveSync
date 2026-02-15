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
    fn is_file<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().to_path(&self.path);
        Ok(path.is_file())
    }

    fn is_dir<P>(&self, path: P) -> Result<bool>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().to_path(&self.path);
        Ok(path.is_dir())
    }

    fn read_dir<P>(&self, path: P) -> Result<impl Iterator<Item = Result<RelativePathBuf>>>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().to_path(&self.path);
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

    fn read_string<P>(&self, path: P) -> Result<String>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().to_path(&self.path);
        std::fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))
    }

    fn write_string<P>(&self, path: P, content: &str) -> Result<()>
    where
        P: AsRef<RelativePath>,
    {
        let path = path.as_ref().to_path(&self.path);
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
