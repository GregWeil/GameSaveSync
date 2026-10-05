use std::{assert_eq, matches};

use crate::games::{GamePlatform, definition::GameDefinitionPath};

use super::*;

use anyhow::Result;
use time::{Date, Month};

#[test]
fn test_determine_sync_direction_empty_repository() -> Result<()> {
    let definition = make_definition()?;
    let files = make_default_save()?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(&definition, &files, &None, &None, &None, &args)?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::ToRepository)
    );
    Ok(())
}

#[test]
fn test_determine_sync_direction_not_yet_synced() -> Result<()> {
    let definition = make_definition()?;
    let local_date = Date::from_calendar_date(2026, Month::August, 2)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, local_date)?;
    let repository_date = Date::from_calendar_date(2026, Month::August, 3)?
        .midnight()
        .as_utc();
    let repository_files = make_save("game.sav", 123, repository_date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &None,
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert!(matches!(
        result,
        SyncDetermination::Conflict {
            message: _,
            local_last_mod: _,
            repository_synced: _
        }
    ));
    Ok(())
}

#[test]
fn test_determine_sync_direction_nothing_changed() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert_eq!(result, SyncDetermination::InSync);
    Ok(())
}

#[test]
fn test_determine_sync_direction_local_changed() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let local_date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, local_date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::ToRepository)
    );
    Ok(())
}

#[test]
fn test_determine_sync_direction_repository_changed() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let repository_date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, repository_date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::FromRepository)
    );
    Ok(())
}

#[test]
fn test_determine_sync_direction_both_changed() -> Result<()> {
    let definition = make_definition()?;
    let synced_date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, date)?;
    let synced_files = make_save("game.sav", 123, synced_date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert!(matches!(
        result,
        SyncDetermination::Conflict {
            message: _,
            local_last_mod: _,
            repository_synced: _
        }
    ));
    Ok(())
}

#[test]
fn test_determine_sync_direction_failed_apply() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let local_date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, local_date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let pending_action = PendingSyncAction::Apply {
        start_time: OffsetDateTime::now_utc(),
        manifest_id: Uuid::new_v4(),
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &Some(pending_action),
        &args,
    )?;
    assert!(matches!(
        result,
        SyncDetermination::Conflict {
            message: _,
            local_last_mod: _,
            repository_synced: _
        }
    ));
    Ok(())
}

#[test]
fn test_determine_sync_direction_force_store() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let repository_date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, repository_date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: true,
        force_apply: false,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::ToRepository)
    );
    Ok(())
}

#[test]
fn test_determine_sync_direction_force_apply() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let local_date = Date::from_calendar_date(2026, Month::August, 5)?
        .midnight()
        .as_utc();
    let local_files = make_save("game.sav", 123, local_date)?;
    let synced_files = make_save("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_save("game.sav", 123, date)?;
    let repository_manifest = make_manifest(&definition, &repository_files)?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: true,
    };
    let result = determine_sync_direction(
        &definition,
        &local_files,
        &Some((synced_files, synced_manifest)),
        &Some((repository_files, repository_manifest)),
        &None,
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::FromRepository)
    );
    Ok(())
}

#[test]
fn test_saves_are_equal() -> Result<()> {
    let truly_empty_save = ResolvedSave {
        directories: BTreeMap::new(),
        files: BTreeMap::new(),
    };
    assert!(
        saves_are_equal(&truly_empty_save, &truly_empty_save),
        "no directories should match"
    );
    assert!(
        !saves_are_equal(&make_empty_save()?, &truly_empty_save),
        "directory vs no directory should not match"
    );
    assert!(
        !saves_are_equal(&truly_empty_save, &make_empty_save()?),
        "no directory vs directory should not match"
    );
    assert!(
        saves_are_equal(&make_empty_save()?, &make_empty_save()?),
        "no files should match"
    );
    assert!(
        saves_are_equal(&make_default_save()?, &make_default_save()?),
        "same file should match"
    );
    assert!(
        !saves_are_equal(&make_default_save()?, &make_empty_save()?),
        "file vs no file should not match"
    );
    assert!(
        !saves_are_equal(&make_empty_save()?, &make_default_save()?),
        "no file vs file should not match"
    );
    let time = UtcDateTime::now();
    assert!(
        !saves_are_equal(
            &make_save("game1.sav", 123, time)?,
            &make_save("game2.sav", 123, time)?
        ),
        "different paths should not match"
    );
    assert!(
        !saves_are_equal(
            &make_save("game.sav", 123, time)?,
            &make_save("game.sav", 234, time)?
        ),
        "different sizes should not match"
    );
    assert!(
        !saves_are_equal(
            &make_save("game.sav", 123, time)?,
            &make_save("game.sav", 123, UtcDateTime::now())?
        ),
        "different times should not match"
    );
    Ok(())
}

fn make_definition() -> Result<GameDefinition> {
    Ok(GameDefinition {
        name: "Game".into(),
        platform: GamePlatform::Linux,
        paths: vec![GameDefinitionPath {
            path: "~/game".into(),
        }],
        steam_app_id: None,
    })
}

fn make_empty_save() -> Result<ResolvedSave> {
    let home = std::env::home_dir().ok_or(Error::msg("failed to get home dir"))?;
    let directories = BTreeMap::from([(
        home.join("game"),
        ("~/game".to_owned(), RelativePathBuf::new()),
    )]);
    let files = BTreeMap::new();
    Ok(ResolvedSave { directories, files })
}

fn make_default_save() -> Result<ResolvedSave> {
    let date = Date::from_calendar_date(2026, Month::July, 27)?
        .midnight()
        .as_utc();
    make_save("game.sav", 123, date)
}

fn make_save(path: &str, size: u64, modified: UtcDateTime) -> Result<ResolvedSave> {
    let home = std::env::home_dir().ok_or(Error::msg("failed to get home dir"))?;
    let directories = BTreeMap::from([(
        home.join("game"),
        ("~/game".to_owned(), RelativePathBuf::new()),
    )]);
    let files = BTreeMap::from([(
        home.join("game").join(path),
        (
            "~/game".to_owned(),
            RelativePathBuf::from(path),
            GameSaveFileMetadata { size, modified },
        ),
    )]);
    Ok(ResolvedSave { directories, files })
}

fn make_manifest(definition: &GameDefinition, save: &ResolvedSave) -> Result<GameSaveManifest> {
    let mut manifest_directories = HashMap::new();
    for (path, directory) in save.directories.values() {
        manifest_directories
            .entry(path.clone())
            .or_insert_with(|| HashSet::new())
            .insert(directory.clone());
    }
    let mut manifest_files = HashMap::new();
    for (path, file, metadata) in save.files.values() {
        manifest_files
            .entry(path.clone())
            .or_insert_with(|| HashMap::new())
            .insert(file.clone(), metadata.clone());
    }
    Ok(GameSaveManifest {
        id: Uuid::new_v4(),
        definition: definition.clone(),
        timestamp: OffsetDateTime::now_local()?,
        directories: manifest_directories,
        files: manifest_files,
    })
}
