use anyhow::Result;
use clap::Parser;

mod commands;
mod games;
mod repository;
mod utils;

use commands::Commands;

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
        Commands::List => commands::list::list(),
    }
}
