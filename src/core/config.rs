use crate::builders::importer::{FileImporter, PatternImporter};
use crate::builders::patterns::IgnorePattern;
use crate::builders::validator::{ConfigValidator, StandardValidator};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// `GlobalSettings` holds application-wide configuration options.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalSettings {
    /// The strategy to use for backing up original file content before a commit.
    pub backup_strategy: BackupStrategy,
    /// A flag to determine if temporary backups should be automatically cleaned up
    /// after a successful commit.
    pub auto_cleanup: bool,
    /// A flag to enable verbose logging for more detailed output.
    pub verbose: bool,
    /// A flag to enable humorous output messages.
    #[serde(default)]
    pub funny_mode: bool,
}

/// An enum defining the different backup strategies.
///
/// This allows the tool to be flexible in how it handles backups, with options
/// for in-memory, temporary files, or a planned Git stash-based approach.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BackupStrategy {
    /// Stores backup data only in memory, which is not persistent across restarts.
    Memory,
    /// Stores backup data in temporary files within the `.git` directory.
    TempFile,
    /// A planned strategy to use `git stash` for backups, which is not yet implemented.
    GitStash,
}

/// `SelectiveIgnoreConfig` is the main struct that represents the entire
/// configuration for the selective ignore tool.
///
/// This struct is directly serialized to and deserialized from the `selective-ignore.toml`
/// configuration file.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectiveIgnoreConfig {
    /// The version of the configuration file format. Used for future-proofing and
    /// handling backward compatibility.
    pub version: String,
    /// A map where each key is a file path and the value is a vector of `IgnorePattern`s
    /// to apply to that file.
    pub files: HashMap<String, Vec<IgnorePattern>>,
    /// Global settings that affect the overall behavior of the tool.
    pub global_settings: GlobalSettings,
}

/// The default implementation for `SelectiveIgnoreConfig`.
///
/// This provides a sensible starting point for a new configuration file.
impl Default for SelectiveIgnoreConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            files: HashMap::new(),
            global_settings: GlobalSettings {
                // `TempFile` is chosen as the default for its persistence and reliability.
                backup_strategy: BackupStrategy::TempFile,
                // `auto_cleanup` is enabled by default to prevent accumulation of temporary files.
                auto_cleanup: true,
                // `verbose` is disabled by default for cleaner output.
                verbose: false,
                // `funny_mode` is disabled by default.
                funny_mode: false,
            },
        }
    }
}

/// `ConfigManager` is a concrete implementation of `ConfigProvider`.
///
/// It handles the primary operations for managing the configuration file, including
/// loading, saving, and validating, as well as modifying individual patterns.
pub struct ConfigManager {
    /// The full path to the configuration file (`.git/selective-ignore.toml`)
    config_path: PathBuf,
    /// The full path to the global configuration file (`~/.config/git-selective-ignore/config.toml`)
    global_config_path: PathBuf,
    /// The root directory of the Git repository.
    repo_root: PathBuf,
}

impl ConfigManager {
    /// Creates a new `ConfigManager` instance.
    ///
    /// This is the entry point for accessing the configuration. It first
    /// locates the root of the Git repository and then determines the path
    /// for the configuration file.
    pub fn new() -> Result<Self> {
        let repo_root = find_git_root()?;
        let config_path = repo_root.join(".git").join("selective-ignore.toml");

        let global_config_path = if let Some(home) = dirs::home_dir() {
            home.join(".config")
                .join("git-selective-ignore")
                .join("config.toml")
        } else {
            // Fallback if home dir cannot be determined
            PathBuf::from("/tmp/git-selective-ignore-global.toml")
        };

        Ok(Self {
            config_path,
            global_config_path,
            repo_root,
        })
    }

    /// Initializes a new configuration file with default settings if one does not already exist.
    pub fn initialize(&self) -> Result<()> {
        if self.config_path.exists() {
            return Ok(());
        }

        let default_config = SelectiveIgnoreConfig::default();
        self.save_config(&default_config)?;
        Ok(())
    }

    /// Initializes the global configuration file.
    pub fn initialize_global(&self) -> Result<()> {
        if self.global_config_path.exists() {
            return Ok(());
        }

        if let Some(parent) = self.global_config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let default_config = SelectiveIgnoreConfig::default();
        let content =
            toml::to_string_pretty(&default_config).context("Failed to serialize global config")?;
        fs::write(&self.global_config_path, content)
            .context("Failed to write global config file")?;
        Ok(())
    }

    /// Validates the entire configuration file using a `StandardValidator`.
    ///
    /// This function reads the configuration, passes it to the validator,
    /// and then prints any issues found. It will return an error if validation fails.
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

    /// Adds a new ignore pattern to a specified file.
    ///
    /// This function loads the existing configuration, creates a new `IgnorePattern`,
    /// and adds it to the list of patterns for the given file path before saving.
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

    /// Removes an ignore pattern using its unique ID.
    ///
    /// It loads the configuration, finds the pattern with the matching ID, removes it,
    /// and if the file's pattern list becomes empty, it removes the file entry from the map.
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

    /// Prints a list of all configured patterns to the console.
    ///
    /// This is the main function for the `list` command.
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

    /// Imports patterns from an external file into the configuration.
    ///
    /// It uses a `FileImporter` to parse the external file and then merges the
    /// resulting patterns into the current configuration.
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

    /// Exports the current configuration to an external file.
    ///
    /// The output format can be specified as `json`, `yaml`, or `toml`.
    pub fn export_patterns(&self, file_path: &str, format: String) -> Result<()> {
        let config = self.load_config()?;

        let content = match format.as_str() {
            "json" => {
                serde_json::to_string_pretty(&config).context("Failed to serialize to JSON")?
            }
            "yaml" => serde_yaml::to_string(&config).context("Failed to serialize to YAML")?,
            _ => toml::to_string_pretty(&config).context("Failed to serialize to TOML")?,
        };

        std::fs::write(file_path, content).context("Failed to write export file")?;

        Ok(())
    }

    /// Returns a reference to the Git repository's root path.
    pub fn get_repo_root(&self) -> &Path {
        &self.repo_root
    }
}

/// The `ConfigProvider` trait defines the core interface for interacting with the
/// configuration.
///
/// This trait allows for different implementations of config providers, making it
/// possible to load configurations from sources other than a local file (e.g., a database).
pub trait ConfigProvider {
    /// Loads the configuration from the provider's source.
    fn load_config(&self) -> Result<SelectiveIgnoreConfig>;
    /// Saves the provided configuration to the provider's source.
    fn save_config(&self, config: &SelectiveIgnoreConfig) -> Result<()>;
}

/// Implementation of the `ConfigProvider` trait for `ConfigManager`.
///
/// This section provides the concrete implementations of the trait methods,
/// which handle the actual file I/O operations.
impl ConfigProvider for ConfigManager {
    /// Loads the configuration from the file. If the file doesn't exist, it returns
    /// a default configuration instead of an error.
    fn load_config(&self) -> Result<SelectiveIgnoreConfig> {
        let mut final_config = SelectiveIgnoreConfig::default();

        // 1. Load Global Config if it exists
        if self.global_config_path.exists() {
            let content = fs::read_to_string(&self.global_config_path)
                .context("Failed to read global config file")?;
            let global_config: SelectiveIgnoreConfig =
                toml::from_str(&content).context("Failed to parse global config file")?;

            final_config.global_settings = global_config.global_settings;
            // Merge global files patterns
            for (file, patterns) in global_config.files {
                final_config.files.entry(file).or_default().extend(patterns);
            }
        }

        // 2. Load Local Config if it exists
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)
                .context("Failed to read local config file")?;
            let local_config: SelectiveIgnoreConfig =
                toml::from_str(&content).context("Failed to parse local config file")?;

            // Local settings override global settings
            final_config.global_settings = local_config.global_settings;

            // Merge local files patterns
            for (file, patterns) in local_config.files {
                final_config.files.entry(file).or_default().extend(patterns);
            }
        }

        Ok(final_config)
    }

    /// Saves the provided configuration struct to the file.
    fn save_config(&self, config: &SelectiveIgnoreConfig) -> Result<()> {
        let content = toml::to_string_pretty(config).context("Failed to serialize config")?;

        fs::write(&self.config_path, content).context("Failed to write config file")?;

        Ok(())
    }
}

/// A private helper function to find the root directory of the current Git repository.
///
/// It walks up the directory tree from the current working directory until it
/// finds a directory containing a `.git` folder.
fn find_git_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    let mut dir = current_dir.as_path();

    loop {
        // Check if the current directory contains a `.git` folder.
        if dir.join(".git").exists() {
            return Ok(dir.to_path_buf());
        }

        // Move up to the parent directory.
        match dir.parent() {
            Some(parent) => dir = parent,
            // If there's no parent, we've reached the root of the filesystem
            // and the `.git` folder was not found.
            None => anyhow::bail!("Not in a Git repository"),
        }
    }
}
