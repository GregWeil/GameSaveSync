use std::{assert_eq, matches};

use crate::games::{GamePlatform, definition::GameDefinitionPath};

use super::*;

use anyhow::Result;
use time::{Date, Month};

#[test]
fn test_determine_sync_direction_empty_repository() -> Result<()> {
    let definition = make_definition()?;
    let files = make_default_files()?;
    let args = SyncArgs {
        game: None,
        dry_run: true,
        force_store: false,
        force_apply: false,
    };
    let result = determine_sync_direction(&definition, &files, &None, &None, &args)?;
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
    let local_files = make_files("game.sav", 123, local_date)?;
    let repository_date = Date::from_calendar_date(2026, Month::August, 3)?
        .midnight()
        .as_utc();
    let repository_files = make_files("game.sav", 123, repository_date)?;
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
        &args,
    )?;
    assert!(matches!(result, SyncDetermination::Conflict(_, _, _)));
    Ok(())
}

#[test]
fn test_determine_sync_direction_nothing_changed() -> Result<()> {
    let definition = make_definition()?;
    let date = Date::from_calendar_date(2026, Month::August, 4)?
        .midnight()
        .as_utc();
    let local_files = make_files("game.sav", 123, date)?;
    let synced_files = make_files("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, date)?;
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
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::DoNothing)
    );
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
    let local_files = make_files("game.sav", 123, local_date)?;
    let synced_files = make_files("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, date)?;
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
    let local_files = make_files("game.sav", 123, date)?;
    let synced_files = make_files("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, repository_date)?;
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
    let local_files = make_files("game.sav", 123, date)?;
    let synced_files = make_files("game.sav", 123, synced_date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, date)?;
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
        &args,
    )?;
    assert!(matches!(result, SyncDetermination::Conflict(_, _, _)));
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
    let local_files = make_files("game.sav", 123, date)?;
    let synced_files = make_files("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, repository_date)?;
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
    let local_files = make_files("game.sav", 123, local_date)?;
    let synced_files = make_files("game.sav", 123, date)?;
    let synced_manifest = make_manifest(&definition, &synced_files)?;
    let repository_files = make_files("game.sav", 123, date)?;
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
        &args,
    )?;
    assert_eq!(
        result,
        SyncDetermination::Automatic(SyncDirection::FromRepository)
    );
    Ok(())
}

#[test]
fn test_save_files_equal() -> Result<()> {
    assert!(
        save_files_equal(&HashMap::new(), &HashMap::new()),
        "no files should match"
    );
    assert!(
        save_files_equal(&make_default_files()?, &make_default_files()?),
        "same file should match"
    );
    assert!(
        !save_files_equal(&make_default_files()?, &HashMap::new()),
        "file vs no file should not match"
    );
    assert!(
        !save_files_equal(&HashMap::new(), &make_default_files()?),
        "no file vs file should not match"
    );
    let time = UtcDateTime::now();
    assert!(
        !save_files_equal(
            &make_files("game1.sav", 123, time)?,
            &make_files("game2.sav", 123, time)?
        ),
        "different paths should not match"
    );
    assert!(
        !save_files_equal(
            &make_files("game.sav", 123, time)?,
            &make_files("game.sav", 234, time)?
        ),
        "different sizes should not match"
    );
    assert!(
        !save_files_equal(
            &make_files("game.sav", 123, time)?,
            &make_files("game.sav", 123, UtcDateTime::now())?
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

fn make_default_files() -> Result<ResolvedSaveFiles> {
    let date = Date::from_calendar_date(2026, Month::July, 27)?
        .midnight()
        .as_utc();
    make_files("game.sav", 123, date)
}

fn make_files(path: &str, size: u64, modified: UtcDateTime) -> Result<ResolvedSaveFiles> {
    let home = std::env::home_dir().ok_or(Error::msg("failed to get home dir"))?;
    Ok(HashMap::from([(
        home.join("game").join(path),
        (
            "~/game".to_owned(),
            RelativePathBuf::from(path),
            GameSaveFileMetadata { size, modified },
        ),
    )]))
}

fn make_manifest(
    definition: &GameDefinition,
    files: &ResolvedSaveFiles,
) -> Result<GameSaveManifest> {
    let mut manifest_files = HashMap::new();
    for (path, file, metadata) in files.values() {
        manifest_files
            .entry(path.clone())
            .or_insert_with(|| HashMap::new())
            .insert(file.clone(), metadata.clone());
    }
    Ok(GameSaveManifest {
        id: Uuid::new_v4(),
        definition: definition.clone(),
        timestamp: OffsetDateTime::now_local()?,
        files: manifest_files,
    })
}
