use std::collections::HashMap;

use anyhow::{Context, Result};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, UtcDateTime};

use crate::{repository::Repository, utils::paths::data_dir};

use super::definition::GameDefinition;

const MANIFEST_FILE: &str = "manifest.toml";

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GameSaveFileMetadata {
    pub modified: UtcDateTime,
    pub size: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSaveManifest {
    pub definition: GameDefinition,
    pub timestamp: Option<OffsetDateTime>,
    pub files: HashMap<String, HashMap<RelativePathBuf, GameSaveFileMetadata>>,
}

pub fn load_repository_manifest(
    repository: &impl Repository,
    game: &str,
) -> Result<Option<GameSaveManifest>> {
    let path = RelativePath::new(&game).join(&MANIFEST_FILE);
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

pub fn load_synced_manifest(game: &str) -> Result<Option<GameSaveManifest>> {
    let path = data_dir()?.join(&game).join(&MANIFEST_FILE);
    if !path.is_file() {
        return Ok(None);
    }
    let file = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read cached manifest for {game}"))?;
    let manifest = toml::from_str(&file)
        .with_context(|| format!("failed to parse cached manifest for {game}"))?;
    Ok(Some(manifest))
}
