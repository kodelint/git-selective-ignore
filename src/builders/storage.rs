use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// `BackupData` is a struct used to store all the necessary information
/// for restoring a file to its original state after a commit.
///
/// This data structure is serialized and saved by the `StorageProvider`
/// implementations.
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    /// The original content of the file before any ignored lines were removed.
    pub original_content: String,
    /// A map of the ignored lines, where the key is the zero-based line index
    /// and the value is the content of the ignored line.
    pub ignored_lines: HashMap<usize, String>,
    /// A hash of the original file content. This is used in the `post-commit`
    /// hook to verify that the file has not been modified since the backup was created.
    pub original_file_hash: String,
    /// A hash of the cleaned file content (the version without ignored lines).
    /// This is used in the `post-commit` hook to ensure the backup is
    /// being restored to the correct file state.
    pub cleaned_file_hash: String,
}

/// The `StorageProvider` trait defines the public interface for handling
/// temporary backups of file content.
///
/// This abstraction allows the application to use different storage mechanisms
/// (e.g., in-memory or on-disk) without changing the core `IgnoreEngine` logic.
pub trait StorageProvider {

    /// Stores the provided `BackupData` for a given file.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file being backed up.
    /// * `backup_data`: The `BackupData` struct containing the content to save.
    ///
    /// # Returns
    /// `Result<()>`: An empty result if the operation was successful.
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()>;

    /// Restores the backup data for a given file.
    ///
    /// The implementation should also handle cleanup of the stored backup data.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file to restore.
    ///
    /// # Returns
    /// `Result<Option<BackupData>>`: An `Option` containing the backup data if it exists,
    /// or `None` if no backup was found.
    fn restore_backup(&mut self, file_path: &str) -> Result<Option<BackupData>>;

    /// Returns all the file paths that currently have backup data stored.
    ///
    /// This is used during post-commit processing to identify all files that
    /// were modified during pre-commit, especially when "all" patterns are used.
    ///
    /// # Returns
    /// `Result<Vec<String>>`: A vector of file paths that have stored backups.
    fn get_all_backup_keys(&self) -> Result<Vec<String>>;

    /// Cleans up all stored backup data.
    ///
    /// This is typically called after the post-commit hook has run to clear
    /// any remaining temporary files or memory.
    fn cleanup(&mut self) -> Result<()>;
}

/// `TempFileStorage` is an implementation of `StorageProvider` that uses
/// the filesystem to store backups.
///
/// It creates a temporary directory inside the `.git` folder of the repository
/// and saves each file's backup data as a separate JSON file.
pub struct TempFileStorage {
    /// The path to the temporary directory where backups are stored.
    temp_dir: PathBuf,
}

impl TempFileStorage {
    /// Constructs a new `TempFileStorage` instance.
    ///
    /// This method creates the backup directory if it doesn't already exist.
    ///
    /// # Arguments
    /// * `repo_path`: The path to the root of the Git repository.
    ///
    /// # Returns
    /// `Result<Self>`: A new `TempFileStorage` instance.
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let temp_dir = repo_path.join("selective-ignore-backups");
        if !temp_dir.exists() {
            fs::create_dir(&temp_dir).context("Failed to create backup directory")?;
        }
        Ok(Self { temp_dir })
    }

    /// A private helper function to get the full path for a backup file.
    ///
    /// It sanitizes the file path to create a safe filename by replacing
    /// path separators with underscores and appending a `.backup` extension.
    ///
    /// # Arguments
    /// * `file_path`: The path of the file to back up.
    ///
    /// # Returns
    /// `PathBuf`: The full path to the backup file.
    fn get_backup_path(&self, file_path: &str) -> PathBuf {
        let safe_filename = file_path.replace(['/', '\\'], "_");
        self.temp_dir.join(format!("{safe_filename}.backup"))
    }

    /// A private helper function to reverse the filename sanitization.
    ///
    /// Takes a backup filename and converts it back to the original file path.
    ///
    /// # Arguments
    /// * `backup_filename`: The sanitized backup filename.
    ///
    /// # Returns
    /// `String`: The original file path.
    fn restore_file_path_from_backup_name(&self, backup_filename: &str) -> String {
        // Remove the .backup extension and restore path separators
        backup_filename
            .strip_suffix(".backup")
            .unwrap_or(backup_filename)
            .replace('_', "/")
    }
}

/// Implementation of the `StorageProvider` trait for `TempFileStorage`.
impl StorageProvider for TempFileStorage {

    /// Stores the `BackupData` by serializing it to JSON and writing it to a file.
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()> {
        let backup_path = self.get_backup_path(file_path);
        let serialized = serde_json::to_string_pretty(&backup_data).context("Failed to serialize backup data")?;
        fs::write(&backup_path, serialized).context("Failed to write backup file")?;
        Ok(())
    }

    /// Restores a backup by reading its file, deserializing the JSON, and then
    /// removing the backup file.
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

    /// Returns all file paths that have backup files in the temp directory.
    fn get_all_backup_keys(&self) -> Result<Vec<String>> {
        // This vector will store the decoded file paths from the backup file names.
        let mut keys = Vec::new();

        if self.temp_dir.exists() {
            let entries = fs::read_dir(&self.temp_dir).context("Failed to read backup directory")?;
            // Read all the entries (files and directories) in the temporary backup directory.
            // The `?` operator handles any potential I/O errors and returns early.
            for entry in entries {
                let entry = entry.context("Failed to read directory entry")?;
                let filename = entry.file_name().to_string_lossy().to_string();

                // Only process files with .backup extension
                if filename.ends_with(".backup") {
                    let original_path = self.restore_file_path_from_backup_name(&filename);
                    keys.push(original_path);
                }
            }
        }

        Ok(keys)
    }

    /// Cleans up the entire temporary backup directory.
    fn cleanup(&mut self) -> Result<()> {
        if self.temp_dir.exists() {
            fs::remove_dir_all(&self.temp_dir).context("Failed to remove temp directory during cleanup")?;
        }
        Ok(())
    }
}

/// `MemoryStorage` is an implementation of `StorageProvider` that keeps
/// all backup data in a `HashMap` in memory.
///
/// This is a non-persistent storage method, meaning all backups are lost
/// when the application process ends.
pub struct MemoryStorage {
    /// A `HashMap` where the key is the file path and the value is the `BackupData`.
    backups: HashMap<String, BackupData>,
}

impl MemoryStorage {
    /// Constructs a new `MemoryStorage` instance.
    pub fn new() -> Self {
        Self {
            backups: HashMap::new(),
        }
    }
}

impl StorageProvider for MemoryStorage {

    /// Implementation of the `StorageProvider` trait for `MemoryStorage`.
    fn store_backup(&mut self, file_path: &str, backup_data: BackupData) -> Result<()> {
        self.backups.insert(file_path.to_string(), backup_data);
        Ok(())
    }

    /// Restores a backup by removing the data from the `HashMap` and returning it.
    fn restore_backup(&mut self, file_path: &str) -> Result<Option<BackupData>> {
        // `HashMap::remove` returns `Some(value)` if the key existed, otherwise `None`.
        // This fits the method's return type perfectly.
        Ok(self.backups.remove(file_path))
    }

    /// Returns all the file paths (keys) that currently have backup data stored.
    fn get_all_backup_keys(&self) -> Result<Vec<String>> {
        Ok(self.backups.keys().cloned().collect())
    }

    /// Clears the `HashMap`, effectively removing all backups from memory.
    fn cleanup(&mut self) -> Result<()> {
        self.backups.clear();
        Ok(())
    }
}