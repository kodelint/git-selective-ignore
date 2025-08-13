use anyhow::{Context, Result};
use std::fmt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatternType {
    LineRegex,
    LineNumber,
    BlockStartEnd,
    LineRange,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IgnorePattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub specification: String,
    pub compiled_regex: Option<String>,
}


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

impl IgnorePattern {
    pub fn new(pattern_type: String, specification: String) -> Result<Self> {
        let pattern_type = match pattern_type.as_str() {
            "line-regex" => PatternType::LineRegex,
            "line-number" => PatternType::LineNumber,
            "block-start-end" => PatternType::BlockStartEnd,
            "line-range" => PatternType::LineRange,
            _ => anyhow::bail!("Invalid pattern type: {}", pattern_type),
        };

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

    pub fn validate(&self) -> Result<()> {
        match self.pattern_type {
            PatternType::LineRegex => {
                Regex::new(&self.specification).context("Invalid regex pattern")?;
            }
            PatternType::LineNumber => {
                self.specification
                    .parse::<usize>()
                    .context("Line number must be a valid integer")?;
            }
            PatternType::LineRange => {
                let parts: Vec<&str> = self.specification.split('-').collect();
                if parts.len() != 2 {
                    anyhow::bail!("Line range must be in format 'start-end'");
                }
                parts[0].parse::<usize>().context("Invalid start line")?;
                parts[1].parse::<usize>().context("Invalid end line")?;
            }
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
}