use std::path::{Path, PathBuf};

use anyhow::{Context, Error, Result};
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

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

fn copy_file(source: &Path, destination: &Path) -> Result<()> {
    match destination.parent() {
        Some(parent) => std::fs::create_dir_all(parent)?,
        None => {}
    }
    let mut source_file = std::fs::File::open(source)
        .with_context(|| format!("failed to open {} for reading", source.display()))?;
    let mut destination_file = std::fs::File::create(destination)
        .with_context(|| format!("failed to open {} for writing", destination.display()))?;
    std::io::copy(&mut source_file, &mut destination_file)
        .with_context(|| format!("failed to copy to {}", destination.display()))?;
    Ok(())
}

impl super::Repository for LocalRepository {
    fn metadata(&self, path: &RelativePath) -> Result<super::RepositoryPathMetadata> {
        let path = path.to_path(&self.path);
        let exists = path
            .try_exists()
            .with_context(|| format!("failed to check existence for {}", path.display()))?;
        if !exists {
            return Ok(super::RepositoryPathMetadata::DoesNotExist);
        }
        let metadata = path
            .metadata()
            .with_context(|| format!("failed to check metadata for {}", path.display()))?;
        if metadata.is_file() {
            return Ok(super::RepositoryPathMetadata::File);
        }
        if metadata.is_dir() {
            return Ok(super::RepositoryPathMetadata::Directory);
        }
        Ok(super::RepositoryPathMetadata::Other)
    }

    fn read_dir(
        &self,
        path: &RelativePath,
    ) -> Result<Box<dyn Iterator<Item = Result<RelativePathBuf>>>> {
        let path = path.to_path(&self.path);
        let iterator = path
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
            });
        Ok(Box::new(iterator))
    }

    fn read_string(&self, path: &RelativePath) -> Result<String> {
        let path = path.to_path(&self.path);
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        Ok(content)
    }

    fn write_string(&self, path: &RelativePath, content: &str) -> Result<()> {
        let path = path.to_path(&self.path);
        match path.parent() {
            Some(parent) => std::fs::create_dir_all(parent)?,
            None => {}
        }
        std::fs::write(&path, content)
            .with_context(|| format!("failed to write {}", path.display()))?;
        Ok(())
    }

    fn download_file(&self, repository_path: &RelativePath, local_path: &Path) -> Result<()> {
        let repository_path = repository_path.to_path(&self.path);
        copy_file(&repository_path, local_path)
            .with_context(|| format!("failed to copy to {}", local_path.display()))?;
        Ok(())
    }

    fn upload_file(&self, local_path: &Path, repository_path: &RelativePath) -> Result<()> {
        let repository_path = repository_path.to_path(&self.path);
        copy_file(local_path, &repository_path)
            .with_context(|| format!("failed to copy from {}", local_path.display()))?;
        Ok(())
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
            "Repository path {} is not absolute",
            config.path.display()
        )));
    }
    Ok(LocalRepository {
        path: config.path.clone(),
    })
}
