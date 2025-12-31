use anyhow::Result;
use clap::{Parser, Subcommand};

// Import sibling modules. `mod` statements in `main.rs` link the
// modules defined in `src/` to the main crate.
mod builders;
mod core;
mod utils;
use crate::core::{config::ConfigManager, version::run};
// Import all public functions from the `utils` module. These functions
// are the core logic handlers for each command-line action.
use crate::utils::{
    add_ignore_pattern, export_patterns, import_patterns, install_hooks, list_patterns,
    process_post_commit, process_pre_commit, remove_ignore_pattern, show_status, uninstall_hooks,
    verify_staging_area,
};

/// `Cli` is the main struct that represents the command-line interface.
///
/// It uses the `clap` crate's `Parser` derive macro to automatically
/// generate a command-line argument parser. This struct serves as the
/// top-level container for all subcommands and global arguments.
#[derive(Parser)]
#[command(name = "git-selective-ignore")]
#[command(about = "A Git plugin to selectively ignore lines during commits")]
struct Cli {
    /// The `Commands` enum defines the available subcommands. `clap` will
    /// automatically match the first positional argument to a variant of this enum.
    #[command(subcommand)]
    command: Commands,
}

/// The `Commands` enum defines the available subcommands for the CLI.
///
/// Each variant corresponds to a specific action a user can perform. The doc
/// comments on each variant are used by `clap` to generate the help text
/// for each subcommand.
#[derive(Subcommand)]
enum Commands {
    /// Initializes the selective ignore configuration for a new repository.
    ///
    /// This command creates the necessary `.git-selective-ignore` configuration
    /// file in the repository's root.
    Init,

    /// Adds a new ignore pattern for a specified file.
    ///
    /// The `file_path`, `pattern_type`, and `pattern` arguments are required
    /// to define a new rule.
    Add {
        /// The path to the file to which the pattern should be applied, relative
        /// to the repository root.
        file_path: Option<String>,
        /// The type of pattern to use, such as `line-regex`, `line-number`, etc.
        #[arg(short, long)]
        pattern_type: Option<String>,
        /// The specific pattern string (e.g., a regex, a line number, or a block marker).
        pattern: Option<String>,
    },

    /// Removes an existing ignore pattern from a file's configuration.
    ///
    /// Patterns are identified by their unique ID, which can be found using the `list` command.
    Remove {
        //// The path to the file from which the pattern should be removed.
        file_path: String,
        /// The unique ID of the pattern to remove.
        pattern_id: String,
    },

    /// Lists all configured selective ignore patterns for all files.
    ///
    /// This command provides a summary of all rules, including the file they apply to
    /// and their unique IDs.
    List,

    /// Processes files before a commit is made. This is intended for use by a Git hook.
    ///
    /// This command is invoked by the `pre-commit` Git hook to clean staged files.
    PreCommit {
        /// If true, simulate the process without modifying any files.
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Restores files after a commit has been completed. This is intended for use by a Git hook.
    ///
    /// This command is invoked by the `post-commit` Git hook to restore the original
    /// file content that was backed up during the `pre-commit` stage.
    PostCommit,

    /// Installs the `pre-commit` and `post-commit` Git hooks.
    ///
    /// This command sets up the necessary shell scripts in the `.git/hooks` directory
    /// to automate the selective ignore process on every commit.
    InstallHooks,

    /// Uninstalls the previously installed Git hooks.
    ///
    /// This command removes the `pre-commit` and `post-commit` hook scripts.
    UninstallHooks,

    /// Displays the status of all configured files and their ignored content.
    ///
    /// This command provides a report showing which files have ignored lines and how many.
    Status,

    /// Verifies that the staged content does not contain any ignored patterns.
    ///
    /// This command acts as a stricter version of `pre-commit` that fails the commit
    /// if ignored content is found, rather than automatically cleaning it.
    Verify,

    /// Imports patterns from an external file into the configuration.
    ///
    /// This is useful for migrating patterns from tools like `.gitignore` or for
    /// sharing configurations.
    Import {
        /// The path to the file containing the patterns to import.
        file_path: String,
        /// The format of the import file (`gitignore` or `custom`).
        #[arg(short, long, default_value = "custom")]
        import_type: String,
    },

    /// Exports the current configuration's patterns to a file.
    ///
    /// This command saves the selective ignore rules in a specified format.
    Export {
        /// The path where the exported file should be saved.
        file_path: String,
        /// The desired output format (`toml`, `json`, or `yaml`).
        #[arg(short, long, default_value = "toml")]
        format: String,
    },
    /// Show the version of the tool
    Version,
}

/// The main entry point of the application.
///
/// This function is responsible for:
/// 1. Parsing command-line arguments using `clap::Parser`.
/// 2. Performing a pre-flight configuration validation for most commands.
/// 3. Matching the user's command to the appropriate logic handler function.
fn main() -> Result<()> {
    // Parse the command-line arguments provided by the user.
    let cli = Cli::parse();

    // Perform a configuration validation check for most commands.
    // The `Init` and `InstallHooks` commands are excluded because they
    // are often run before a valid configuration exists.
    if !matches!(
        cli.command,
        Commands::Init | Commands::InstallHooks | Commands::Version
    ) {
        let config_manager = ConfigManager::new()?;
        config_manager.validate_config()?;
    }

    // A `match` statement is used to dispatch the parsed command to the
    // correct function. Each arm calls a specific function from the `utils`
    // module to handle the command's logic.
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
        Commands::PreCommit { dry_run } => process_pre_commit(dry_run),
        Commands::PostCommit => process_post_commit(),
        Commands::InstallHooks => install_hooks(),
        Commands::UninstallHooks => uninstall_hooks(),
        Commands::Status => show_status(),
        Commands::Verify => verify_staging_area(),
        Commands::Import {
            file_path,
            import_type,
        } => import_patterns(file_path, import_type),
        Commands::Export { file_path, format } => export_patterns(file_path, format),
        Commands::Version => {
            run();
            Ok(())
        }
    }
}
