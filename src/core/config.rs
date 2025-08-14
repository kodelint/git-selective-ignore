use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use crate::builders::importer::{FileImporter, PatternImporter};
use crate::builders::patterns::IgnorePattern;
use crate::builders::validator::{ConfigValidator, StandardValidator};

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

    pub fn validate_config(&self) -> Result<()> {
        let config = self.load_config()?;
        let validator = StandardValidator::new();
        let issues = validator.validate_config(&config)?;

        if issues.is_empty() {
            println!("✓ Configuration is valid.");
            Ok(())
        } else {
            println!("⚠️  Found issues in configuration:");
            for issue in issues {
                println!("  - {issue}");
            }
            anyhow::bail!("Configuration validation failed.");
        }
    }

    pub fn add_pattern(
        &mut self,
        file_path: String,
        pattern_type: String,
        pattern_spec: String,
    ) -> Result<()> {
        let mut config = self.load_config()?;
        let ignore_pattern = IgnorePattern::new(pattern_type, pattern_spec)?;

        config
            .files
            .entry(file_path)
            .or_insert_with(Vec::new)
            .push(ignore_pattern);

        self.save_config(&config)?;
        Ok(())
    }

    pub fn remove_pattern(&mut self, file_path: String, pattern_id: String) -> Result<()> {
        let mut config = self.load_config()?;

        if let Some(patterns) = config.files.get_mut(&file_path) {
            patterns.retain(|p| p.id != pattern_id);
            if patterns.is_empty() {
                config.files.remove(&file_path);
            }
        }

        self.save_config(&config)?;
        Ok(())
    }

    pub fn list_patterns(&self) -> Result<()> {
        let config = self.load_config()?;

        if config.files.is_empty() {
            println!("No ignore patterns configured.");
            return Ok(());
        }

        for (file_path, patterns) in &config.files {
            println!("\n📁 File: {file_path}");
            for pattern in patterns {
                println!(
                    "  🔍 ID: {} | Type: {:?} | Pattern: {}",
                    pattern.id, pattern.pattern_type, pattern.specification
                );
            }
        }
        Ok(())
    }

    pub fn import_patterns(&mut self, file_path: String, import_type: String) -> Result<()> {
        let mut importer = FileImporter::new();
        let patterns = importer.import_from_file(&file_path, &import_type)?;

        let mut config = self.load_config()?;
        for (file, pattern_list) in patterns {
            config
                .files
                .entry(file)
                .or_insert_with(Vec::new)
                .extend(pattern_list);
        }

        self.save_config(&config)?;
        Ok(())
    }

    pub fn export_patterns(&self, file_path: &str, format: String) -> Result<()> {
        let config = self.load_config()?;

        let content = match format.as_str() {
            "json" => {
                serde_json::to_string_pretty(&config).context("Failed to serialize to JSON")?
            }
            "yaml" => serde_yaml::to_string(&config).context("Failed to serialize to YAML")?,
            "toml" | _ => toml::to_string_pretty(&config).context("Failed to serialize to TOML")?,
        };

        std::fs::write(file_path, content).context("Failed to write export file")?;

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
