use anyhow::{Context, Error, Result};
use relative_path::RelativePath;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::repository::Repository;

const STATE_FILE: &str = "state.toml";

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct GameState {
    pub current: Option<Uuid>,
}

pub fn read_game_state(repository: &impl Repository, game: &str) -> Result<GameState> {
    let path = RelativePath::new(&game).join(STATE_FILE);
    if !repository.is_file(&path)? {
        return Ok(GameState { current: None });
    }
    let file = repository
        .read_string(&path)
        .with_context(|| format!("failed to read game state for {game}"))?;
    let state =
        toml::from_str(&file).with_context(|| format!("failed to parse game state for {game}"))?;
    Ok(state)
}

pub fn write_game_state(state: &GameState, repository: &impl Repository, game: &str) -> Result<()> {
    let path = RelativePath::new(game).join(STATE_FILE);
    match path.parent() {
        Some(dir) if !repository.is_dir(dir)? => {
            return Err(Error::msg("game directory should already exist for {game}"));
        }
        Some(_) => {}
        None => unreachable!("game state file should always have a parent path"),
    }
    let serialized =
        toml::to_string_pretty(state).with_context(|| "failed to serialize game state")?;
    repository
        .write_string(&path, &serialized)
        .with_context(|| "failed to write game state")?;
    Ok(())
}
