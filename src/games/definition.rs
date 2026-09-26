use anyhow::{Context, Error, Result};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};

use super::GamePlatform;
use crate::repository::Repository;

const DEFINITION_FILE: &str = "definition.toml";

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GameDefinitionPath {
    pub path: String,
}

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GameDefinition {
    pub name: String,
    pub platform: GamePlatform,
    #[serde(default)]
    pub paths: Vec<GameDefinitionPath>,
    pub steam_app_id: Option<u64>,
}

pub fn definition_exists(repository: &impl Repository, game: &str) -> Result<bool> {
    let path = RelativePath::new(&game).join(DEFINITION_FILE);
    return Ok(repository.is_file(&path)?);
}

pub fn list_definitions(repository: &impl Repository) -> Result<Vec<String>> {
    let mut games = vec![];
    let items = repository
        .read_dir(RelativePath::new(""))
        .with_context(|| "failed to iterate repository")?;
    for path in items {
        match path?.file_name() {
            Some(name) if definition_exists(repository, name)? => games.push(name.into()),
            Some(_) => {}
            None => unreachable!("read_dir should never return an empty path"),
        }
    }
    Ok(games)
}

pub fn load_definition(repository: &impl Repository, game: &str) -> Result<GameDefinition> {
    if !definition_exists(repository, game)? {
        return Result::Err(Error::msg(format!("game {game} is not defined")));
    }
    let file = repository
        .read_string(&RelativePath::new(&game).join(DEFINITION_FILE))
        .with_context(|| format!("failed to read definition for {game}"))?;
    toml::from_str(&file).with_context(|| format!("failed to parse definition for {game}"))
}
