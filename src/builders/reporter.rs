use anyhow::Result;
use std::collections::HashMap;

use crate::builders::patterns::IgnorePattern;
use crate::core::config::SelectiveIgnoreConfig;

/// A struct that holds the status summary for a single file.
///
/// This provides a clean way to pass file-specific data from the `IgnoreEngine`
/// to the `StatusReporter`.
#[derive(Debug)]
pub struct FileStatus {
    /// Indicates whether the file exists in the filesystem.
    pub exists: bool,
    /// A flag indicating if the file contains lines that would be ignored by a pattern.
    pub has_ignored_lines: bool,
    /// The number of lines that match an ignore pattern.
    pub ignored_line_count: usize,
    /// The total number of lines in the file.
    pub total_lines: usize,
}

pub trait StatusReporter {
    fn generate_status_report(
        &self,
        config: &SelectiveIgnoreConfig,
        file_statuses: HashMap<String, FileStatus>,
    ) -> Result<()>;
}

/// A concrete implementation of `StatusReporter` that prints the report to the console.
///
/// This is the primary reporter used by the `show-status` command.
pub struct ConsoleReporter;

impl ConsoleReporter {
    /// Constructs a new `ConsoleReporter` instance.
    pub fn new() -> Self {
        Self
    }

    /// A private helper function to format the status message for a single file.
    ///
    /// This function generates a human-readable string with icons, file path,
    /// and a summary of the ignored lines.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file.
    /// * `status`: A reference to the `FileStatus` struct for this file.
    /// * `patterns`: A slice of `IgnorePattern`s configured for this file.
    ///
    /// # Returns
    /// A `String` containing the formatted status report line.
    fn format_file_status(
        &self,
        file_path: &str,
        status: &FileStatus,
        patterns: &[IgnorePattern],
    ) -> String {
        // Determine the appropriate emoji icon based on the file's status.
        // 🔴: File does not exist.
        // 🟡: File exists and has ignored lines.
        // 🟢: File exists but has no ignored lines.
        let status_icon = if status.exists {
            if status.has_ignored_lines {
                "🟡"
            } else {
                "🟢"
            }
        } else {
            "🔴"
        };

        // Calculate the percentage of ignored lines, handling the case of a zero-line file.
        let percentage = if status.total_lines > 0 {
            (status.ignored_line_count as f64 / status.total_lines as f64) * 100.0
        } else {
            0.0
        };

        // Format the final output string.
        format!(
            "{} {} ({} patterns, {}/{} lines ignored, {:.1}%)",
            status_icon,
            file_path,
            patterns.len(),
            status.ignored_line_count,
            status.total_lines,
            percentage
        )
    }
}

/// Implementation of the `StatusReporter` trait for `ConsoleReporter`.
impl StatusReporter for ConsoleReporter {
    /// Generates and prints the full status report to the standard output.
    fn generate_status_report(
        &self,
        config: &SelectiveIgnoreConfig,
        file_statuses: HashMap<String, FileStatus>,
    ) -> Result<()> {
        println!("📊 Git Selective Ignore Status Report");
        println!("=====================================");

        // If no files are configured, print a simple message and exit.
        if config.files.is_empty() {
            println!("No files configured for selective ignore.");
            return Ok(());
        }

        // Initialize counters for the summary section.
        let mut total_patterns = 0;
        let mut total_ignored_lines = 0;
        let mut files_with_issues = 0;

        // Count total patterns including global "all" patterns
        for patterns in config.files.values() {
            total_patterns += patterns.len();
        }

        // Separate files into specific and "all"-only categories
        let mut specific_files = Vec::new();
        let mut all_only_files = Vec::new();

        for (file_path, status) in &file_statuses {
            total_ignored_lines += status.ignored_line_count;

            // Increment the counter for files that don't exist in the working directory.
            if !status.exists {
                files_with_issues += 1;
            }

            // Check if this file has specific configuration or only "all" patterns
            if config.files.contains_key(file_path) {
                specific_files.push((file_path, status));
            } else {
                // This file is only affected by "all" patterns
                all_only_files.push((file_path, status));
            }
        }

        // Print specifically configured files first
        if !specific_files.is_empty() {
            println!("🎯 Specifically Configured Files:");
            for (file_path, status) in &specific_files {
                // Calculate how many patterns apply to this file
                let mut applicable_patterns = Vec::new();

                // Add file-specific patterns
                if let Some(file_specific_patterns) = config.files.get(*file_path) {
                    applicable_patterns.extend(file_specific_patterns.clone());
                }

                // Add global "all" patterns if they exist
                if let Some(global_patterns) = config.files.get("all") {
                    applicable_patterns.extend(global_patterns.clone());
                }

                // Print the formatted status line for the current file.
                println!(
                    "{}",
                    self.format_file_status(file_path, status, &applicable_patterns)
                );

                // If verbose mode is enabled, print the details of each pattern for the file.
                if config.global_settings.verbose {
                    for pattern in &applicable_patterns {
                        println!(
                            "  └─ {} ({}): {}",
                            pattern.id, pattern.pattern_type, pattern.specification
                        );
                    }
                }
            }
            println!(); // Add spacing
        }

        // Print files affected only by "all" patterns
        if !all_only_files.is_empty() && config.files.contains_key("all") {
            println!("🌐 Files Affected by Global 'ALL' Patterns:");
            let global_patterns = config.files.get("all").unwrap();

            for (file_path, status) in &all_only_files {
                // Print the formatted status line with only global patterns
                println!(
                    "{}",
                    self.format_file_status(file_path, status, global_patterns)
                );

                // If verbose mode is enabled, print the details of each pattern for the file.
                if config.global_settings.verbose {
                    for pattern in global_patterns {
                        println!(
                            "  └─ {} ({}): {}",
                            pattern.id, pattern.pattern_type, pattern.specification
                        );
                    }
                }
            }
        }

        // Print the final summary section.
        let actual_file_count = file_statuses.len();
        let files_with_problems = file_statuses
            .values()
            .filter(|status| status.has_ignored_lines)
            .count();

        println!("\n📈 Summary:");
        println!("  Total files: {actual_file_count}");
        println!("  Total patterns: {total_patterns}");
        println!("  Total ignored lines: {total_ignored_lines}");
        println!("  Files with issues: {files_with_problems}");

        // Show breakdown by category
        if !specific_files.is_empty() || !all_only_files.is_empty() {
            println!("\n📋 Breakdown:");
            if !specific_files.is_empty() {
                println!("  Specifically configured files: {}", specific_files.len());
            }
            if !all_only_files.is_empty() {
                println!(
                    "  Files affected by 'ALL' patterns only: {}",
                    all_only_files.len()
                );
            }
        }

        // Provide a hint to the user if any files had issues (e.g., didn't exist).
        if files_with_issues > 0 {
            println!("\n⚠️  Run with --verbose to see detailed pattern information");
        }

        Ok(())
    }
}
