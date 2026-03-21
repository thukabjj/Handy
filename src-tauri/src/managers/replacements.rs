//! Text replacement manager for post-transcription processing.
//!
//! Supports:
//! - Simple text replacements
//! - Regular expression patterns
//! - Magic commands: [lowercase], [uppercase], [capitalize], [date], [time]

use chrono::Local;
use log::{debug, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;

/// A text replacement rule.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct TextReplacement {
    /// Unique identifier for this replacement.
    pub id: String,
    /// Whether this replacement is enabled.
    pub enabled: bool,
    /// The pattern to match (literal text or regex).
    pub pattern: String,
    /// The replacement text (supports magic commands).
    pub replacement: String,
    /// Whether the pattern is a regular expression.
    pub is_regex: bool,
    /// Whether the match should be case-insensitive.
    pub case_insensitive: bool,
    /// Optional description of what this replacement does.
    #[serde(default)]
    pub description: Option<String>,
}

impl TextReplacement {
    /// Create a new text replacement with a generated ID.
    pub fn new(pattern: String, replacement: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            enabled: true,
            pattern,
            replacement,
            is_regex: false,
            case_insensitive: false,
            description: None,
        }
    }

    /// Apply this replacement to the given text.
    pub fn apply(&self, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }

        let replacement = expand_magic_commands(&self.replacement);

        if self.is_regex {
            self.apply_regex(text, &replacement)
        } else {
            self.apply_literal(text, &replacement)
        }
    }

    fn apply_regex(&self, text: &str, replacement: &str) -> String {
        let regex_result = if self.case_insensitive {
            Regex::new(&format!("(?i){}", &self.pattern))
        } else {
            Regex::new(&self.pattern)
        };

        match regex_result {
            Ok(re) => re.replace_all(text, replacement).to_string(),
            Err(e) => {
                warn!("Invalid regex pattern '{}': {}", self.pattern, e);
                text.to_string()
            }
        }
    }

    fn apply_literal(&self, text: &str, replacement: &str) -> String {
        if self.case_insensitive {
            // Case-insensitive literal replacement
            let pattern_lower = self.pattern.to_lowercase();
            let mut result = String::with_capacity(text.len());
            let mut remaining = text;

            while let Some(pos) = remaining.to_lowercase().find(&pattern_lower) {
                result.push_str(&remaining[..pos]);
                result.push_str(replacement);
                remaining = &remaining[pos + self.pattern.len()..];
            }
            result.push_str(remaining);
            result
        } else {
            text.replace(&self.pattern, replacement)
        }
    }
}

/// Expand magic commands in replacement text.
fn expand_magic_commands(text: &str) -> String {
    let mut result = text.to_string();

    // Date/time commands
    let now = Local::now();
    result = result.replace("[date]", &now.format("%Y-%m-%d").to_string());
    result = result.replace("[time]", &now.format("%H:%M").to_string());
    result = result.replace("[datetime]", &now.format("%Y-%m-%d %H:%M").to_string());

    result
}

/// Apply text transformations based on magic commands.
#[allow(dead_code)]
fn apply_text_transformations(text: &str, replacement: &str) -> String {
    let mut result = text.to_string();

    if replacement.contains("[lowercase]") {
        result = result.to_lowercase();
    }
    if replacement.contains("[uppercase]") {
        result = result.to_uppercase();
    }
    if replacement.contains("[capitalize]") {
        result = capitalize_words(&result);
    }
    if replacement.contains("[nospace]") {
        result = result.replace(' ', "");
    }

    result
}

/// Capitalize the first letter of each word.
#[allow(dead_code)]
fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Apply all enabled replacements to text in order.
pub fn apply_replacements(text: &str, replacements: &[TextReplacement]) -> String {
    let mut result = text.to_string();

    for replacement in replacements.iter().filter(|r| r.enabled) {
        let before = result.clone();
        result = replacement.apply(&result);
        if result != before {
            debug!(
                "Applied replacement '{}' -> '{}'",
                replacement.pattern, replacement.replacement
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_replacement() {
        let replacement = TextReplacement::new("hello".to_string(), "hi".to_string());
        assert_eq!(replacement.apply("hello world"), "hi world");
    }

    #[test]
    fn test_case_insensitive_replacement() {
        let mut replacement = TextReplacement::new("Hello".to_string(), "hi".to_string());
        replacement.case_insensitive = true;
        assert_eq!(replacement.apply("HELLO world"), "hi world");
        assert_eq!(replacement.apply("hello world"), "hi world");
    }

    #[test]
    fn test_regex_replacement() {
        let mut replacement = TextReplacement::new(r"\bfoo\b".to_string(), "bar".to_string());
        replacement.is_regex = true;
        assert_eq!(replacement.apply("foo is foo"), "bar is bar");
        assert_eq!(replacement.apply("foobar"), "foobar"); // Word boundary doesn't match
    }

    #[test]
    fn test_date_magic_command() {
        let replacement = TextReplacement::new("TODAY".to_string(), "[date]".to_string());
        let result = replacement.apply("Meeting TODAY");
        assert!(result.contains("-")); // Date format contains dashes
        assert!(!result.contains("TODAY"));
    }

    #[test]
    fn test_multiple_replacements() {
        let replacements = vec![
            TextReplacement::new("foo".to_string(), "bar".to_string()),
            TextReplacement::new("bar".to_string(), "baz".to_string()),
        ];
        let result = apply_replacements("foo", &replacements);
        assert_eq!(result, "baz"); // foo -> bar -> baz
    }

    #[test]
    fn test_disabled_replacement() {
        let mut replacement = TextReplacement::new("hello".to_string(), "hi".to_string());
        replacement.enabled = false;
        assert_eq!(replacement.apply("hello world"), "hello world");
    }
}
