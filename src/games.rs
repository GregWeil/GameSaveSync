use anyhow::{Context, Error, Result};
use relative_path::{RelativePath, RelativePathBuf};
use serde::{Deserialize, Serialize};

use crate::repository::Repository;

#[derive(Serialize, Deserialize, Debug)]
pub enum GamePlatform {
    Linux,
}

impl std::fmt::Display for GamePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GamePlatform::Linux => write!(f, "Linux"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDefinitionPath {
    pub path: RelativePathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDefinition {
    pub name: String,
    pub platform: GamePlatform,
    #[serde(default)]
    pub paths: Vec<GameDefinitionPath>,
    pub steam_app_id: Option<u64>,
}

pub fn load_definition(repository: &impl Repository, game: &String) -> Result<GameDefinition> {
    let path = RelativePath::new(&game).join("definition.toml");
    if !repository.is_file(&path)? {
        return Result::Err(Error::msg(format!("game {game} is not defined")));
    }
    let file = repository
        .read_string(&path)
        .with_context(|| format!("failed to read definition for {game}"))?;
    toml::from_str(&file).with_context(|| format!("failed to parse definition for {game}"))
}
