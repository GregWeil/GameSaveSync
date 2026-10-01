use std::collections::HashMap;
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use clap::Args;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use time::{OffsetDateTime, UtcDateTime};
use uuid::Uuid;

use crate::games::state::{GameState, read_game_state, write_game_state};
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

#[cfg(test)]
mod tests;

#[derive(Args, Debug)]
pub struct SyncArgs {
    #[arg(help = "Sync only a specific game")]
    game: Option<String>,
    #[arg(short, long, help = "Simulate without modifying files")]
    dry_run: bool,
    #[arg(
        long,
        conflicts_with = "force_apply",
        help = "Store the local save in the repository regardless of sync state"
    )]
    force_store: bool,
    #[arg(
        long,
        conflicts_with = "force_store",
        help = "Apply the repository save over the local save regardless of sync state"
    )]
    force_apply: bool,
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

#[derive(Debug, PartialEq)]
enum SyncDirection {
    ToRepository,
    FromRepository,
    DoNothing,
}

#[derive(Debug, PartialEq)]
enum SyncDetermination {
    Automatic(SyncDirection),
    Conflict(String, Option<UtcDateTime>, OffsetDateTime),
}

fn sync_game(game: &str, repository: &impl Repository, args: &SyncArgs) -> Result<()> {
    let definition = load_definition(repository, game)?;
    let game = &definition.name; // fixes casing or other formatting
    println!("Checking {}", game);
    let local_save = get_local_save(&definition)?;
    let repository_state = match read_game_state(repository, game)?.current {
        Some(manifest_id) => match read_repository_manifest(repository, game, &manifest_id)? {
            Some(manifest) => Some((get_manifest_save(&manifest)?, manifest)),
            None => None,
        },
        None => None,
    };
    let synced_state = match read_synced_manifest(game)? {
        Some(manifest) => Some((get_manifest_save(&manifest)?, manifest)),
        None => None,
    };
    let sync_direction = determine_sync_direction(
        &definition,
        &local_save,
        &synced_state,
        &repository_state,
        &args,
    )?;
    let sync_direction = match sync_direction {
        SyncDetermination::Automatic(direction) => direction,
        SyncDetermination::Conflict(message, local_last_mod, repository_synced) => {
            conflict_prompt(&message, local_last_mod, repository_synced)?
        }
    };
    match sync_direction {
        SyncDirection::ToRepository => {
            println!("- Storing save in repository");
            sync_game_to_repository(&definition, &local_save, repository, args)?;
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
        SyncDirection::DoNothing => {
            println!("- Taking no action");
        }
    }
    Ok(())
}

fn determine_sync_direction(
    definition: &GameDefinition,
    local_save: &ResolvedSave,
    synced_state: &Option<(ResolvedSave, GameSaveManifest)>,
    repository_state: &Option<(ResolvedSave, GameSaveManifest)>,
    args: &SyncArgs,
) -> Result<SyncDetermination> {
    if args.force_store && args.force_apply {
        print!("- Unable to simultaneously force-store and force-apply");
        return Ok(SyncDetermination::Automatic(SyncDirection::DoNothing));
    }
    if args.force_store {
        return Ok(SyncDetermination::Automatic(SyncDirection::ToRepository));
    }
    if args.force_apply {
        match repository_state {
            Some(_) => {
                return Ok(SyncDetermination::Automatic(SyncDirection::FromRepository));
            }
            None => {
                print!("- No save in repository to apply");
                return Ok(SyncDetermination::Automatic(SyncDirection::DoNothing));
            }
        }
    }
    let direction = match repository_state {
        Some((repository_save, repository_manifest)) => match synced_state {
            Some((synced_save, synced_manifest)) => {
                let local_changed = !saves_are_equal(&local_save, &synced_save);
                let repository_changed = !saves_are_equal(&synced_save, &repository_save);
                match (local_changed, repository_changed) {
                    (true, true) => {
                        let local_last_mod =
                            local_save.files.values().map(|file| file.2.modified).max();
                        SyncDetermination::Conflict(
                            format!(
                                "{} has changed here and in the repository (last synced at {})",
                                &definition.name, &synced_manifest.timestamp
                            ),
                            local_last_mod,
                            repository_manifest.timestamp,
                        )
                    }
                    (true, false) => SyncDetermination::Automatic(SyncDirection::ToRepository),
                    (false, true) => SyncDetermination::Automatic(SyncDirection::FromRepository),
                    (false, false) => SyncDetermination::Automatic(SyncDirection::DoNothing),
                }
            }
            // No local manifest
            None => {
                let local_last_mod = local_save.files.values().map(|file| file.2.modified).max();
                SyncDetermination::Conflict(
                    format!("{} has not been synced to this device", &definition.name),
                    local_last_mod,
                    repository_manifest.timestamp,
                )
            }
        },
        // No repository manifest
        None => SyncDetermination::Automatic(SyncDirection::ToRepository),
    };
    Ok(direction)
}

struct ConflictChoice {
    sync_direction: SyncDirection,
    label: String,
}

impl std::fmt::Display for ConflictChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.label)?;
        Ok(())
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
    local_save: &ResolvedSave,
    repository: &impl Repository,
    args: &SyncArgs,
) -> Result<()> {
    let manifest_id = Uuid::new_v4();
    let save_path = RelativePath::new(&definition.name).join(manifest_id.to_string());
    repository
        .make_dir(&save_path)
        .with_context(|| format!("failed to sync {} to repository", &definition.name))?;
    let mut directories = HashMap::new();
    for (_, (path, directory)) in &local_save.directories {
        if !args.dry_run {
            let repository_path = save_path.join(make_path_safe(path)).join(directory);
            repository
                .make_dir(&repository_path)
                .with_context(|| format!("failed to sync {} to repository", &definition.name))?;
        }
        directories
            .entry(path.clone())
            .or_insert_with(|| HashSet::new())
            .insert(directory.clone());
    }
    let mut files = HashMap::new();
    for (real_path, (path, file, metadata)) in &local_save.files {
        if args.dry_run {
            println!("- Storing {} in repository", real_path.display());
        } else {
            let repository_path = save_path.join(make_path_safe(path)).join(file);
            repository
                .upload_file(&real_path, &repository_path)
                .with_context(|| format!("failed to sync {} to repository", &definition.name))?;
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
        directories,
        files,
    };
    if !args.dry_run {
        let old_state = read_game_state(repository, &definition.name)?;
        write_repository_manifest(&manifest, repository)?;
        write_game_state(
            &GameState {
                current: Some(manifest_id),
            },
            repository,
            &definition.name,
        )?;
        write_synced_manifest(&manifest)?;
        match old_state.current {
            Some(old_id) => {
                repository.remove(&RelativePath::new(&definition.name).join(old_id.to_string()))?
            }
            None => {}
        }
    }
    Ok(())
}

fn sync_game_from_repository(
    manifest: &GameSaveManifest,
    repository_save: &ResolvedSave,
    repository: &impl Repository,
    args: &SyncArgs,
) -> Result<()> {
    let game = &manifest.definition.name;
    for path in &manifest.definition.paths {
        let path = rewrite_path(&path.path)?;
        if repository_save.files.contains_key(&path) {
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
    let save_path = RelativePath::new(&manifest.definition.name).join(manifest.id.to_string());
    for real_path in repository_save.directories.keys() {
        if args.dry_run {
            println!("- Creating save directory at {}", real_path.display());
        } else {
            fs::create_dir_all(&real_path)
                .with_context(|| format!("failed to sync {} from repository", &game))?;
        }
    }
    for (real_path, (path, file, metadata)) in &repository_save.files {
        if args.dry_run {
            println!("- Applying save file at {}", real_path.display())
        } else {
            let repository_path = save_path.join(make_path_safe(path)).join(file);
            repository
                .download_file(&repository_path, &real_path)
                .with_context(|| format!("failed to sync {} from repository", &game))?;
            let local_file = std::fs::File::open(real_path)
                .with_context(|| format!("failed to open {}", real_path.display()))?;
            local_file
                .set_modified(metadata.modified.into())
                .with_context(|| {
                    format!("failed to set modified time on {}", real_path.display())
                })?;
            let local_metadata = real_path.metadata()?;
            if local_metadata.len() != metadata.size {
                return Err(Error::msg(format!(
                    "downloaded file {} is {} bytes but should be {} bytes",
                    real_path.display(),
                    local_metadata.len(),
                    metadata.size
                )))
                .with_context(|| format!("failed to sync {} from the repository", &game));
            }
            let local_modified = time::UtcDateTime::from(local_metadata.modified()?);
            if local_modified != metadata.modified {
                return Err(Error::msg(format!(
                    "downloaded file {} has mod time {} but should be {}",
                    real_path.display(),
                    local_modified,
                    metadata.modified
                )))
                .with_context(|| format!("failed to sync {} from the repository", &game));
            }
        }
    }
    if !args.dry_run {
        write_synced_manifest(&GameSaveManifest {
            id: manifest.id,
            definition: manifest.definition.clone(),
            timestamp: time::OffsetDateTime::now_local()?,
            directories: manifest.directories.clone(),
            files: manifest.files.clone(),
        })?;
    }
    Ok(())
}

struct ResolvedSave {
    /// A folder listing from either the local system or a GameSaveManifest
    ///
    /// Each entry's key is a full path mapped to the current system
    ///
    /// Each value is a GameDefinitionPath and the relative path from it to the directory
    directories: BTreeMap<PathBuf, (String, RelativePathBuf)>,

    /// A file listing from either the local system or a GameSaveManifest
    ///
    /// Each entry's key is a full path mapped to the current system
    ///
    /// Each value is a GameDefinitionPath and the relative path from it to the file
    files: BTreeMap<PathBuf, (String, RelativePathBuf, GameSaveFileMetadata)>,
}

fn saves_are_equal(left: &ResolvedSave, right: &ResolvedSave) -> bool {
    if left.directories.len() != right.directories.len() {
        return false;
    }
    for path in left.directories.keys() {
        if !right.directories.contains_key(path) {
            return false;
        }
    }
    if left.files.len() != right.files.len() {
        return false;
    }
    for (path, (_, _, left_metadata)) in &left.files {
        match right.files.get(path) {
            Some((_, _, right_metadata)) if left_metadata == right_metadata => {}
            _ => return false,
        }
    }
    true
}

fn get_local_save(definition: &GameDefinition) -> Result<ResolvedSave> {
    let mut seen = HashSet::new();
    let mut directories = BTreeMap::new();
    let mut files = BTreeMap::new();
    for definition_path in &definition.paths {
        let root_path = paths::rewrite_path(&definition_path.path)?;
        let mut queue = vec![root_path.clone()];
        while let Some(path) = queue.pop() {
            if seen.contains(&path) {
                continue;
            }
            seen.insert(path.clone());
            let relative_path = path.relative_to(&root_path)?;
            if path.is_dir() {
                for item in path.read_dir()? {
                    queue.push(item?.path());
                }
                directories.insert(path, (definition_path.path.clone(), relative_path));
            } else if path.is_file() {
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
    Ok(ResolvedSave { directories, files })
}

fn get_manifest_save(manifest: &GameSaveManifest) -> Result<ResolvedSave> {
    let mut directories = BTreeMap::new();
    for (definition_path, path_directories) in &manifest.directories {
        let resolved_path = paths::rewrite_path(&definition_path)?;
        for directory_path in path_directories {
            directories.insert(
                directory_path.to_path(&resolved_path),
                (definition_path.clone(), directory_path.clone()),
            );
        }
    }
    let mut files = BTreeMap::new();
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
    Ok(ResolvedSave { directories, files })
}
