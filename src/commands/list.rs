use anyhow::{Context, Result};

use crate::{
    games::{definition::list_definitions, manifest::read_synced_manifest},
    repository::get_repository,
    utils::config,
};

pub fn list() -> Result<()> {
    let config = config::load().with_context(|| "failed to load config")?;
    let repository = get_repository(&config.repository)?;
    let mut games = list_definitions(&repository)?;
    games.sort();
    for game in games {
        let synced_manifest = read_synced_manifest(&game)?;
        match synced_manifest {
            Some(_) => println!("{} (linked)", game),
            None => println!("{}", game),
        }
    }
    Ok(())
}
