use crate::builders::storage::{MemoryStorage, StorageProvider, TempFileStorage};
use crate::core::config::{BackupStrategy, ConfigManager, ConfigProvider};
use anyhow::Result;
use git2::Repository;

pub struct IgnoreEngine {
    config_manager: ConfigManager,
    storage: Box<dyn StorageProvider>,
    repo: Repository,
}

impl IgnoreEngine {
    pub fn new(config_manager: ConfigManager) -> Result<Self> {
        let repo = Repository::open(config_manager.get_repo_root())?;

        // Choose storage strategy based on config
        let config = config_manager.load_config()?;
        let storage: Box<dyn StorageProvider> = match config.global_settings.backup_strategy {
            BackupStrategy::Memory => Box::new(MemoryStorage::new()),
            BackupStrategy::TempFile => Box::new(TempFileStorage::new(repo.path().to_path_buf())?),
            BackupStrategy::GitStash => {
                // For now, fallback to TempFile. GitStash implementation would be more complex
                Box::new(TempFileStorage::new(repo.path().to_path_buf())?)
            }
        };

        Ok(Self {
            config_manager,
            storage,
            repo,
        })
    }
}