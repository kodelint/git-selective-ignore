use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSettings {
    pub backup_strategy: BackupStrategy,
    pub auto_cleanup: bool,
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BackupStrategy {
    Memory,
    TempFile,
    GitStash,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectiveIgnoreConfig {
    pub version: String,
    pub files: HashMap<String, Vec<IgnorePattern>>,
    pub global_settings: GlobalSettings,
}

impl Default for SelectiveIgnoreConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            files: HashMap::new(),
            global_settings: GlobalSettings {
                backup_strategy: BackupStrategy::TempFile,
                auto_cleanup: true,
                verbose: false,
            },
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    repo_root: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let repo_root = find_git_root()?;
        let config_path = repo_root.join(".git").join("selective-ignore.toml");

        Ok(Self {
            config_path,
            repo_root,
        })
    }

    pub fn initialize(&self) -> Result<()> {
        if self.config_path.exists() {
            return Ok(());
        }

        let default_config = SelectiveIgnoreConfig::default();
        self.save_config(&default_config)?;
        Ok(())
    }

    pub fn get_repo_root(&self) -> &Path {
        &self.repo_root
    }
}

pub trait ConfigProvider {
    fn load_config(&self) -> Result<SelectiveIgnoreConfig>;
    fn save_config(&self, config: &SelectiveIgnoreConfig) -> Result<()>;
    fn get_config_path(&self) -> Result<PathBuf>;
}

impl ConfigProvider for ConfigManager {
    fn load_config(&self) -> Result<SelectiveIgnoreConfig> {
        if !self.config_path.exists() {
            return Ok(SelectiveIgnoreConfig::default());
        }

        let content =
            fs::read_to_string(&self.config_path).context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }

    fn save_config(&self, config: &SelectiveIgnoreConfig) -> Result<()> {
        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(&self.config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    fn get_config_path(&self) -> Result<PathBuf> {
        Ok(self.config_path.clone())
    }
}

fn find_git_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let mut dir = current_dir.as_path();

    loop {
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => anyhow::bail!("Not in a Git repository"),
        }
    }
}
