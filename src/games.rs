use serde::{Deserialize, Serialize};

pub mod definition;

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
