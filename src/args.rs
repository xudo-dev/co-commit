use clap::Parser;
use clap_verbosity_flag::{InfoLevel, Verbosity};

#[derive(Parser)]
pub struct CommandArgs {
    #[clap(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[arg(
        long = "dry-run",
        help = "Output the generated message, but don't create a commit."
    )]
    pub dry_run: bool,

    #[arg(
        short,
        long,
        help = "Edit the generated commit message before committing."
    )]
    pub review: bool,

    #[arg(short, long, help = "Don't ask for confirmation before committing.")]
    pub force: bool,
}