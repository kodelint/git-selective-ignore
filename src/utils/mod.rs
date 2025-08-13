use crate::core::config::ConfigManager;
use anyhow::Result;

pub fn initialize_repository() -> Result<()> {
    let config_manager = ConfigManager::new()?;
    config_manager.initialize()?;
    println!("✓ Initialized selective ignore for this repository");
    println!("Run 'git-selective-ignore install-hooks' to enable automatic processing");
    Ok(())
}
