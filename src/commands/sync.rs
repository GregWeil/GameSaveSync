use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use clap::Args;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use time::{OffsetDateTime, UtcDateTime};
use uuid::Uuid;

use crate::{
    games::{
        definition::{GameDefinition, list_definitions, load_definition},
        manifest::{
            GameSaveFileMetadata, GameSaveManifest, read_repository_manifest, read_synced_manifest,
            write_repository_manifest, write_synced_manifest,
        },
        paths::{self, rewrite_path},
    },
    repository::{Repository, get_repository},
    utils::{config, paths::make_path_safe},
};

#[derive(Args, Debug)]
pub struct SyncArgs {
    #[arg(help = "Sync only a specific game")]
    game: Option<String>,
    #[arg(short, long, help = "Simulate without modifying files")]
    dry_run: bool,
}

pub fn sync(args: &SyncArgs) -> Result<()> {
    let config = config::load().with_context(|| "failed to load config")?;
    let repository = get_repository(&config.repository)?;
    match &args.game {
        Some(game) => sync_game(&game, &repository, args)?,
        None => {
            for game in list_definitions(&repository)? {
                sync_game(&game, &repository, &args)?
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
enum SyncDirection {
    ToRepository,
    FromRepository,
    DoNothing,
}

fn sync_game(game: &str, repository: &impl Repository, args: &SyncArgs) -> Result<()> {
    println!("Checking {}", game);
    let definition = load_definition(repository, game)?;
    let local_files = get_local_files(&definition)?;
    let repository_state = match read_repository_manifest(repository, game)? {
        Some(manifest) => Some((get_manifest_files(&manifest)?, manifest)),
        None => None,
    };
    let synced_state = match read_synced_manifest(game)? {
        Some(manifest) => Some((get_manifest_files(&manifest)?, manifest)),
        None => None,
    };
    let sync_direction = match &repository_state {
        Some((repository_files, repository_manifest)) => match &synced_state {
            Some((synced_files, synced_manifest)) => {
                let local_changed = !save_files_equal(&local_files, &synced_files);
                let repository_changed = !save_files_equal(&synced_files, &repository_files);
                match (local_changed, repository_changed) {
                    (true, true) => {
                        let local_last_mod = local_files.values().map(|file| file.2.modified).max();
                        conflict_prompt(
                            &format!(
                                "{} has changed here and in the repository (last synced at {})",
                                &game, &synced_manifest.timestamp
                            ),
                            local_last_mod,
                            repository_manifest.timestamp,
                        )?
                    }
                    (true, false) => SyncDirection::ToRepository,
                    (false, true) => SyncDirection::FromRepository,
                    (false, false) => SyncDirection::DoNothing,
                }
            }
            // No local manifest
            None => {
                let local_last_mod = local_files.values().map(|file| file.2.modified).max();
                conflict_prompt(
                    &format!("{} has not been synced to this device", &game),
                    local_last_mod,
                    repository_manifest.timestamp,
                )?
            }
        },
        // No repository manifest
        None => SyncDirection::ToRepository,
    };
    match sync_direction {
        SyncDirection::ToRepository => {
            println!("- Storing save in repository");
            sync_game_to_repository(&definition, &local_files, repository, args)?;
        }
        SyncDirection::FromRepository => {
            println!("- Applying save from repository");
            match repository_state {
                Some((files, manifest)) => {
                    sync_game_from_repository(&manifest, &files, repository, args)?
                }
                None => {
                    unreachable!("impossible to sync from repository with no repository manifest")
                }
            }
        }
        SyncDirection::DoNothing => {}
    }
    Ok(())
}

struct ConflictChoice {
    sync_direction: SyncDirection,
    label: String,
}

impl std::fmt::Display for ConflictChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.label)
    }
}

fn conflict_prompt(
    message: &str,
    local_last_mod: Option<UtcDateTime>,
    repository_synced: OffsetDateTime,
) -> Result<SyncDirection> {
    let offset = match time::UtcOffset::current_local_offset() {
        Ok(local_offset) => local_offset,
        Err(_) => repository_synced.offset(),
    };
    let choice = inquire::Select::new(
        message,
        vec![
            ConflictChoice {
                sync_direction: SyncDirection::DoNothing,
                label: format!("Do nothing"),
            },
            ConflictChoice {
                sync_direction: SyncDirection::ToRepository,
                label: format!(
                    "Keep local device save{}",
                    match local_last_mod {
                        Some(local_last_mod) =>
                            format!(" (modified {})", local_last_mod.to_offset(offset)),
                        None => "".into(),
                    }
                ),
            },
            ConflictChoice {
                sync_direction: SyncDirection::FromRepository,
                label: format!(
                    "Keep repository save (synced {})",
                    repository_synced.to_offset(offset)
                ),
            },
        ],
    )
    .prompt()
    .with_context(|| "failed to prompt for sync direction")?;
    Ok(choice.sync_direction)
}

fn sync_game_to_repository(
    definition: &GameDefinition,
    local_files: &ResolvedSaveFiles,
    repository: &impl Repository,
    args: &SyncArgs,
) -> Result<()> {
    let manifest_id = Uuid::new_v4();
    let mut files = HashMap::new();
    for (real_path, (path, file, metadata)) in local_files {
        if args.dry_run {
            println!("- Storing {} in repository", real_path.display());
        } else {
            let repository_path = RelativePath::new(&definition.name)
                .join(manifest_id.to_string())
                .join(make_path_safe(path))
                .join(file);
            let mut local_file = std::fs::File::open(real_path)?;
            let mut repository_file = repository.write_file(&repository_path)?;
            std::io::copy(&mut local_file, &mut repository_file)
                .with_context(|| format!("failed to copy {} to repository", real_path.display()))?;
        }
        files
            .entry(path.clone())
            .or_insert_with(|| HashMap::new())
            .insert(file.clone(), metadata.clone());
    }
    let manifest = GameSaveManifest {
        id: manifest_id,
        definition: definition.clone(),
        timestamp: time::OffsetDateTime::now_local()?,
        files,
    };
    let old_manifest = read_repository_manifest(repository, &definition.name)?;
    if !args.dry_run {
        write_repository_manifest(&manifest, repository)?;
        write_synced_manifest(&manifest)?;
        match old_manifest {
            Some(old_manifest) => repository
                .remove(&RelativePath::new(&definition.name).join(old_manifest.id.to_string()))?,
            None => {}
        }
    }
    Ok(())
}

fn sync_game_from_repository(
    manifest: &GameSaveManifest,
    repository_files: &ResolvedSaveFiles,
    repository: &impl Repository,
    args: &SyncArgs,
) -> Result<()> {
    for path in &manifest.definition.paths {
        let path = rewrite_path(&path.path)?;
        if repository_files.contains_key(&path) {
            continue;
        }
        if args.dry_run {
            println!("- Removing save file at {}", path.display());
        } else if path.is_file() {
            std::fs::remove_file(&path)?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path)?;
        }
    }
    for (real_path, (path, file, metadata)) in repository_files {
        if args.dry_run {
            println!("Applying save file {}", real_path.display())
        } else {
            let repository_path = RelativePath::new(&manifest.definition.name)
                .join(manifest.id.to_string())
                .join(make_path_safe(path))
                .join(file);
            let mut repository_file = repository.read_file(&repository_path)?;
            match real_path.parent() {
                Some(parent) => std::fs::create_dir_all(parent)?,
                None => unreachable!("save file paths must have a parent"),
            }
            let mut local_file = std::fs::File::create(real_path)
                .with_context(|| format!("failed to create file at {}", real_path.display()))?;
            std::io::copy(&mut repository_file, &mut local_file)
                .with_context(|| format!("failed to copy {} to repository", real_path.display()))?;
            local_file
                .set_modified(metadata.modified.into())
                .with_context(|| {
                    format!("failed to set modified time on {}", real_path.display())
                })?;
        }
    }
    if !args.dry_run {
        write_synced_manifest(&GameSaveManifest {
            id: manifest.id,
            definition: manifest.definition.clone(),
            timestamp: time::OffsetDateTime::now_local()?,
            files: manifest.files.clone(),
        })?;
    }
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
            _ => {
                println!("{} {:?} {:?}",path.display(),left_metadata,right.get(path));
                return false},
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
            } else if path.exists() {
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
