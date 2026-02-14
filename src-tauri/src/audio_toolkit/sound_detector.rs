//! Environmental Sound Detection
//!
//! Stub framework for environmental sound detection using a YAMNet-style model.
//! The actual ONNX inference is a placeholder that returns an empty Vec.
//! Dropping in a real model later requires ~20 lines of change in `detect_sounds`.

use crate::settings::sound_detection::{SoundCategory, SoundDetectionSettings};
use serde::{Deserialize, Serialize};
use specta::Type;

/// A detected sound event
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SoundEvent {
    pub category: SoundCategory,
    pub confidence: f32,
    pub timestamp_ms: u64,
}

/// Environmental sound detector with stub inference
pub struct SoundDetector {
    enabled: bool,
    threshold: f32,
    categories: Vec<SoundCategory>,
}

impl SoundDetector {
    /// Create a new disabled SoundDetector with no categories
    pub fn new() -> Self {
        Self {
            enabled: false,
            threshold: 0.5,
            categories: Vec::new(),
        }
    }

    /// Sync detector state from application settings
    pub fn update_settings(&mut self, settings: &SoundDetectionSettings) {
        self.enabled = settings.enabled;
        self.threshold = settings.threshold;
        self.categories = settings.categories.clone();
    }

    /// Detect environmental sounds in audio samples.
    ///
    /// This is a stub implementation that always returns an empty Vec.
    /// To integrate a real YAMNet ONNX model, replace the body of this
    /// method with actual inference logic (~20 lines).
    pub fn detect_sounds(&self, _samples: &[f32], _sample_rate: u32) -> Vec<SoundEvent> {
        if !self.enabled || self.categories.is_empty() {
            return Vec::new();
        }

        // STUB: Real implementation would:
        // 1. Resample _samples to 16kHz if needed
        // 2. Run ONNX inference on the audio frame
        // 3. Map class indices to SoundCategory
        // 4. Filter by self.categories and self.threshold
        // 5. Return matching SoundEvent entries

        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_detector_is_disabled() {
        let detector = SoundDetector::new();
        assert!(!detector.enabled);
        assert!(detector.categories.is_empty());
    }

    #[test]
    fn test_stub_returns_empty() {
        let mut detector = SoundDetector::new();
        detector.enabled = true;
        detector.categories = vec![SoundCategory::Doorbell];
        let events = detector.detect_sounds(&[0.0; 16000], 16000);
        assert!(events.is_empty());
    }

    #[test]
    fn test_disabled_returns_empty() {
        let detector = SoundDetector::new();
        let events = detector.detect_sounds(&[0.0; 16000], 16000);
        assert!(events.is_empty());
    }

    #[test]
    fn test_update_settings() {
        let mut detector = SoundDetector::new();
        let settings = SoundDetectionSettings {
            enabled: true,
            categories: vec![SoundCategory::Alarm, SoundCategory::Siren],
            threshold: 0.7,
            notification_enabled: true,
        };
        detector.update_settings(&settings);
        assert!(detector.enabled);
        assert_eq!(detector.threshold, 0.7);
        assert_eq!(detector.categories.len(), 2);
    }
}
