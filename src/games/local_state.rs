use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::utils::paths::data_dir;

const STATE_FILE: &str = "state.toml";

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum PendingSyncAction {
    Store {
        start_time: OffsetDateTime,
    },
    Apply {
        start_time: OffsetDateTime,
        manifest_id: Uuid,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LocalGameState {
    pub last_synced: Option<OffsetDateTime>,
    pub pending_action: Option<PendingSyncAction>,
}

pub fn read_local_game_state(game: &str) -> Result<LocalGameState> {
    let path = data_dir()?.join(&game).join(STATE_FILE);
    if !path.is_file() {
        return Ok(LocalGameState {
            last_synced: None,
            pending_action: None,
        });
    }
    let file = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read local state for {game}"))?;
    let state =
        toml::from_str(&file).with_context(|| format!("failed to parse local state for {game}"))?;
    Ok(state)
}

fn write_local_game_state(game: &str, state: &LocalGameState) -> Result<()> {
    let path = data_dir()?.join(&game).join(STATE_FILE);
    match path.parent() {
        Some(dir) => std::fs::create_dir_all(dir)
            .with_context(|| "failed to create local state directory")?,
        None => unreachable!("local state file should always have a parent path"),
    }
    let serialized = toml::to_string_pretty(state)
        .with_context(|| format!("failed to serialize local state for {game}"))?;
    std::fs::write(path, serialized)
        .with_context(|| format!("failed to write local state for {game}"))?;
    Ok(())
}

pub fn write_local_game_synced(game: &str) -> Result<()> {
    let state = LocalGameState {
        last_synced: Some(OffsetDateTime::now_local()?),
        pending_action: None,
    };
    write_local_game_state(&game, &state)
}

pub fn write_local_game_store(game: &str) -> Result<()> {
    let old_state = read_local_game_state(&game)?;
    let new_state = LocalGameState {
        last_synced: old_state.last_synced,
        pending_action: Some(PendingSyncAction::Store {
            start_time: OffsetDateTime::now_local()?,
        }),
    };
    write_local_game_state(&game, &new_state)
}

pub fn write_local_game_apply(game: &str, manifest_id: &Uuid) -> Result<()> {
    let old_state = read_local_game_state(&game)?;
    let new_state = LocalGameState {
        last_synced: old_state.last_synced,
        pending_action: Some(PendingSyncAction::Apply {
            start_time: OffsetDateTime::now_local()?,
            manifest_id: manifest_id.clone(),
        }),
    };
    write_local_game_state(&game, &new_state)
}
