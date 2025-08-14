use crate::core::config::ConfigManager;
use anyhow::Result;
use crate::builders::hooks;
use crate::core::engine::IgnoreEngine;

/// Initializes the selective ignore configuration for a new repository.
///
/// This function creates the necessary configuration files and directory structure
/// within the current Git repository. It's the first command a user should run
/// to set up the tool.
pub fn initialize_repository() -> Result<()> {
    // Create a new instance of the ConfigManager.
    let config_manager = ConfigManager::new()?;
    // Call the initialize method to create the config file.
    config_manager.initialize()?;
    println!("✓ Initialized selective ignore for this repository");
    println!("Run 'git-selective-ignore install-hooks' to enable automatic processing");
    Ok(())
}

/// Adds a new ignore pattern to a specified file's configuration.
///
/// This function takes the file path, the pattern type (e.g., `line-regex`),
/// and the pattern specification string. It updates the configuration file
/// to include the new pattern.
///
/// # Arguments
/// * `file_path`: The path to the file to which the pattern should be applied.
/// * `pattern_type`: A string representing the type of pattern (e.g., "line-regex").
/// * `pattern`: The actual pattern string (e.g., a regular expression).
pub fn add_ignore_pattern(file_path: String, pattern_type: String, pattern: String) -> Result<()> {
    // Get a ConfigManager instance using a helper function.
    let mut config_manager = get_config_manager()?;
    // Call the ConfigManager's method to add the new pattern.
    config_manager.add_pattern(file_path, pattern_type, pattern)?;
    println!("✓ Added ignore pattern");
    Ok(())
}

/// Removes a specific ignore pattern from a file's configuration.
///
/// This function requires a pattern's unique ID to remove it, ensuring that the
/// correct pattern is targeted.
///
/// # Arguments
/// * `file_path`: The path to the file from which the pattern should be removed.
/// * `pattern_id`: The unique ID of the pattern to remove.
pub fn remove_ignore_pattern(file_path: String, pattern_id: String) -> Result<()> {
    let mut config_manager = get_config_manager()?;
    config_manager.remove_pattern(file_path, pattern_id)?;
    println!("✓ Removed ignore pattern");
    Ok(())
}

/// Lists all configured selective ignore patterns.
///
/// This function provides a summary of all patterns defined in the configuration,
/// grouped by file, which is useful for auditing and managing the settings.
pub fn list_patterns() -> Result<()> {
    let config_manager = get_config_manager()?;
    config_manager.list_patterns()?;
    Ok(())
}

/// Executes the pre-commit processing logic.
///
/// This function is intended to be called by the `pre-commit` Git hook. It
/// initializes the `IgnoreEngine`, which then finds staged files, applies
/// ignore patterns, backs up the original content, and re-stages the cleaned content.
pub fn process_pre_commit() -> Result<()> {
    let mut engine = get_engine()?;
    engine.process_pre_commit()?;
    Ok(())
}

/// Executes the post-commit processing logic.
///
/// This function is intended to be called by the `post-commit` Git hook. It
/// initializes the `IgnoreEngine`, which then restores the original file content
/// from the temporary backups created during the pre-commit phase.
pub fn process_post_commit() -> Result<()> {
    let mut engine = get_engine()?;
    engine.process_post_commit()?;
    Ok(())
}

/// Installs the necessary Git hooks (`pre-commit` and `post-commit`) into the
/// local repository.
///
/// This enables the selective ignore functionality to run automatically on every
/// commit, without manual intervention.
pub fn install_hooks() -> Result<()> {
    let config_manager = get_config_manager()?;
    hooks::install_git_hooks(&config_manager.get_repo_root())?;
    println!("✓ Installed Git hooks for automatic processing");
    Ok(())
}

/// Uninstalls the previously installed Git hooks.
///
/// This disables the automatic selective ignore processing, allowing the user
/// to revert to standard Git behavior.
pub fn uninstall_hooks() -> Result<()> {
    let config_manager = get_config_manager()?;
    hooks::uninstall_git_hooks(&config_manager.get_repo_root())?;
    println!("✓ Uninstalled Git hooks");
    Ok(())
}

/// A private helper function to create and return an `IgnoreEngine` instance.
///
/// This function encapsulates the logic of initializing the `ConfigManager`
/// and passing it to the `IgnoreEngine::new` constructor. This avoids
/// code duplication in the public functions.
fn get_engine() -> Result<IgnoreEngine> {
    let config_manager = ConfigManager::new()?;
    IgnoreEngine::new(config_manager)
}

/// A private helper function to create a `ConfigManager` instance.
///
/// This is a utility function to simplify the creation of a `ConfigManager`
/// instance, used by several public functions.
fn get_config_manager() -> Result<ConfigManager> {
    ConfigManager::new()
}