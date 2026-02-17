use anyhow::{Context, Result};

use crate::{
    repository::{Repository, get_repository},
    utils::config,
};

pub fn list() -> Result<()> {
    let config = config::load().with_context(|| "failed to load config")?;
    let repository = get_repository(&config.repository)?;
    for path in repository.read_dir("")? {
        let path = path?;
        if !repository.is_dir(&path)? || !repository.is_file(path.join("definition.toml"))? {
            continue;
        }
        println!("{}", path);
    }
    Ok(())
}
