use anyhow::{Context, Result};
use clap::Args;

use crate::{
    games::{
        definition,
        local_state::{print_pending_action_warning, read_local_game_state},
        manifest::read_repository_manifest,
        paths,
        repository_state::read_repository_game_state,
    },
    repository::get_repository,
    utils::config,
};

#[derive(Args, Debug)]
pub struct ShowArgs {
    #[arg(help = "The game to show")]
    game: String,
}

pub fn show(args: &ShowArgs) -> Result<()> {
    let config = config::load().with_context(|| "failed to load config")?;
    let repository = get_repository(&config.repository)?;
    let definition = definition::load_definition(&repository, &args.game)?;
    println!("Name: {}", &definition.name);
    println!("Platform: {}", &definition.platform);
    if definition.paths.is_empty() {
        println!("Save Paths: None");
    } else {
        println!("Save Paths:");
        for path in definition.paths {
            match paths::rewrite_path(&path.path) {
                Ok(rewritten) => println!("\t{} ➙ {}", path.path, rewritten.display()),
                Err(error) => println!("\t{} × {}", path.path, error),
            }
        }
    }
    println!("Status:");
    let repository_state = read_repository_game_state(&repository, &definition.name)?;
    let repository_manifest = match &repository_state.current {
        Some(manifest_id) => read_repository_manifest(&repository, &definition.name, manifest_id)?,
        None => None,
    };
    match &repository_manifest {
        Some(repository_manifest) => println!(
            "\tRepository last modified: {}",
            repository_manifest.timestamp
        ),
        None => println!("\tRepository last modified: No save in repository"),
    }
    let local_state = read_local_game_state(&definition.name)?;
    match &local_state.last_synced {
        Some(timestamp) => println!("\tLast synced on this device: {timestamp}"),
        None => println!("\tLast synced on this device: Never"),
    }
    match &local_state.pending_action {
        Some(_) => {
            println!("Warning:");
            print_pending_action_warning(&local_state.pending_action, "\t");
        }
        _ => {}
    }
    Ok(())
}
