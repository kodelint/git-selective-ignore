use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::io;

use crate::builders::patterns::IgnorePattern;

/// A trait that defines the behavior for importing ignore patterns from a source.
///
/// This trait allows for different implementations of pattern importers (e.g., from
/// files, from a network source) to be used interchangeably.
pub trait PatternImporter {
    /// Imports patterns from a file and returns them in a structured format.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file to be imported.
    /// * `import_type`: The type of format to parse (e.g., "gitignore", "custom").
    ///
    /// # Returns
    /// A `Result<HashMap<String, Vec<IgnorePattern>>>`. The HashMap maps file paths
    /// to a vector of `IgnorePattern`s, ready to be merged into the main configuration.
    fn import_from_file(
        &mut self,
        file_path: &str,
        import_type: &str,
    ) -> Result<HashMap<String, Vec<IgnorePattern>>>;
}

/// A concrete implementation of `PatternImporter` for handling file-based imports.
///
/// This struct contains the logic for parsing different file formats and converting
/// their content into the internal `IgnorePattern` representation.
pub struct FileImporter;

/// Implementation of the `PatternImporter` trait for `FileImporter`.
impl PatternImporter for FileImporter {
    /// The main public method for importing patterns from a file.
    ///
    /// This function dispatches to the correct parsing method based on the
    /// `import_type` argument. It also includes an interactive step for
    /// `gitignore` imports to prompt the user for the target file.
    ///
    /// # Arguments
    /// * `file_path`: The path to the file to be imported.
    /// * `import_type`: A string indicating the format ("gitignore" or "custom").
    ///
    /// # Returns
    /// A `Result<HashMap<String, Vec<IgnorePattern>>>` with the parsed patterns.
    fn import_from_file(
        &mut self,
        file_path: &str,
        import_type: &str,
    ) -> Result<HashMap<String, Vec<IgnorePattern>>> {
        // Read the entire file content into a string.
        let content = fs::read_to_string(file_path).context("Failed to read import file")?;

        match import_type {
            // For `gitignore` imports, the patterns are not tied to a specific file.
            // The tool must interactively ask the user which file to apply them to.
            "gitignore" => {
                // For gitignore, we need to ask which file to apply patterns to
                println!("Enter the target file path for these patterns:");
                let mut target_file = String::new();
                io::stdin().read_line(&mut target_file)?;
                let target_file = target_file.trim().to_string();

                // Parse the gitignore-style content.
                let patterns = self.parse_gitignore_style(&content, &target_file)?;
                let mut result = HashMap::new();
                // Associate all the parsed patterns with the single `target_file`.
                result.insert(target_file, patterns);
                Ok(result)
            }
            // The custom format already contains file paths, so we can directly
            // parse the content and return the result. The `_` arm
            // acts as a default for any unrecognized type.
            "custom" | _ => self.parse_custom_format(&content),
        }
    }
}

impl FileImporter {
    /// Constructs a new `FileImporter` instance.
    pub fn new() -> Self {
        Self
    }

    /// Parses a file using `.gitignore`-style syntax.
    ///
    /// This function reads each line, ignores comments and empty lines, and converts
    /// the glob-style patterns into a `line-regex` pattern. For example, `*.log`
    /// becomes `.*\.log`.
    ///
    /// # Arguments
    /// * `content`: The full string content of the `.gitignore` file.
    /// * `target_file`: The file path that these patterns should be applied to.
    ///
    /// # Returns
    /// A `Result<Vec<IgnorePattern>>` containing the converted patterns.
    fn parse_gitignore_style(
        &self,
        content: &str,
        _target_file: &str,
    ) -> Result<Vec<IgnorePattern>> {
        let mut patterns = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // Skip empty lines and comments (lines starting with '#').
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Convert `.gitignore` style glob patterns into regular expressions.
            // '*' is replaced with '.*' to match any sequence of characters.
            // '?' is replaced with '.' to match any single character.
            let regex_pattern = line.replace("*", ".*").replace("?", ".");
            // Create a new `IgnorePattern` with the `LineRegex` type.
            patterns.push(IgnorePattern::new("line-regex".to_string(), regex_pattern)?);
        }

        // Return the vector of patterns. The `target_file` argument is used
        // in the calling function to associate these patterns with a file path.
        Ok(patterns)
    }

    /// Parses a custom-formatted file for importing patterns.
    ///
    /// The custom format is a simple `.ini`-style format where files are defined
    /// in `[file_path]` sections, and patterns are listed on subsequent lines
    /// as `pattern_type:specification`.
    ///
    /// # Arguments
    /// * `content`: The full string content of the custom format file.
    ///
    /// # Returns
    /// A `Result<HashMap<String, Vec<IgnorePattern>>>` mapping file paths to
    /// their respective patterns.
    fn parse_custom_format(&self, content: &str) -> Result<HashMap<String, Vec<IgnorePattern>>> {
        let mut result = HashMap::new();
        let mut current_file = String::new();

        for line in content.lines() {
            let line = line.trim();
            // Skip comments and empty lines.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Check for a new file section header like `[file_path]`.
            if line.starts_with('[') && line.ends_with(']') {
                // Extract the file path from within the brackets.
                current_file = line[1..line.len() - 1].to_string();
            } else if !current_file.is_empty() {
                // If we are inside a file section, parse the pattern line.
                // The line is expected to be in the format `type:pattern`.
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    // Create a new `IgnorePattern` from the parsed parts.
                    let pattern = IgnorePattern::new(parts[0].to_string(), parts[1].to_string())?;

                    // Add the new pattern to the HashMap, creating a new vector
                    // if this is the first pattern for the `current_file`.
                    result
                        .entry(current_file.clone())
                        .or_insert_with(Vec::new)
                        .push(pattern);
                }
            }
        }

        Ok(result)
    }
}