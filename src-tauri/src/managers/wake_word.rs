//! Wake-word detection for triggering Active Listening.
//!
//! This module provides infrastructure for wake-word detection that can trigger
//! Active Listening sessions. The actual detection uses simple keyword matching
//! from transcribed audio segments, which works with the existing transcription
//! pipeline without requiring additional ONNX models.
//!
//! Future improvements could add:
//! - ONNX-based wake-word models (openWakeWord, etc.)
//! - Embedded keyword spotting for lower latency
//! - Custom wake-word training

use log::{debug, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Action to perform when wake-word is detected.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WakeWordAction {
    /// Start an Active Listening session
    StartActiveListening,
    /// Start standard transcription
    StartRecording,
    /// No action (disabled)
    None,
}

impl Default for WakeWordAction {
    fn default() -> Self {
        Self::StartActiveListening
    }
}

/// Wake-word detection settings.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeWordSettings {
    /// Whether wake-word detection is enabled
    pub enabled: bool,
    /// The wake phrase to listen for (e.g., "Hey Handy")
    pub wake_phrase: String,
    /// Case-insensitive matching
    pub case_insensitive: bool,
    /// Action to perform when wake-word is detected
    pub trigger_action: WakeWordAction,
    /// Minimum confidence threshold (0.0 - 1.0)
    /// For transcript matching, this is fuzzy match threshold
    pub detection_threshold: f32,
    /// Cooldown period in seconds between detections
    pub cooldown_seconds: u32,
}

impl Default for WakeWordSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            wake_phrase: "Hey Handy".to_string(),
            case_insensitive: true,
            trigger_action: WakeWordAction::StartActiveListening,
            detection_threshold: 0.8,
            cooldown_seconds: 5,
        }
    }
}

/// Wake-word detector using transcript matching.
/// This is a simple implementation that looks for the wake phrase
/// in transcribed audio segments.
#[allow(dead_code)]
pub struct WakeWordDetector {
    settings: WakeWordSettings,
    is_enabled: Arc<AtomicBool>,
    last_detection_time: std::sync::Mutex<std::time::Instant>,
}

#[allow(dead_code)]
impl WakeWordDetector {
    /// Create a new wake-word detector.
    pub fn new(settings: WakeWordSettings) -> Self {
        Self {
            is_enabled: Arc::new(AtomicBool::new(settings.enabled)),
            settings,
            last_detection_time: std::sync::Mutex::new(
                std::time::Instant::now()
                    .checked_sub(std::time::Duration::from_secs(60))
                    .unwrap_or_else(std::time::Instant::now),
            ),
        }
    }

    /// Update the detector settings.
    pub fn update_settings(&mut self, settings: WakeWordSettings) {
        self.is_enabled.store(settings.enabled, Ordering::Relaxed);
        self.settings = settings;
    }

    /// Check if wake-word detection is enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::Relaxed)
    }

    /// Enable or disable wake-word detection.
    pub fn set_enabled(&self, enabled: bool) {
        self.is_enabled.store(enabled, Ordering::Relaxed);
        info!(
            "Wake-word detection {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if the given transcript contains the wake phrase.
    /// Returns true if detected and cooldown has passed.
    pub fn detect_in_transcript(&self, transcript: &str) -> bool {
        if !self.is_enabled() {
            return false;
        }

        // Check cooldown
        let now = std::time::Instant::now();
        {
            let last = self.last_detection_time.lock().unwrap();
            let cooldown = std::time::Duration::from_secs(self.settings.cooldown_seconds as u64);
            if now.duration_since(*last) < cooldown {
                debug!("Wake-word cooldown active, ignoring detection");
                return false;
            }
        }

        // Perform matching
        let detected = if self.settings.case_insensitive {
            transcript
                .to_lowercase()
                .contains(&self.settings.wake_phrase.to_lowercase())
        } else {
            transcript.contains(&self.settings.wake_phrase)
        };

        if detected {
            info!(
                "Wake-word '{}' detected in transcript",
                self.settings.wake_phrase
            );
            // Update last detection time
            if let Ok(mut last) = self.last_detection_time.lock() {
                *last = now;
            }
        }

        detected
    }

    /// Get the action to perform when wake-word is detected.
    pub fn get_trigger_action(&self) -> WakeWordAction {
        self.settings.trigger_action.clone()
    }

    /// Get the current settings.
    pub fn get_settings(&self) -> &WakeWordSettings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wake_word_detection() {
        let settings = WakeWordSettings {
            enabled: true,
            wake_phrase: "Hey Handy".to_string(),
            case_insensitive: true,
            cooldown_seconds: 0, // No cooldown for tests
            ..Default::default()
        };

        let detector = WakeWordDetector::new(settings);

        assert!(detector.detect_in_transcript("Hey Handy, start listening"));
        assert!(detector.detect_in_transcript("hey handy what's up"));
        assert!(detector.detect_in_transcript("HEY HANDY"));
        assert!(!detector.detect_in_transcript("Hello there"));
        assert!(!detector.detect_in_transcript("Handy hey"));
    }

    #[test]
    fn test_wake_word_disabled() {
        let settings = WakeWordSettings {
            enabled: false,
            wake_phrase: "Hey Handy".to_string(),
            ..Default::default()
        };

        let detector = WakeWordDetector::new(settings);
        assert!(!detector.detect_in_transcript("Hey Handy"));
    }

    #[test]
    fn test_case_sensitive_matching() {
        let settings = WakeWordSettings {
            enabled: true,
            wake_phrase: "Hey Handy".to_string(),
            case_insensitive: false,
            cooldown_seconds: 0,
            ..Default::default()
        };

        let detector = WakeWordDetector::new(settings);

        assert!(detector.detect_in_transcript("Hey Handy"));
        assert!(!detector.detect_in_transcript("hey handy"));
    }
}
