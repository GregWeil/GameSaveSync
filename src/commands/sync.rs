use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use clap::Args;
use relative_path::{PathExt, RelativePathBuf};

use crate::{
    games::{
        definition::{GameDefinition, list_definitions, load_definition},
        manifest::{
            GameSaveFileMetadata, GameSaveManifest, load_repository_manifest, load_synced_manifest,
        },
        paths,
    },
    repository::{self, Repository, get_repository},
    utils::config,
};

#[derive(Args, Debug)]
pub struct SyncArgs {
    #[arg(help = "Sync only a specific game")]
    game: Option<String>,
}

pub fn sync(args: &SyncArgs) -> Result<()> {
    let config = config::load().with_context(|| "failed to load config")?;
    let repository = get_repository(&config.repository)?;
    match &args.game {
        Some(game) => sync_game(&game, &repository)?,
        None => {
            for game in list_definitions(&repository)? {
                sync_game(&game, &repository)?
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
enum SyncDirection {
    ToRepository,
    FromRepository,
    Conflict,
    NoChange,
}

fn sync_game(game: &str, repository: &impl Repository) -> Result<()> {
    println!("Synchronizing {}", game);
    let definition = load_definition(repository, game)?;
    let local_files = get_local_files(&definition)?;
    let repository_manifest = load_repository_manifest(repository, game)?;
    let synced_manifest = load_synced_manifest(game)?;
    let sync_direction = match &repository_manifest {
        Some(repository_manifest) => match &synced_manifest {
            Some(synced_manifest) => {
                let synced_files = get_manifest_files(synced_manifest)?;
                let repository_files = get_manifest_files(repository_manifest)?;
                let local_changed = !save_files_equal(&local_files, &synced_files);
                let repository_changed = save_files_equal(&synced_files, &repository_files);
                match (local_changed, repository_changed) {
                    (true, true) => SyncDirection::Conflict,
                    (true, false) => SyncDirection::ToRepository,
                    (false, true) => SyncDirection::FromRepository,
                    (false, false) => SyncDirection::NoChange,
                }
            }
            None => SyncDirection::Conflict,
        },
        None => SyncDirection::ToRepository,
    };
    println!("sync direction: {:?}", sync_direction);
    Ok(())
}

type ResolvedSaveFiles = HashMap<PathBuf, (String, RelativePathBuf, GameSaveFileMetadata)>;

fn save_files_equal(left: &ResolvedSaveFiles, right: &ResolvedSaveFiles) -> bool {
    if left.len() != right.len() {
        return false;
    }
    for (path, (_, _, left_metadata)) in left {
        match right.get(path) {
            Some((_, _, right_metadata)) if left_metadata == right_metadata => {}
            _ => return false,
        }
    }
    true
}

fn get_local_files(definition: &GameDefinition) -> Result<ResolvedSaveFiles> {
    let mut files = HashMap::new();
    for definition_path in &definition.paths {
        let root_path = paths::rewrite_path(&definition_path.path)?;
        let mut queue = vec![root_path.clone()];
        while let Some(path) = queue.pop() {
            if files.contains_key(&path) {
                continue;
            }
            if path.is_dir() {
                for item in path.read_dir()? {
                    queue.push(item?.path());
                }
            } else if path.is_file() {
                let relative_path = path.relative_to(&root_path)?;
                let metadata = path.metadata()?;
                let modified = time::UtcDateTime::from(metadata.modified()?);
                let size = metadata.len();
                files.insert(
                    path,
                    (
                        definition_path.path.clone(),
                        relative_path,
                        GameSaveFileMetadata { modified, size },
                    ),
                );
            } else {
                return Err(Error::msg(format!(
                    "save path {} is not a file or directory",
                    path.display()
                )));
            }
        }
    }
    Ok(files)
}

fn get_manifest_files(manifest: &GameSaveManifest) -> Result<ResolvedSaveFiles> {
    let mut files = HashMap::new();
    for (definition_path, path_files) in &manifest.files {
        let resolved_path = paths::rewrite_path(&definition_path)?;
        for (file_path, file_metadata) in path_files {
            files.insert(
                file_path.to_path(&resolved_path),
                (
                    definition_path.clone(),
                    file_path.clone(),
                    file_metadata.clone(),
                ),
            );
        }
    }
    Ok(files)
}
