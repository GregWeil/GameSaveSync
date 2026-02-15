use clap::Subcommand;

pub mod list;
pub mod set_repository;

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Set the shared location where saves are stored")]
    SetRepository(set_repository::SetRepositoryArgs),
    #[command(about = "List all games registered in the repository")]
    List,
}
