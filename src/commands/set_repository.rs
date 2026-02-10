use clap::Args;

#[derive(Args, Debug)]
#[command(about = "Set the shared location where saves are stored")]
pub struct SetRepositoryArgs {
    #[arg(help = "The path to the repository")]
    path: String,
}

pub fn set_repository(args: SetRepositoryArgs) {
    println!("{:?}", args)
}
