use serde::{Deserialize, Serialize};
use specta::Type;

/// Categories of errors that can occur in the application
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Settings,
    Audio,
    Model,
    Transcription,
    Network,
    Validation,
    State,
    Filesystem,
    Permission,
    Unknown,
}

/// A structured error type for the Dictum application
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HandyError {
    pub category: ErrorCategory,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

impl HandyError {
    /// Create a new HandyError with the specified category and message
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            details: None,
            recoverable: false,
            suggestion: None,
        }
    }

    // Category-specific constructors

    /// Create a settings-related error
    pub fn settings(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Settings, message)
    }

    /// Create an audio-related error
    pub fn audio(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Audio, message)
    }

    /// Create a model-related error
    pub fn model(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Model, message)
    }

    /// Create a transcription-related error
    pub fn transcription(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Transcription, message)
    }

    /// Create a network-related error
    pub fn network(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Network, message)
    }

    /// Create a validation-related error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Validation, message)
    }

    /// Create a state-related error (e.g., mutex poisoning)
    pub fn state(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::State, message)
    }

    /// Create a filesystem-related error
    pub fn filesystem(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Filesystem, message)
    }

    /// Create a permission-related error
    pub fn permission(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Permission, message)
    }

    /// Create an unknown/generic error
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::new(ErrorCategory::Unknown, message)
    }

    // Builder methods

    /// Add details to the error
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// Mark the error as recoverable
    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    /// Add a suggestion for how to resolve the error
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for HandyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(details) = &self.details {
            write!(f, ": {}", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for HandyError {}

// Conversion from String for backward compatibility
impl From<String> for HandyError {
    fn from(message: String) -> Self {
        Self::unknown(message)
    }
}

impl From<&str> for HandyError {
    fn from(message: &str) -> Self {
        Self::unknown(message)
    }
}

// Common error conversions
impl From<std::io::Error> for HandyError {
    fn from(err: std::io::Error) -> Self {
        Self::filesystem("I/O error").with_details(err.to_string())
    }
}

impl From<serde_json::Error> for HandyError {
    fn from(err: serde_json::Error) -> Self {
        Self::validation("JSON serialization error").with_details(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = HandyError::audio("Microphone not found")
            .with_details("No audio input devices detected")
            .recoverable()
            .with_suggestion("Please connect a microphone and try again");

        assert_eq!(error.category, ErrorCategory::Audio);
        assert_eq!(error.message, "Microphone not found");
        assert_eq!(
            error.details,
            Some("No audio input devices detected".to_string())
        );
        assert!(error.recoverable);
        assert_eq!(
            error.suggestion,
            Some("Please connect a microphone and try again".to_string())
        );
    }

    #[test]
    fn test_error_display() {
        let error = HandyError::settings("Failed to save settings")
            .with_details("Permission denied");

        assert_eq!(
            format!("{}", error),
            "Failed to save settings: Permission denied"
        );
    }

    #[test]
    fn test_error_from_string() {
        let error: HandyError = "Something went wrong".into();
        assert_eq!(error.category, ErrorCategory::Unknown);
        assert_eq!(error.message, "Something went wrong");
    }
}
