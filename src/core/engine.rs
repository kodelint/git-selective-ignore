use anyhow::{anyhow, Result};
use git2::{DiffOptions, Index, Repository};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::builders::storage::{BackupData, MemoryStorage, StorageProvider, TempFileStorage};
use crate::builders::patterns::{IgnorePattern, PatternMatcher, PatternType};
use crate::builders::reporter::{ConsoleReporter, FileStatus, StatusReporter};
use crate::core::config::{BackupStrategy, ConfigManager, ConfigProvider};

/// The `IgnoreEngine` is the central component responsible for managing the selective
/// ignore process within a Git repository. It acts as the orchestrator for the
/// `pre-commit` and `post-commit` hooks, coordinating file analysis, content modification,
/// and backup/restore operations.
///
/// This struct holds key state, including the Git repository itself, a configuration manager
/// to load settings, and a storage provider to handle temporary backups of modified files.
pub struct IgnoreEngine {
    config_manager: ConfigManager,
    storage: Box<dyn StorageProvider>,
    repo: Repository,
}

impl IgnoreEngine {
    /// Constructs a new `IgnoreEngine` instance.
    ///
    /// This is the entry point for initializing the engine. It performs two key tasks:
    /// 1. Opens the Git repository from the root path specified in the configuration.
    /// 2. Initializes the correct `StorageProvider` based on the `backup_strategy`
    ///    defined in the configuration. This allows for flexible backup methods
    ///    (in-memory, temporary files, or a future Git stash implementation).
    ///
    /// # Arguments
    /// * `config_manager`: A `ConfigManager` instance that provides access to the
    ///   application's configuration, including the repository path and backup strategy.
    ///
    /// # Returns
    /// A `Result<Self>` which is the new `IgnoreEngine` instance on success,
    /// or an error if the repository cannot be opened or storage cannot be initialized.
    pub fn new(config_manager: ConfigManager) -> Result<Self> {
        // Attempt to open the Git repository. The path is retrieved from the config.
        // `git2::Repository::open` can fail if the path is not a valid Git repository.
        let repo = Repository::open(config_manager.get_repo_root())?;

        // Load the configuration to determine the backup strategy.
        let config = config_manager.load_config()?;
        let storage: Box<dyn StorageProvider> = match config.global_settings.backup_strategy {
            // `MemoryStorage` is a simple, non-persistent storage option.
            BackupStrategy::Memory => Box::new(MemoryStorage::new()),
            // `TempFileStorage` uses the filesystem for backups, providing persistence
            // across separate process runs. The temporary files are stored within
            // the `.git` directory to avoid being accidentally committed.
            BackupStrategy::TempFile => Box::new(TempFileStorage::new(repo.path().to_path_buf())?),
            // `GitStash` is a planned but not yet implemented feature. The current
            // implementation falls back to `TempFileStorage`. This design pattern
            // makes it easy to swap in the new implementation later.
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
    /// The main entry point for the `pre-commit` Git hook.
    ///
    /// This function orchestrates the process of modifying staged files. It identifies
    /// staged files that have ignore patterns configured, removes the ignored content,
    /// backs up the original content, writes the cleaned content to the working directory,
    /// and finally, re-stages the modified files.
    ///
    /// The re-staging step is critical: the `pre-commit` hook changes the working
    /// directory file, so it must be added back to the index to include the
    /// "cleaned" version in the commit.
    pub fn process_pre_commit(&mut self) -> Result<()> {
        println!("📝 Processing files with selective ignore patterns...");
        let config = self.config_manager.load_config()?;
        let mut index = self.repo.index()?;

        // Get a list of files that are currently staged.
        let staged_files = self.get_staged_files(&mut index)?;
        let mut files_to_add_after_processing = Vec::new();

        for file_path in staged_files.iter() {
            let file_path_str = file_path.to_string_lossy().to_string();

            if let Some(patterns) = config.files.get(&file_path_str) {
                // Read content from index to get the staged version of the file
                let entry = index.get_path(file_path, 0).ok_or_else(|| {
                    anyhow!(
                        "Failed to get staged file entry for {}",
                        file_path.display()
                    )
                })?;
                let blob = self.repo.find_blob(entry.id)?;
                let original_content = std::str::from_utf8(blob.content())?;

                let (cleaned_content, ignored_lines) =
                    self.process_file_content(original_content, patterns)?;

                if cleaned_content != original_content {
                    // Create a backup of the original staged file content
                    let backup_data = BackupData {
                        original_content: original_content.to_string(),
                        ignored_lines,
                        original_file_hash: calculate_hash(original_content),
                        cleaned_file_hash: calculate_hash(&cleaned_content),
                    };
                    self.storage.store_backup(&file_path_str, backup_data)?;

                    // Write the cleaned content to the working directory.
                    fs::write(file_path, cleaned_content)?;

                    // Mark the file to be re-staged.
                    files_to_add_after_processing.push(file_path.clone());
                }
            }
        }

        // Re-add any files that were modified by the hook
        if !files_to_add_after_processing.is_empty() {
            println!("🔄 Re-staging modified files...");
            for path in files_to_add_after_processing {
                index.add_path(&path)?;
            }
            index.write()?;
        }

        println!("✅ Pre-commit processing complete.");
        Ok(())
    }

    /// The main entry point for the `post-commit` Git hook.
    ///
    /// This function's primary purpose is to restore the original file content from the
    /// backups created during the `pre-commit` hook. It iterates through all configured
    /// files, checks for a valid backup, and verifies the file's state before restoring
    /// the original content. This prevents data loss if a user has modified the file
    /// between the pre-commit and post-commit phases.
    pub fn process_post_commit(&mut self) -> Result<()> {
        println!("🔄 Restoring files after commit...");
        let config = self.config_manager.load_config()?;

        // Iterate through all files that have ignore patterns in the configuration
        for file_path in config.files.keys() {
            let path = Path::new(file_path);

            if let Some(backup_data) = self.storage.restore_backup(file_path)? {
                // Check if the working directory file matches the cleaned version we committed
                let current_content = fs::read_to_string(path)?;
                if calculate_hash(&current_content) == backup_data.cleaned_file_hash {
                    // Restore the original content to the working directory
                    fs::write(path, &backup_data.original_content)?;
                    println!("✓ Restored {file_path}");
                } else {
                    println!(
                        "⚠️ Skipping restore for {file_path} - file was modified after pre-commit"
                    );
                }
            }
        }

        // Cleanup backups if configured
        if config.global_settings.auto_cleanup {
            self.storage.cleanup()?;
        }

        println!("✅ Post-commit processing complete.");
        Ok(())
    }

    /// Generates and displays a status report for all configured files.
    ///
    /// This function is intended to be a user-facing command that provides insight
    /// into the state of the files managed by the selective ignore tool. It checks
    /// for file existence, total lines, and how many lines would be ignored based
    /// on the current configuration.
    pub fn show_status(&mut self) -> Result<()> {
        let config = self.config_manager.load_config()?;
        let mut file_statuses = HashMap::new();
        let reporter = ConsoleReporter::new();

        for (file_path, patterns) in &config.files {
            let path = Path::new(file_path);
            let mut status = FileStatus {
                exists: path.exists(),
                has_ignored_lines: false,
                ignored_line_count: 0,
                total_lines: 0,
            };

            if status.exists {
                let content = fs::read_to_string(path)?;
                status.total_lines = content.lines().count();
                let (_, ignored_lines) = self.process_file_content(&content, patterns)?;
                if !ignored_lines.is_empty() {
                    status.has_ignored_lines = true;
                    status.ignored_line_count = ignored_lines.len();
                }
            }
            file_statuses.insert(file_path.clone(), status);
        }

        reporter.generate_status_report(&config, file_statuses)?;
        Ok(())
    }

    /// Verifies that no ignored content is present in the Git staging area.
    ///
    /// This is a strict verification function, often used in a separate `verify-staged`
    /// hook. Unlike `process_pre_commit` which modifies files, this function will
    /// fail the operation and return an error if it detects any configured ignored
    /// content in the staged files, preventing the commit from proceeding.
    pub fn verify_staging(&mut self) -> Result<()> {
        println!("🕵️ Verifying staging area for ignored content...");
        let config = self.config_manager.load_config()?;
        let mut index = self.repo.index()?;

        let staged_files = self.get_staged_files(&mut index)?;
        let mut violations = Vec::new();

        for file_path in staged_files {
            let file_path_str = file_path.to_string_lossy().to_string();

            if let Some(patterns) = config.files.get(&file_path_str) {
                // Read content from index to get the staged version
                let entry = index.get_path(&file_path, 0).ok_or_else(|| {
                    anyhow!(
                        "Failed to get staged file entry for {}",
                        file_path.display()
                    )
                })?;
                let blob = self.repo.find_blob(entry.id)?;
                let content = std::str::from_utf8(blob.content())?;

                // Check for ignored patterns in the staged content
                for pattern in patterns {
                    let (_, ignored_lines) =
                        self.process_file_content(content, &vec![pattern.clone()])?;
                    if !ignored_lines.is_empty() {
                        violations.push(format!(
                            "  - In file {}: pattern '{}' is present.",
                            file_path.display(),
                            pattern.specification
                        ));
                    }
                }
            }
        }

        if !violations.is_empty() {
            println!("⚠️ Found ignored content in staging area:");
            for violation in violations {
                println!("{violation}");
            }
            anyhow::bail!("Verification failed - ignored content detected");
        }

        println!("✓ Staging area verification passed");
        Ok(())
    }

    /// Internal helper function to determine which files are currently staged.
    ///
    /// This is a critical function that needs to be robust for various Git repository states.
    /// It uses a multipronged approach to find staged files:
    /// 1. The primary method is `diff_tree_to_index` which compares the `HEAD` commit
    ///    with the current index. This works for all repositories with at least one commit.
    /// 2. If no `HEAD` exists (e.g., an empty repository before the first commit), it
    ///    diffs against an empty tree.
    /// 3. As a fallback, it iterates through all entries in the index directly. This
    ///    is a less efficient but reliable method for covering all edge cases.
    fn get_staged_files(&self, index: &mut Index) -> Result<Vec<std::path::PathBuf>> {
        println!("DEBUG: Getting staged files...");
        let mut staged_files = Vec::new();
        let mut options = DiffOptions::new();

        // Method 1: Try diff from HEAD to index (for existing repo with commits)
        if let Ok(head) = self.repo.head() {
            if let Ok(head_tree) = head.peel_to_tree() {
                println!("DEBUG: Using HEAD to index diff");
                let diff = self.repo.diff_tree_to_index(
                    Some(&head_tree),
                    Some(index),
                    Some(&mut options),
                )?;

                println!("DEBUG: Found {} deltas in diff", diff.deltas().len());
                // Iterate through the diff deltas to find new and modified files.
                for delta in diff.deltas() {
                    if let Some(path) = delta.new_file().path() {
                        println!("DEBUG: Staged file from diff: {}", path.display());
                        staged_files.push(path.to_path_buf());
                    }
                }
            }
        } else {
            // Method 2: For the initial commit, diff against an empty tree.
            println!("DEBUG: No HEAD found (initial commit), using diff from empty tree");
            // For initial commit, diff against empty tree
            let empty_tree = self.repo.treebuilder(None)?.write()?;
            let empty_tree_obj = self.repo.find_tree(empty_tree)?;
            let diff = self.repo.diff_tree_to_index(
                Some(&empty_tree_obj),
                Some(index),
                Some(&mut options),
            )?;

            println!(
                "DEBUG: Found {} deltas in empty tree diff",
                diff.deltas().len()
            );
            for delta in diff.deltas() {
                if let Some(path) = delta.new_file().path() {
                    println!(
                        "DEBUG: Staged file from empty tree diff: {}",
                        path.display()
                    );
                    staged_files.push(path.to_path_buf());
                }
            }
        }

        // Method 3: Fallback. Directly iterate through the index if the diff methods
        // didn't find any staged files. This handles cases where a file might be
        // staged but not yet committed, and no HEAD exists.
        if staged_files.is_empty() {
            println!("DEBUG: No files from diff, checking index entries directly");
            let entry_count = index.len();
            println!("DEBUG: Index has {entry_count} entries");

            for i in 0..entry_count {
                if let Some(entry) = index.get(i) {
                    let path = std::path::PathBuf::from(std::str::from_utf8(&*entry.path)?);
                    println!("DEBUG: Index entry: {}", path.display());
                    staged_files.push(path);
                }
            }
        }

        println!("DEBUG: Total staged files: {}", staged_files.len());
        for file in &staged_files {
            println!("DEBUG: Staged file: {}", file.display());
        }

        Ok(staged_files)
    }

    /// Internal function that applies the selective ignore logic to a file's content.
    ///
    /// It takes the original content and a set of `IgnorePattern`s, then processes
    /// the content line by line to determine which lines should be removed. It handles
    /// different pattern types, including `LineRegex`, `LineNumber`, and multi-line
    /// `BlockStartEnd` patterns.
    ///
    /// # Arguments
    /// * `content`: The string content of the file to be processed.
    /// * `patterns`: A slice of `IgnorePattern`s to apply.
    ///
    /// # Returns
    /// A `Result<(String, HashMap<usize, String>)>` where the first element is the
    /// new, cleaned content, and the second is a map of the zero-based line numbers
    /// and their original content that was ignored.
    fn process_file_content(
        &self,
        content: &str,
        patterns: &[IgnorePattern],
    ) -> Result<(String, HashMap<usize, String>)> {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let mut lines_to_ignore = HashMap::new();

        for pattern in patterns {
            match pattern.pattern_type {
                // Apply patterns that match single lines.
                PatternType::LineRegex | PatternType::LineNumber | PatternType::LineRange => {
                    for (i, line) in lines.iter().enumerate() {
                        if pattern.matches_line(line, i + 1)? {
                            // The key is the zero-based index of the line.
                            lines_to_ignore.insert(i, line.clone());
                        }
                    }
                }
                // Apply patterns that match a range of lines (e.g., a block comment).
                PatternType::BlockStartEnd => {
                    // Debug: Add some logging to see what's happening
                    println!(
                        "Processing BlockStartEnd pattern: {}",
                        pattern.specification
                    );
                    // The `get_block_range` method returns 1-based line numbers.
                    let ranges = pattern.get_block_range(content)?;
                    println!("Found {} block ranges", ranges.len());

                    for (start, end) in ranges {
                        println!("Block range: {start} to {end}");
                        // Ensure we're using 1-based line numbers from get_block_range
                        for i in start..=end {
                            if i > 0 && i <= lines.len() {
                                let zero_based_index = i - 1;
                                println!("Ignoring line {}: {}", i, lines[zero_based_index]);
                                lines_to_ignore
                                    .insert(zero_based_index, lines[zero_based_index].clone());
                            }
                        }
                    }
                }
            }
        }

        // Build the new content by filtering out the ignored lines.
        let mut new_content = String::new();
        for (i, line) in lines.iter().enumerate() {
            if !lines_to_ignore.contains_key(&i) {
                new_content.push_str(line);
                new_content.push('\n');
            }
        }

        // A final check to ensure we don't add an extra newline if the original file
        // didn't end with one.
        if !content.ends_with('\n') && new_content.ends_with('\n') {
            new_content.pop();
        }

        // Debug: Show what we're removing
        if !lines_to_ignore.is_empty() {
            println!(
                "Lines to ignore: {:?}",
                lines_to_ignore.keys().collect::<Vec<_>>()
            );
            println!("Original content length: {}", content.len());
            println!("New content length: {}", new_content.len());
        }

        Ok((new_content, lines_to_ignore))
    }
}

/// A simple helper function to calculate a hash of a string.
///
/// This hash is used for verifying file integrity between the pre-commit and post-commit
/// phases. It helps ensure that the file hasn't been modified by a user after the
/// pre-commit hook has run, which would prevent a dangerous restore operation.
fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish().to_string()
}
