use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod utils;

#[derive(Subcommand, Debug)]
enum Commands {
    SetRepository(commands::set_repository::SetRepositoryArgs),
}

#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::SetRepository(ref args) => commands::set_repository::set_repository(args),
    }
}
