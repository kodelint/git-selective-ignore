use crate::core::config::ConfigManager;
use anyhow::Result;
use crate::builders::hooks;

pub fn initialize_repository() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    config_manager.initialize()?;
    println!("✓ Initialized selective ignore for this repository");
    println!("Run 'git-selective-ignore install-hooks' to enable automatic processing");
    Ok(())
}

pub fn install_hooks() -> Result<()> {
    let config_manager = get_config_manager()?;
    hooks::install_git_hooks(&config_manager.get_repo_root())?;
    println!("✓ Installed Git hooks for automatic processing");
    Ok(())
}

// Helper function to create ConfigManager instance
fn get_config_manager() -> Result<ConfigManager> {
    ConfigManager::new()
}