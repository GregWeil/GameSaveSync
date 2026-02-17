use anyhow::Result;
use clap::Subcommand;

pub mod list;
pub mod set_repository;
pub mod show;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Set the shared location where saves are stored")]
    SetRepository(set_repository::SetRepositoryArgs),
    #[command(about = "List all games registered in the repository")]
    List,
    #[command(about = "Show the definition of a registered game")]
    Show(show::ShowArgs),
}

pub fn exec(command: &Commands) -> Result<()> {
    match command {
        Commands::SetRepository(args) => set_repository::set_repository(args),
        Commands::List => list::list(),
        Commands::Show(args) => show::show(args),
    }
}
