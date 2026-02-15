use relative_path::RelativePathBuf;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum GamePlatform {
    Linux,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDefinitionPath {
    path: RelativePathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameDefinition {
    name: String,
    platform: GamePlatform,
    paths: Vec<GameDefinitionPath>,
    steam_app_id: Option<u64>,
}

impl std::fmt::Display for GameDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
