use anyhow::Result;
use std::collections::HashSet;

use crate::builders::patterns;
use crate::core::config;

/// The `ConfigValidator` trait defines the public interface for validating the
/// selective ignore configuration.
///
/// This trait allows for the implementation of different validation strategies,
/// such as a strict validator or a more permissive one, by adhering to a common
/// set of methods.
pub trait ConfigValidator {
    /// Performs a full validation of the `SelectiveIgnoreConfig` and returns
    /// a list of issues found.
    ///
    /// # Arguments
    /// * `config`: The `SelectiveIgnoreConfig` to be validated.
    ///
    /// # Returns
    /// A `Result<Vec<String>>` containing a vector of strings, where each string
    /// describes a specific validation issue.
    fn validate_config(&self, config: &config::SelectiveIgnoreConfig) -> Result<Vec<String>>;

    /// Validates a single `IgnorePattern` and returns a list of issues.
    ///
    /// # Arguments
    /// * `pattern`: The `IgnorePattern` to be validated.
    ///
    /// # Returns
    /// A `Result<Vec<String>>` containing a vector of strings, each describing a
    /// validation issue for the given pattern.
    fn validate_pattern(&self, pattern: &patterns::IgnorePattern) -> Result<Vec<String>>;
}

/// The `StandardValidator` is a concrete implementation of `ConfigValidator`.
///
/// It performs a series of standard checks to ensure the configuration file
/// is well-formed and does not contain potentially dangerous or conflicting
/// patterns.
pub struct StandardValidator;

impl StandardValidator {
    /// Creates a new instance of `StandardValidator`.
    pub fn new() -> Self {
        Self
    }

    /// Checks if a file exists at a given path.
    ///
    /// This is a simple helper function used to verify that configured file
    /// paths are valid in the current filesystem.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file to check.
    ///
    /// # Returns
    /// `true` if the file exists, `false` otherwise.
    fn check_file_exists(&self, file_path: &str) -> bool {
        std::path::Path::new(file_path).exists()
    }

    /// Checks for conflicting patterns within a single file's configuration.
    ///
    /// This is an important check to prevent unintended behavior. For example,
    /// it detects if multiple `LineNumber` patterns are defined for the same line.
    /// This check can be extended to cover other types of conflicts in the future.
    ///
    /// # Arguments
    /// * `patterns`: A slice of `IgnorePattern`s for a single file.
    ///
    /// # Returns
    /// A `Vec<String>` containing warnings for any conflicts found.
    fn check_pattern_conflicts(&self, patterns: &[patterns::IgnorePattern]) -> Vec<String> {
        let mut warnings = Vec::new();
        let mut line_numbers = HashSet::new();

        for pattern in patterns {
            if let patterns::PatternType::LineNumber = pattern.pattern_type
                && let Ok(line_num) = pattern.specification.parse::<usize>()
            {
                // Check if a pattern for this line number has already been seen.
                if line_numbers.contains(&line_num) {
                    warnings.push(format!(
                        "Duplicate line number pattern for line {}",
                        line_num
                    ));
                }
                line_numbers.insert(line_num);
            }
        }
        warnings
    }
}

impl ConfigValidator for StandardValidator {
    /// The main public method for validating the entire configuration.
    ///
    /// It orchestrates multiple checks, including:
    /// - Version compatibility.
    /// - Whether each configured file exists.
    /// - Conflicts between patterns within the same file.
    /// - The validity of each individual pattern's specification.
    fn validate_config(&self, config: &config::SelectiveIgnoreConfig) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Check for an unsupported configuration version.
        if config.version != "1.0" {
            issues.push(format!("Unsupported config version: {}", config.version));
        }

        // Iterate through each file and its patterns for validation.
        for (file_path, patterns) in &config.files {
            if file_path != "all" && !self.check_file_exists(file_path) {
                issues.push(format!("File not found: {file_path}"));
            }

            // Check for pattern conflicts within the file's patterns.
            let conflicts = self.check_pattern_conflicts(patterns);
            issues.extend(conflicts);

            // Validate each pattern's syntax and semantics.
            for pattern in patterns {
                let pattern_issues = self.validate_pattern(pattern)?;
                issues.extend(pattern_issues);
            }
        }

        Ok(issues)
    }

    /// Validates a single pattern's syntax and checks for problematic configurations.
    ///
    /// This function performs two levels of validation:
    /// 1. **Syntax Validation:** It calls the pattern's own `validate()` method
    ///    to check if its `specification` string is well-formed (e.g., a valid
    ///    regex or integer).
    /// 2. **Semantic Validation:** It checks for specific patterns that, while
    ///    syntactically correct, might be a mistake (e.g., an empty regex or a
    ///    pattern that matches everything).
    fn validate_pattern(&self, pattern: &patterns::IgnorePattern) -> Result<Vec<String>> {
        let mut issues = Vec::new();

        // Perform the pattern's own validation, which checks for correct syntax.
        if let Err(e) = pattern.validate() {
            issues.push(format!("Invalid pattern {}: {}", pattern.id, e));
        }

        // Check for patterns that are syntactically valid but semantically problematic.
        match pattern.pattern_type {
            patterns::PatternType::LineRegex => {
                if pattern.specification.is_empty() {
                    issues.push("Empty regex pattern will match nothing".to_string());
                }
                if pattern.specification == ".*" {
                    issues.push("Pattern '.*' will match all lines".to_string());
                }
            }
            patterns::PatternType::LineNumber => {
                if let Ok(line_num) = pattern.specification.parse::<usize>()
                    && line_num == 0
                {
                    issues.push("Line numbers start from 1, not 0".to_string());
                }
            }
            _ => {} // No specific semantic checks for other pattern types yet.
        }

        Ok(issues)
    }
}
