use clap::{Parser, Subcommand};

mod commands;

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::SetRepository(args) => commands::set_repository::set_repository(args),
    }
}
