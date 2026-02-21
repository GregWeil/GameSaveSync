use std::collections::HashMap;

use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};
use uuid::Uuid;

use crate::{
    repository::{Repository, RepositoryExt},
    utils::paths::data_dir,
};

use super::definition::GameDefinition;

const MANIFEST_FILE: &str = "manifest.toml";

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GameSaveFileMetadata {
    pub modified: UtcDateTime,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSaveManifest {
    pub id: Uuid,
    pub definition: GameDefinition,
    pub timestamp: OffsetDateTime,
    pub files: HashMap<String, HashMap<RelativePathBuf, GameSaveFileMetadata>>,
}

pub fn read_repository_manifest(
    repository: &impl Repository,
    game: &str,
) -> Result<Option<GameSaveManifest>> {
    let path = RelativePath::new(&game).join(MANIFEST_FILE);
    if !repository.is_file(&path)? {
        return Ok(None);
    }
    let file = repository
        .read_string(&path)
        .with_context(|| format!("failed to read manifest for {game}"))?;
    let manifest =
        toml::from_str(&file).with_context(|| format!("failed to parse manifest for {game}"))?;
    Ok(Some(manifest))
}

pub fn write_repository_manifest(
    manifest: &GameSaveManifest,
    repository: &impl Repository,
) -> Result<()> {
    let path = RelativePath::new(&manifest.definition.name).join(MANIFEST_FILE);
    match path.parent() {
        Some(dir) if !repository.is_dir(dir)? => {
            return Err(Error::msg(
                "repository manifest directory should already exist",
            ));
        }
        Some(_) => {}
        None => unreachable!("repository manifest file should always have a parent path"),
    }
    let serialized = toml::to_string_pretty(manifest)
        .with_context(|| "failed to serialize repository manifest")?;
    repository
        .write_string(&path, &serialized)
        .with_context(|| "failed to write repository manifest")?;
    Ok(())
}

pub fn read_synced_manifest(game: &str) -> Result<Option<GameSaveManifest>> {
    let path = data_dir()?.join(&game).join(MANIFEST_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let file = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read cached manifest for {game}"))?;
    let manifest = toml::from_str(&file)
        .with_context(|| format!("failed to parse cached manifest for {game}"))?;
    Ok(Some(manifest))
}

pub fn write_synced_manifest(manifest: &GameSaveManifest) -> Result<()> {
    let path = data_dir()?
        .join(&manifest.definition.name)
        .join(MANIFEST_FILE);
    match path.parent() {
        Some(dir) => std::fs::create_dir_all(dir)
            .with_context(|| "failed to create synced manifest directory")?,
        None => unreachable!("synced manifest file should always have a parent path"),
    }
    let serialized =
        toml::to_string_pretty(manifest).with_context(|| "failed to serialize synced manifest")?;
    std::fs::write(path, serialized).with_context(|| "failed to write synced manifest")?;
    Ok(())
}
