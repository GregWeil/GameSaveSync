use anyhow::{Context, Result};
use clap::Args;

use crate::{
    games::{self, definition},
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
            println!("\t{}", path.path);
            match games::paths::rewrite_path(&path.path) {
                Ok(rewritten) => println!("\t\t➙ {}", rewritten.display()),
                Err(error) => println!("\t\t× {}", error),
            }
        }
    }
    Ok(())
}
