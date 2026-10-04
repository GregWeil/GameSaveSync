use anyhow::{Context, Error, Result};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::{Repository, RepositoryPathMetadata};

const STATE_FILE: &str = "state.toml";

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct RepositoryGameState {
    pub current: Option<Uuid>,
}

pub fn read_repository_game_state(
    repository: &impl Repository,
    game: &str,
) -> Result<RepositoryGameState> {
    let path = RelativePath::new(&game).join(STATE_FILE);
    if !matches!(repository.metadata(&path)?, RepositoryPathMetadata::File) {
        return Ok(RepositoryGameState { current: None });
    }
    let file = repository
        .read_string(&path)
        .with_context(|| format!("failed to read game state for {game}"))?;
    let state =
        toml::from_str(&file).with_context(|| format!("failed to parse game state for {game}"))?;
    Ok(state)
}

pub fn write_repository_game_state(
    state: &RepositoryGameState,
    repository: &impl Repository,
    game: &str,
) -> Result<()> {
    let path = RelativePath::new(game).join(STATE_FILE);
    match path.parent() {
        Some(dir) => match repository.metadata(&dir)? {
            RepositoryPathMetadata::Directory => {}
            _ => {
                return Err(Error::msg(format!(
                    "game directory should already exist for {game}"
                )));
            }
        },
        None => unreachable!("game state file should always have a parent path"),
    }
    let serialized =
        toml::to_string_pretty(state).with_context(|| "failed to serialize game state")?;
    repository
        .write_string(&path, &serialized)
        .with_context(|| "failed to write game state")?;
    Ok(())
}
