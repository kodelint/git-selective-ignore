use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    pub original_content: String,
    pub ignored_lines: HashMap<usize, String>,
    pub original_file_hash: String,
    pub cleaned_file_hash: String,
}

pub trait StorageProvider {
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()>;
    fn restore_backup(&mut self, file_path: &str) -> Result<Option<BackupData>>;
    fn cleanup(&mut self) -> Result<()>;
}

pub struct TempFileStorage {
    temp_dir: PathBuf,
}

impl TempFileStorage {
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let temp_dir = repo_path.join("selective-ignore-backups");
        if !temp_dir.exists() {
            fs::create_dir(&temp_dir).context("Failed to create backup directory")?;
        }
        Ok(Self { temp_dir })
    }

    fn get_backup_path(&self, file_path: &str) -> PathBuf {
        let safe_filename = file_path.replace(['/', '\\'], "_");
        self.temp_dir.join(format!("{safe_filename}.backup"))
    }
}

impl StorageProvider for TempFileStorage {
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()> {
        let backup_path = self.get_backup_path(file_path);
        let serialized = serde_json::to_string_pretty(&backup_data).context("Failed to serialize backup data")?;
        fs::write(&backup_path, serialized).context("Failed to write backup file")?;
        Ok(())
    }

    fn restore_backup(&mut self, file_path: &str) -> Result<Option<BackupData>> {
        let backup_path = self.get_backup_path(file_path);

        if backup_path.exists() {
            let content = fs::read_to_string(&backup_path).context("Failed to read backup file")?;
            let backup_data: BackupData = serde_json::from_str(&content).context("Failed to deserialize backup data")?;

            // Clean up the backup file after restoring it
            fs::remove_file(&backup_path).context("Failed to remove backup file after restore")?;

            return Ok(Some(backup_data));
        }

        Ok(None)
    }

    fn cleanup(&mut self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).context("Failed to remove temp directory during cleanup")?;
        }
        Ok(())
    }
}
pub struct MemoryStorage {
    backups: HashMap<String, BackupData>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            backups: HashMap::new(),
        }
    }
}

impl StorageProvider for MemoryStorage {
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()> {
        self.backups.insert(file_path.to_string(), backup_data);
        Ok(())
    }

    fn restore_backup(&mut self, file_path: &str) -> Result<Option<BackupData>> {
        Ok(self.backups.remove(file_path))
    }

    fn cleanup(&mut self) -> Result<()> {
        self.backups.clear();
        Ok(())
    }
}