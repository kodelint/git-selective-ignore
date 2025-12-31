use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// An enum that defines the different types of patterns supported by the engine.
///
/// Each variant corresponds to a different method for identifying lines or blocks
/// of text to be ignored. This design allows for a flexible and extensible
/// pattern-matching system.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatternType {
    /// Matches lines using a regular expression. The specification string can be
    /// either a raw regex (enclosed in `/` delimiters) or a literal word that
    /// will be matched with word boundaries (`\b`).
    LineRegex,
    /// Matches a single, specific line number. The specification is a string
    /// representation of a 1-based line number (e.g., "42").
    LineNumber,
    /// Matches a block of lines starting with one literal string and ending
    /// with another. The specification is a string in the format
    /// `start_pattern|||end_pattern`.
    BlockStartEnd,
    /// Matches a contiguous range of line numbers. The specification is a string
    /// in the format `start_line-end_line` (e.g., "10-20").
    LineRange,
}

/// Represents a single selective ignore pattern defined in the configuration.
///
/// This struct holds all the necessary information to identify and handle a specific
/// pattern, including its type, the string specification, and a pre-compiled regex
/// for efficiency where applicable.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnorePattern {
    /// A unique identifier for the pattern, useful for tracking and management.
    pub id: String,
    /// The type of pattern, which dictates how the `specification` is interpreted.
    pub pattern_type: PatternType,
    /// The raw string defining the pattern (e.g., a regex string, a line number, etc.).
    pub specification: String,
    /// An optional string that stores a pre-compiled regex pattern. This is not
    /// a `regex::Regex` object directly because `IgnorePattern` needs to be
    /// serializable and cloneable without a lifetime. The `Regex` object is
    /// created on-the-fly during matching.
    pub compiled_regex: Option<String>,
}

/// Implements `fmt::Display` to provide a user-friendly string representation
/// for each `PatternType`. This is useful for logging and reporting.
impl fmt::Display for PatternType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternType::LineRegex => write!(f, "line-regex"),
            PatternType::LineNumber => write!(f, "line-number"),
            PatternType::BlockStartEnd => write!(f, "block-start-end"),
            PatternType::LineRange => write!(f, "line-range"),
        }
    }
}

/// The `PatternMatcher` trait defines the core behavior for matching a pattern.
///
/// This trait allows the `IgnoreEngine` to treat all pattern types uniformly when
/// processing file content, abstracting away the specifics of how each pattern works.
/// Implementors of this trait provide the logic for checking a line or finding a block.
pub trait PatternMatcher {
    /// Checks if a single line of content matches the pattern.
    ///
    /// This method is intended for line-based patterns (`LineRegex`, `LineNumber`, `LineRange`).
    /// For block patterns, this method should always return `Ok(false)` as their logic
    /// is handled by `get_block_range`.
    ///
    /// # Arguments
    /// * `line`: The string slice of the line to check.
    /// * `line_number`: The 1-based line number of the current line.
    ///
    /// # Returns
    /// `Result<bool>` which is `true` if the line matches, `false` otherwise.
    /// An `anyhow::Error` is returned if parsing the specification fails.
    fn matches_line(&self, line: &str, line_number: usize) -> Result<bool>;

    /// Finds and returns all line number ranges that match a block pattern.
    ///
    /// This method is specifically for `BlockStartEnd` patterns and returns a vector
    /// of tuples, where each tuple represents a `(start_line, end_line)` range.
    ///
    /// # Arguments
    /// * `content`: The full string content of the file to search.
    ///
    /// # Returns
    /// `Result<Vec<(usize, usize)>>` which is a vector of 1-based line number ranges.
    /// Returns an empty vector for non-block patterns.
    fn get_block_range(&self, content: &str) -> Result<Vec<(usize, usize)>>;
}

impl IgnorePattern {
    /// Creates a new `IgnorePattern` from a given type and specification string.
    ///
    /// This constructor handles the conversion of a string `pattern_type` into the
    /// `PatternType` enum and initializes the `compiled_regex` field if needed.
    /// It also assigns a new, unique UUID to the pattern for identification.
    ///
    /// # Arguments
    /// * `pattern_type`: A string representing the type of pattern (e.g., "line-regex").
    /// * `specification`: The raw string that defines the pattern itself.
    ///
    /// # Returns
    /// `Result<Self>` which is the new `IgnorePattern` instance, or an error
    /// if the `pattern_type` string is invalid.
    pub fn new(pattern_type: String, specification: String) -> Result<Self> {
        let pattern_type = match pattern_type.as_str() {
            "line-regex" => PatternType::LineRegex,
            "line-number" => PatternType::LineNumber,
            "block-start-end" => PatternType::BlockStartEnd,
            "line-range" => PatternType::LineRange,
            _ => anyhow::bail!("Invalid pattern type: {}", pattern_type),
        };
        // For `LineRegex` and `BlockStartEnd`, the specification string itself
        // serves as the compiled pattern, which can be validated later.
        let compiled_regex = if matches!(
            pattern_type,
            PatternType::LineRegex | PatternType::BlockStartEnd
        ) {
            Some(specification.clone())
        } else {
            None
        };

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            pattern_type,
            specification,
            compiled_regex,
        })
    }

    /// Validates the pattern's specification string based on its type.
    ///
    /// This function ensures that the pattern is well-formed before it is
    /// used for matching. It checks for valid regex syntax, integer parsing,
    /// and correct block pattern formatting.
    pub fn validate(&self) -> Result<()> {
        match self.pattern_type {
            PatternType::LineRegex => {
                // For LineRegex, we now support both direct regex patterns and word-boundary patterns
                // Try to validate as regex first, but if it fails, treat it as a literal word pattern
                if self.specification.starts_with('/') && self.specification.ends_with('/') {
                    // It's a regex pattern enclosed in slashes
                    let regex_pattern = &self.specification[1..self.specification.len() - 1];
                    Regex::new(regex_pattern).context("Invalid regex pattern")?;
                } else {
                    // It's a word/identifier pattern - create word boundary regex to validate
                    let word_boundary_pattern =
                        format!(r"\b{}\b", regex::escape(&self.specification));
                    Regex::new(&word_boundary_pattern).context("Invalid word pattern")?;
                }
            }
            // Validate that the specification is a parsable integer.
            PatternType::LineNumber => {
                self.specification
                    .parse::<usize>()
                    .context("Line number must be a valid integer")?;
            }
            // Validate the format 'start-end' and that both parts are integers.
            PatternType::LineRange => {
                let parts: Vec<&str> = self.specification.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Line range must be in format 'start-end'");
                }
                parts[0].parse::<usize>().context("Invalid start line")?;
                parts[1].parse::<usize>().context("Invalid end line")?;
            }
            // Validate the format 'start_pattern|||end_pattern' and that
            // neither part is empty. The patterns themselves are treated as
            // literal strings, not regexes, so no further validation is needed.
            PatternType::BlockStartEnd => {
                let parts: Vec<&str> = self.specification.split("|||").collect();
                if parts.len() != 2 {
                    anyhow::bail!("Block pattern must be in format 'start_pattern|||end_pattern'");
                }
                // Don't validate as regex - they can be literal strings
                if parts[0].trim().is_empty() || parts[1].trim().is_empty() {
                    anyhow::bail!("Start and end patterns cannot be empty");
                }
            }
        }
        Ok(())
    }

    /// Creates the appropriate regex pattern for `LineRegex` matching.
    ///
    /// This method handles two cases:
    /// 1. If the specification is enclosed in slashes (`/pattern/`), it's treated as a raw regex.
    /// 2. Otherwise, it's treated as a word/identifier that should match with word boundaries.
    ///
    /// This approach ensures a user can define simple word matches without needing to
    /// know regex syntax.
    fn create_line_regex_pattern(&self) -> String {
        if self.specification.starts_with('/') && self.specification.ends_with('/') {
            // Extract raw regex pattern from between the slashes
            self.specification[1..self.specification.len() - 1].to_string()
        } else {
            // Create hardcoded assignment detection pattern that handles various contexts
            let var_name = regex::escape(&self.specification);
            format!(r#"\b{}\s*=\s*(?:"[^"]+"|'[^']+')"#, var_name)
        }
    }
}

/// Implementation of the `PatternMatcher` trait for the `IgnorePattern` struct.
/// This trait implementation provides the core matching logic for all pattern types.
impl PatternMatcher for IgnorePattern {
    /// Checks if a single line of content matches the pattern.
    ///
    /// This method is intended for line-based patterns (`LineRegex`, `LineNumber`, `LineRange`).
    /// For block patterns, this method should always return `Ok(false)` as their logic
    /// is handled by `get_block_range`.
    ///
    /// # Arguments
    /// * `line`: The string slice of the line to check.
    /// * `line_number`: The 1-based line number of the current line.
    ///
    /// # Returns
    /// `Result<bool>` which is `true` if the line matches, `false` otherwise.
    /// An `anyhow::Error` is returned if parsing the specification fails.
    fn matches_line(&self, line: &str, line_number: usize) -> Result<bool> {
        match self.pattern_type {
            PatternType::LineRegex => {
                // Create the appropriate regex pattern based on specification format
                let regex_pattern = self.create_line_regex_pattern();
                let regex = Regex::new(&regex_pattern)?;
                Ok(regex.is_match(line))
            }
            PatternType::LineNumber => {
                // Parse the specification as a line number and compare.
                let target_line: usize = self.specification.parse()?;
                Ok(line_number == target_line)
            }
            PatternType::LineRange => {
                // Parse the start and end of the range and check if the line number falls within it.
                let parts: Vec<&str> = self.specification.split('-').collect();
                let start: usize = parts[0].parse()?;
                let end: usize = parts[1].parse()?;
                Ok(line_number >= start && line_number <= end)
            }
            PatternType::BlockStartEnd => {
                // This pattern type is not designed to match a single line,
                // so it always returns false here. The block matching logic
                // is handled by `get_block_range`.
                Ok(false)
            }
        }
    }

    /// Finds and returns all line number ranges that match a block pattern.
    ///
    /// This method is specifically for `BlockStartEnd` patterns and returns a vector
    /// of tuples, where each tuple represents a `(start_line, end_line)` range.
    ///
    /// # Arguments
    /// * `content`: The full string content of the file to search.
    ///
    /// # Returns
    /// `Result<Vec<(usize, usize)>>` which is a vector of 1-based line number ranges.
    /// Returns an empty vector for non-block patterns.
    fn get_block_range(&self, content: &str) -> Result<Vec<(usize, usize)>> {
        // Only proceed if the pattern is `BlockStartEnd`.
        if !matches!(self.pattern_type, PatternType::BlockStartEnd) {
            return Ok(vec![]);
        }

        // Split the specification into the start and end literal strings.
        let parts: Vec<&str> = self.specification.split("|||").collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "BlockStartEnd pattern must have start and end separated by |||"
            ));
        }

        let start_pattern = parts[0].trim();
        let end_pattern = parts[1].trim();
        let mut ranges = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0; // The current line index (0-based)

        while i < lines.len() {
            // Look for start pattern using contains() for literal string matching
            if lines[i].contains(start_pattern) {
                let start_line = i + 1; // 1-based line number for the start

                // Look for end pattern
                let mut found_end = false;
                // Start a nested loop to search for the end pattern from the next line.
                for (j, line) in lines.iter().enumerate().skip(i + 1) {
                    if line.contains(end_pattern) {
                        // println!("DEBUG: Found end pattern at line {}: '{}'", j + 1, lines[j]);
                        let end_line = j + 1; // Convert to 1-based line number
                        ranges.push((start_line, end_line));
                        // Move the outer loop's index past the end of the found block
                        // to prevent re-matching patterns within the same block.
                        i = j + 1; // Continue searching after this block
                        found_end = true;
                        break; // Exit the inner loop
                    }
                }

                // If no end pattern was found, the start pattern is ignored.
                if !found_end {
                    // If no end pattern found, just ignore the start pattern
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Ok(ranges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_pattern_matching() {
        let pattern = IgnorePattern::new("line-regex".to_string(), "/^debug/".to_string()).unwrap();
        assert!(pattern.matches_line("debug: something happened", 1).unwrap());
        assert!(!pattern.matches_line("info: something happened", 2).unwrap());
    }

    #[test]
    fn test_word_assignment_pattern_matching() {
        let pattern = IgnorePattern::new("line-regex".to_string(), "api_key".to_string()).unwrap();
        assert!(pattern.matches_line("api_key = \"secret\"", 1).unwrap());
        assert!(pattern.matches_line("  api_key = 'another-secret' ", 2).unwrap());
        assert!(!pattern.matches_line("not_an_api_key = \"val\"", 3).unwrap());
    }

    #[test]
    fn test_line_number_matching() {
        let pattern = IgnorePattern::new("line-number".to_string(), "5".to_string()).unwrap();
        assert!(pattern.matches_line("any content", 5).unwrap());
        assert!(!pattern.matches_line("any content", 4).unwrap());
    }

    #[test]
    fn test_line_range_matching() {
        let pattern = IgnorePattern::new("line-range".to_string(), "10-20".to_string()).unwrap();
        assert!(pattern.matches_line("line 10", 10).unwrap());
        assert!(pattern.matches_line("line 15", 15).unwrap());
        assert!(pattern.matches_line("line 20", 20).unwrap());
        assert!(!pattern.matches_line("line 9", 9).unwrap());
        assert!(!pattern.matches_line("line 21", 21).unwrap());
    }

    #[test]
    fn test_block_pattern_matching() {
        let pattern = IgnorePattern::new(
            "block-start-end".to_string(),
            "BEGIN DEBUG|||END DEBUG".to_string(),
        )
        .unwrap();
        let content = "line 1\nBEGIN DEBUG\nline 3\nline 4\nEND DEBUG\nline 6";
        let ranges = pattern.get_block_range(content).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0], (2, 5));
    }
}
