use std::{env::current_dir, path::PathBuf};

use anyhow::{Context, Result};
use clap::Args;

use crate::{repository, utils::config};

#[derive(Args, Debug)]
#[command(about = "Set the shared location where saves are stored")]
pub struct SetRepositoryArgs {
    #[arg(help = "The path to the repository")]
    path: PathBuf,
}

pub fn set_repository(args: &SetRepositoryArgs) -> Result<()> {
    let path = current_dir()
        .with_context(|| "failed to get working directory")?
        .join(args.path.clone());
    let mut config = config::load().with_context(|| "failed to load config")?;
    let new_repository = repository::AnyRepositoryConfig::Local(
        repository::local_repository::LocalRepositoryConfig { path },
    );
    match config.repository {
        Some(ref repository) => {
            println!(
                "Changing repository from {} to {}",
                repository, new_repository
            );
        }
        None => {
            println!("Setting repository to {}", new_repository);
        }
    }
    config.repository = Some(new_repository);
    repository::prepare_repository(&config.repository)?;
    config::save(&config).with_context(|| "failed to save config")?;
    Ok(())
}
