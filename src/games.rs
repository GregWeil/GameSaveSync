use serde::{Deserialize, Serialize};

pub mod definition;
pub mod manifest;
pub mod paths;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Debug)]
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
