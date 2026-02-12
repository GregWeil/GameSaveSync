use std::{
    env::current_dir,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, Error, Result};
use clap::Args;

use crate::utils::config::{self, RepositoryConfig};

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
    match config.repository {
        Some(ref repository) => {
            println!(
                "Changing repository from {} to {}",
                repository.path.display(),
                path.display()
            );
        }
        None => {
            println!("Setting repository to {}", path.display());
        }
    }
    if !path.is_absolute() {
        return Result::Err(Error::msg(format!(
            "Path {} is not absolute",
            path.display()
        )));
    }
    if !path.exists() {
        return Result::Err(Error::msg(format!(
            "Path {} does not exist",
            path.display()
        )));
    }
    if !path.is_dir() {
        return Result::Err(Error::msg(format!(
            "Path {} is not a directory",
            path.display()
        )));
    }
    let file = path.join("GameSaveSync.toml");
    if !file.exists() || !file.is_file() {
        if path.read_dir()?.next().is_some() {
            return Result::Err(Error::msg(format!("Path {} is not empty", path.display())));
        }
        File::create(file).with_context(|| "failed to create repository")?;
    }
    config.repository = Some(RepositoryConfig { path });
    verify_repository(&config.repository)?;
    config::save(&config).with_context(|| "failed to save config")?;
    Ok(())
}

fn verify_repository(config: &Option<RepositoryConfig>) -> Result<()> {
    let config = config.as_ref().ok_or(Error::msg("Repository is not set"))?;
    if !config.path.is_absolute() {
        return Result::Err(Error::msg(format!(
            "Path {} is not absolute",
            config.path.display()
        )));
    }
    if !config.path.exists() {
        return Result::Err(Error::msg(format!(
            "Path {} does not exist",
            config.path.display()
        )));
    }
    if !config.path.is_dir() {
        return Result::Err(Error::msg(format!(
            "Path {} is not a directory",
            config.path.display()
        )));
    }
    if !config.path.join("GameSaveSync.toml").is_file() {
        return Result::Err(Error::msg(format!(
            "Path {} has not been initialized as a repository",
            config.path.display()
        )));
    }
    Ok(())
}
