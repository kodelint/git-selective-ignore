use anyhow::{Result};
use clap::{Parser, Subcommand};
use crate::core::config::ConfigManager;
use crate::utils::{install_hooks, uninstall_hooks};

mod core;
mod utils;
mod builders;

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
    /// Install git hooks for automatic processing
    InstallHooks,
    /// Uninstall git hooks
    UninstallHooks,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Validate config for all commands except `init` and `install-hooks`
    if !matches!(cli.command, Commands::Init | Commands::InstallHooks) {
        let config_manager = ConfigManager::new()?;
        println!("Initializing...");
    }

    match cli.command {
        Commands::Init => utils::initialize_repository(),
        Commands::InstallHooks => install_hooks(),
        Commands::UninstallHooks => uninstall_hooks(),
    }
}
