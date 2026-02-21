use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Error, Result};
use clap::Args;
use relative_path::{PathExt, RelativePath, RelativePathBuf};
use uuid::Uuid;

use crate::{
    games::{
        definition::{GameDefinition, list_definitions, load_definition},
        manifest::{
            GameSaveFileMetadata, GameSaveManifest, read_repository_manifest, read_synced_manifest,
            write_repository_manifest, write_synced_manifest,
        },
        paths,
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
    Conflict,
    NoChange,
}

fn sync_game(game: &str, repository: &impl Repository, args: &SyncArgs) -> Result<()> {
    println!("Checking {}", game);
    let definition = load_definition(repository, game)?;
    let local_files = get_local_files(&definition)?;
    let repository_manifest = read_repository_manifest(repository, game)?;
    let synced_manifest = read_synced_manifest(game)?;
    let sync_direction = match &repository_manifest {
        Some(repository_manifest) => match &synced_manifest {
            Some(synced_manifest) => {
                let synced_files = get_manifest_files(synced_manifest)?;
                let repository_files = get_manifest_files(repository_manifest)?;
                let local_changed = !save_files_equal(&local_files, &synced_files);
                let repository_changed = !save_files_equal(&synced_files, &repository_files);
                match (local_changed, repository_changed) {
                    (true, true) => SyncDirection::Conflict,
                    (true, false) => SyncDirection::ToRepository,
                    (false, true) => SyncDirection::FromRepository,
                    (false, false) => SyncDirection::NoChange,
                }
            }
            // No local manifest
            None => SyncDirection::Conflict,
        },
        // No repository manifest
        None => SyncDirection::ToRepository,
    };
    match sync_direction {
        SyncDirection::Conflict => println!("- Conflict detected, skipping {game}"),
        SyncDirection::ToRepository => {
            println!("- Storing save in repository");
            sync_game_to_repository(&definition, &local_files, repository, args)?;
        }
        SyncDirection::FromRepository => {
            println!("- Applying save from repository");
            sync_game_from_repository()?;
        }
        SyncDirection::NoChange => println!("- No changes detected"),
    }
    Ok(())
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

fn sync_game_from_repository() -> Result<()> {
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
