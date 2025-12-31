use anyhow::Result;
use colored::Colorize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::builders::patterns::{IgnorePattern, PatternMatcher, PatternType};
use crate::builders::reporter::{ConsoleReporter, FileStatus, StatusReporter};
use crate::builders::storage::{BackupData, MemoryStorage, StorageProvider, TempFileStorage};
use crate::core::config::{BackupStrategy, ConfigManager, ConfigProvider};
use crate::core::git::{Git2Client, GitClient};

/// The `IgnoreEngine` is the central component responsible for managing the selective
/// ignore process within a Git repository. It acts as the orchestrator for the
/// `pre-commit` and `post-commit` hooks, coordinating file analysis, content modification,
/// and backup/restore operations.
///
/// This struct holds key state, including the Git repository abstraction, a configuration manager
/// to load settings, and a storage provider to handle temporary backups of modified files.
pub struct IgnoreEngine {
    config_manager: ConfigManager,
    storage: Box<dyn StorageProvider>,
    git_client: Box<dyn GitClient>,
}

impl IgnoreEngine {
    /// Constructs a new `IgnoreEngine` instance.
    pub fn new(config_manager: ConfigManager) -> Result<Self> {
        // Initialize Git client
        let git_client = Box::new(Git2Client::new(config_manager.get_repo_root())?);

        // Load the configuration to determine the backup strategy.
        let config = config_manager.load_config()?;
        let storage: Box<dyn StorageProvider> = match config.global_settings.backup_strategy {
            BackupStrategy::Memory => Box::new(MemoryStorage::new()),
            BackupStrategy::TempFile => Box::new(TempFileStorage::new(git_client.get_git_dir())?),
            BackupStrategy::GitStash => {
                // For now, fallback to TempFile.
                Box::new(TempFileStorage::new(git_client.get_git_dir())?)
            }
        };

        Ok(Self {
            config_manager,
            storage,
            git_client,
        })
    }

    /// The main entry point for the `pre-commit` Git hook.
    pub fn process_pre_commit(&mut self, dry_run: bool) -> Result<()> {
        let config = self.config_manager.load_config()?;
        let funny = config.global_settings.funny_mode;

        if dry_run {
            println!(
                "{}",
                "🔍 DRY RUN: No changes will be persisted.".cyan().bold()
            );
        }

        if funny {
            println!(
                "{}",
                "🧙‍♂️  Abra Kadabra! Vanishing unwanted lines...".magenta()
            );
        } else {
            println!(
                "{}",
                "📝 Processing files with selective ignore patterns...".yellow()
            );
        }

        let staged_files = self.git_client.get_staged_files()?;
        let mut files_to_add_after_processing = Vec::new();

        for file_path in staged_files.iter() {
            let file_path_str = file_path.to_string_lossy().to_string();

            // Collect all patterns that apply to this file
            let mut all_patterns = Vec::new();

            if let Some(global_patterns) = config.files.get("all") {
                all_patterns.extend(global_patterns.clone());
            }

            if let Some(file_specific_patterns) = config.files.get(&file_path_str) {
                all_patterns.extend(file_specific_patterns.clone());
            }

            if !all_patterns.is_empty() {
                println!("\n📄 Processing file: {}", file_path_str.bright_cyan());
                println!(
                    "   └─ Found {} ignore pattern(s) installed",
                    all_patterns.len().to_string().blue()
                );

                let original_content = self.git_client.read_staged_file_content(file_path)?;

                let (cleaned_content, ignored_lines) =
                    self.process_file_content(&original_content, &all_patterns, &file_path_str)?;

                if cleaned_content != original_content {
                    if !dry_run {
                        let backup_data = BackupData {
                            original_content: original_content.to_string(),
                            ignored_lines,
                            original_file_hash: calculate_hash(&original_content),
                            cleaned_file_hash: calculate_hash(&cleaned_content),
                        };
                        self.storage.store_backup(&file_path_str, backup_data)?;

                        // Write the cleaned content to the working directory.
                        self.git_client
                            .write_working_file(file_path, &cleaned_content)?;

                        // Mark the file to be re-staged.
                        files_to_add_after_processing.push(file_path.clone());
                    } else {
                        println!(
                            "   └─ {} Would modify and re-stage this file",
                            "DRY RUN:".cyan()
                        );
                    }
                }
            }
        }

        if !files_to_add_after_processing.is_empty() && !dry_run {
            println!("\n🔄 Re-staging modified files...");
            for path in files_to_add_after_processing {
                self.git_client.stage_file(&path)?;
            }
        }

        if funny {
            println!("✨ Mischief managed.");
        } else {
            println!("✅ Pre-commit processing complete.");
        }
        Ok(())
    }

    /// The main entry point for the `post-commit` Git hook.
    pub fn process_post_commit(&mut self, dry_run: bool) -> Result<()> {
        let config = self.config_manager.load_config()?;
        let funny = config.global_settings.funny_mode;

        if dry_run {
            println!(
                "{}",
                "🔍 DRY RUN: No changes will be persisted.".cyan().bold()
            );
        }

        if funny {
            println!("🧟  It's alive! Bringing lines back from the dead...");
        } else {
            println!("🔄 Restoring files after commit...");
        }

        // Restore files with specific patterns
        for file_path in config.files.keys() {
            if file_path == "all" {
                continue;
            }
            let path = Path::new(file_path);

            if let Some(backup_data) = self.storage.restore_backup(file_path)? {
                // Check if file exists in working dir before reading
                if self.git_client.file_exists(path) {
                    let current_content = self.git_client.read_working_file(path)?;
                    if calculate_hash(&current_content) == backup_data.cleaned_file_hash {
                        if !dry_run {
                            self.git_client
                                .write_working_file(path, &backup_data.original_content)?;
                            println!("✓ Restored {file_path}");
                        } else {
                            println!("   └─ {} Would restore {file_path}", "DRY RUN:".cyan());
                        }
                    } else {
                        println!(
                            "⚠️ Skipping restore for {file_path} - file was modified after pre-commit"
                        );
                    }
                }
            }
        }

        // Handle "all" patterns
        if config.files.contains_key("all") {
            let all_backup_keys = self.storage.get_all_backup_keys()?;
            let specific_file_keys: HashSet<String> = config
                .files
                .keys()
                .filter(|&k| k != "all")
                .cloned()
                .collect();

            for backup_key in all_backup_keys {
                if !specific_file_keys.contains(&backup_key) {
                    let path = Path::new(&backup_key);

                    if let Some(backup_data) = self.storage.restore_backup(&backup_key)?
                        && self.git_client.file_exists(path)
                    {
                        let current_content = self.git_client.read_working_file(path)?;
                        if calculate_hash(&current_content) == backup_data.cleaned_file_hash {
                            if !dry_run {
                                self.git_client
                                    .write_working_file(path, &backup_data.original_content)?;
                                println!("✓ Restored {backup_key}");
                            } else {
                                println!("   └─ {} Would restore {backup_key}", "DRY RUN:".cyan());
                            }
                        } else {
                            println!(
                                "⚠️ Skipping restore for {backup_key} - file was modified after pre-commit"
                            );
                        }
                    }
                }
            }
        }

        if config.global_settings.auto_cleanup && !dry_run {
            self.storage.cleanup()?;
        }

        if funny {
            println!("🎉  All restored. Like nothing happened.");
        } else {
            println!("✅ Post-commit processing complete.");
        }
        Ok(())
    }

    /// Generates and displays a status report for all configured files.
    pub fn show_status(&mut self) -> Result<()> {
        let config = self.config_manager.load_config()?;
        let mut file_statuses = HashMap::new();
        let reporter = ConsoleReporter::new();

        // Get all files that could be affected
        let mut files_to_check = std::collections::HashSet::new();

        // Add explicitly configured files (excluding "all")
        for file_path in config.files.keys() {
            if file_path != "all" {
                files_to_check.insert(file_path.clone());
            }
        }

        // If there are "all" patterns, find files they could apply to
        if config.files.contains_key("all") {
            // Get all tracked files
            let tracked_files = self.git_client.get_tracked_files()?;
            for f in tracked_files {
                files_to_check.insert(f);
            }

            // Also check staged files
            let staged_files = self.git_client.get_staged_files()?;
            for staged_file in staged_files {
                files_to_check.insert(staged_file.to_string_lossy().to_string());
            }
        }

        // Process each file
        for file_path in files_to_check {
            let path = Path::new(&file_path);
            let mut status = FileStatus {
                exists: self.git_client.file_exists(path),
                has_ignored_lines: false,
                ignored_line_count: 0,
                total_lines: 0,
            };

            if status.exists {
                let content = self.git_client.read_working_file(path)?;
                status.total_lines = content.lines().count();

                // Collect all patterns that apply to this file
                let mut all_patterns = Vec::new();
                if let Some(file_specific_patterns) = config.files.get(&file_path) {
                    all_patterns.extend(file_specific_patterns.clone());
                }
                if let Some(global_patterns) = config.files.get("all") {
                    all_patterns.extend(global_patterns.clone());
                }

                if !all_patterns.is_empty() {
                    let (_, ignored_lines) =
                        self.process_file_content(&content, &all_patterns, &file_path)?;
                    if !ignored_lines.is_empty() {
                        status.has_ignored_lines = true;
                        status.ignored_line_count = ignored_lines.len();
                    }
                }
            }

            if status.has_ignored_lines {
                file_statuses.insert(file_path, status);
            }
        }

        reporter.generate_status_report(&config, file_statuses)?;
        Ok(())
    }

    /// Verifies that no ignored content is present in the Git staging area.
    pub fn verify_staging(&mut self) -> Result<()> {
        println!("🕵️ Verifying staging area for ignored content...");
        let config = self.config_manager.load_config()?;

        let staged_files = self.git_client.get_staged_files()?;
        let mut violations = Vec::new();

        for file_path in staged_files {
            let file_path_str = file_path.to_string_lossy().to_string();

            let mut all_patterns = Vec::new();
            if let Some(global_patterns) = config.files.get("all") {
                all_patterns.extend(global_patterns.clone());
            }
            if let Some(file_specific_patterns) = config.files.get(&file_path_str) {
                all_patterns.extend(file_specific_patterns.clone());
            }

            if !all_patterns.is_empty() {
                let content = self.git_client.read_staged_file_content(&file_path)?;

                for pattern in &all_patterns {
                    let (_, ignored_lines) = self.process_file_content(
                        &content,
                        std::slice::from_ref(pattern),
                        &file_path_str,
                    )?;

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

    fn process_file_content(
        &self,
        content: &str,
        patterns: &[IgnorePattern],
        _file_path: &str,
    ) -> Result<(String, HashMap<usize, String>)> {
        let lines: Vec<String> = content.lines().map(String::from).collect();
        let mut lines_to_ignore = HashMap::new();
        let mut pattern_matches = Vec::new();

        for pattern in patterns {
            let mut current_pattern_matches = Vec::new();

            match pattern.pattern_type {
                PatternType::LineRegex | PatternType::LineNumber | PatternType::LineRange => {
                    for (i, line) in lines.iter().enumerate() {
                        if pattern.matches_line(line, i + 1)? {
                            lines_to_ignore.insert(i, line.clone());
                            current_pattern_matches.push(i + 1);
                        }
                    }
                }
                PatternType::BlockStartEnd => {
                    let ranges = pattern.get_block_range(content)?;
                    for (start, end) in ranges {
                        for i in start..=end {
                            if i > 0 && i <= lines.len() {
                                let zero_based_index = i - 1;
                                lines_to_ignore
                                    .insert(zero_based_index, lines[zero_based_index].clone());
                                current_pattern_matches.push(i);
                            }
                        }
                    }
                }
            }

            if !current_pattern_matches.is_empty() {
                pattern_matches.push((pattern, current_pattern_matches));
            }
        }

        if !pattern_matches.is_empty() {
            for (pattern, matched_lines) in &pattern_matches {
                let pattern_type_str = match pattern.pattern_type {
                    PatternType::LineRegex => "Regex",
                    PatternType::LineNumber => "Line Number",
                    PatternType::LineRange => "Line Range",
                    PatternType::BlockStartEnd => "Block",
                };

                println!(
                    "   ├─ {} Pattern '{}': {} line(s) matched",
                    pattern_type_str,
                    pattern.specification,
                    matched_lines.len()
                );

                let grouped_lines = Self::group_consecutive_lines(matched_lines);
                for group in grouped_lines {
                    if group.len() == 1 {
                        println!("   │  └─ Line {}", group[0]);
                    } else {
                        println!("   │  └─ Lines {}-{}", group[0], group[group.len() - 1]);
                    }
                }
            }

            let total_ignored = lines_to_ignore.len();
            let total_lines = lines.len();
            let remaining_lines = total_lines - total_ignored;

            println!(
                "   └─ {}: {} line(s) ignored, {} line(s) remaining (of {} total)",
                "Summary".bright_green().bold(),
                total_ignored,
                remaining_lines,
                total_lines
            );
        } else {
            println!("   └─ No lines matched any patterns");
        }

        let kept_lines: Vec<&str> = lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if !lines_to_ignore.contains_key(&i) {
                    Some(line.as_str())
                } else {
                    None
                }
            })
            .collect();

        let mut cleaned_lines = Vec::new();
        let mut prev_line_was_blank = false;

        for line in kept_lines {
            let current_line_is_blank = line.trim().is_empty();

            if current_line_is_blank {
                if !prev_line_was_blank {
                    cleaned_lines.push(line);
                }
                prev_line_was_blank = true;
            } else {
                cleaned_lines.push(line);
                prev_line_was_blank = false;
            }
        }

        let mut new_content = cleaned_lines.join("\n");

        if content.ends_with('\n') && !new_content.is_empty() {
            new_content.push('\n');
        }

        Ok((new_content, lines_to_ignore))
    }

    fn group_consecutive_lines(lines: &[usize]) -> Vec<Vec<usize>> {
        if lines.is_empty() {
            return vec![];
        }

        let mut sorted_lines = lines.to_vec();
        sorted_lines.sort();

        let mut groups = vec![];
        let mut current_group = vec![sorted_lines[0]];

        for &line in &sorted_lines[1..] {
            if line == current_group.last().unwrap() + 1 {
                current_group.push(line);
            } else {
                groups.push(current_group);
                current_group = vec![line];
            }
        }

        groups.push(current_group);
        groups
    }
}

fn calculate_hash(content: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish().to_string()
}
