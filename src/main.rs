/// The Big IDEA:
/// Idea is very simple, why can't I tell git to ignore
/// some specific lines from my code base.
/// Gitignore ignores the whole file but that not what I need
///  it to ignore any line, block of code, line numbers
/// etc. I don't want to accidentally commit something in the
/// Git History which is not suppose to be there.
/// I am also not interested to keep track of what all the changes
/// I did to test it locally and now when I am ready to push
/// I have to remove those not-required-in-git history stuff
use anyhow::{Result};
use clap::{Parser, Subcommand};
mod core;
mod utils;

#[derive(Parser)]
#[command(name = "git-selective-ignore")]
#[command(about = "A Git plugin to selectively ignore lines during commits")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize selective ignore for this repository
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => utils::initialize_repository(),
    }
}
