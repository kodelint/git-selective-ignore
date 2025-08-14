use anyhow::{Result};
use clap::{Parser, Subcommand};
use crate::core::config::ConfigManager;
use crate::utils::{add_ignore_pattern, install_hooks, list_patterns, process_post_commit, process_pre_commit, remove_ignore_pattern, show_status, uninstall_hooks};

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
    /// Add a new ignore pattern for a file
    Add {
        /// File path relative to repository root
        file_path: String,
        /// Pattern type (line-regex, line-number, block-start-end)
        #[arg(short, long, default_value = "line-regex")]
        pattern_type: String,
        /// Pattern specification
        pattern: String,
    },
    /// Remove an ignore pattern
    Remove {
        /// File path
        file_path: String,
        /// Pattern ID to remove
        pattern_id: String,
    },
    /// List all configured ignore patterns
    List,
    /// Process files before commit (used by git hooks)
    PreCommit,
    /// Restore files after commit (used by git hooks)
    PostCommit,
    /// Install git hooks for automatic processing
    InstallHooks,
    /// Uninstall git hooks
    UninstallHooks,
    /// Check status of ignored lines
    Status,
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
        Commands::Add {
            file_path,
            pattern_type,
            pattern,
        } => add_ignore_pattern(file_path, pattern_type, pattern),
        Commands::Remove {
            file_path,
            pattern_id,
        } => remove_ignore_pattern(file_path, pattern_id),
        Commands::List => list_patterns(),
        Commands::PreCommit => process_pre_commit(),
        Commands::PostCommit => process_post_commit(),
        Commands::InstallHooks => install_hooks(),
        Commands::UninstallHooks => uninstall_hooks(),
        Commands::Status => show_status(),
    }
}
