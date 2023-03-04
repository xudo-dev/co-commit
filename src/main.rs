mod args;
mod commit;

use args::CommandArgs;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = CommandArgs::parse();
    if let Err(e) = commit::generate_commit(&args).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
   }
}
