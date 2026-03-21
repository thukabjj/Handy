//! Environmental Sound Detection
//!
//! Lightweight environmental sound detection based on audio heuristics.
//!
//! This implementation avoids heavyweight model dependencies while providing
//! practical category detection for alerting workflows.

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

/// Environmental sound detector using heuristic classifiers.
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
    pub fn detect_sounds(&self, samples: &[f32], sample_rate: u32) -> Vec<SoundEvent> {
        if !self.enabled || self.categories.is_empty() {
            return Vec::new();
        }
        if samples.is_empty() || sample_rate == 0 {
            return Vec::new();
        }

        let rms = rms(samples);
        let peak = peak(samples);
        let zcr = zero_crossing_rate(samples);
        let dominant_hz = estimate_dominant_frequency(samples, sample_rate);
        let pulse_density = pulse_density(samples, 0.6 * peak.max(0.02));

        let mut events = Vec::new();
        let base_threshold = self.threshold.clamp(0.05, 0.95);

        for category in &self.categories {
            let confidence = match category {
                SoundCategory::Alarm => score_alarm(rms, zcr, dominant_hz),
                SoundCategory::Siren => score_siren(rms, zcr, dominant_hz),
                SoundCategory::Doorbell => score_doorbell(rms, pulse_density, dominant_hz),
                SoundCategory::PhoneRing => score_phone_ring(rms, pulse_density, dominant_hz),
                SoundCategory::DogBark => score_dog_bark(peak, rms, pulse_density, dominant_hz),
                SoundCategory::BabyCry => score_baby_cry(rms, zcr, dominant_hz),
                SoundCategory::Knocking => score_knocking(peak, rms, pulse_density),
                SoundCategory::Applause => score_applause(rms, zcr, pulse_density),
            };

            if confidence >= base_threshold {
                events.push(SoundEvent {
                    category: category.clone(),
                    confidence,
                    timestamp_ms: 0,
                });
            }
        }

        events.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        events
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn rms(samples: &[f32]) -> f32 {
    let mean_square = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
    mean_square.sqrt()
}

fn peak(samples: &[f32]) -> f32 {
    samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f32, |acc, value| acc.max(value))
}

fn zero_crossing_rate(samples: &[f32]) -> f32 {
    if samples.len() < 2 {
        return 0.0;
    }
    let crossings = samples
        .windows(2)
        .filter(|w| (w[0] >= 0.0 && w[1] < 0.0) || (w[0] < 0.0 && w[1] >= 0.0))
        .count();
    crossings as f32 / samples.len() as f32
}

fn estimate_dominant_frequency(samples: &[f32], sample_rate: u32) -> f32 {
    if samples.len() < 128 {
        return 0.0;
    }

    let min_lag = (sample_rate / 2000).max(1) as usize;
    let max_lag = (sample_rate / 80).max(min_lag as u32 + 1) as usize;
    let mut best_lag = min_lag;
    let mut best_corr = f32::MIN;

    for lag in min_lag..=max_lag.min(samples.len() - 1) {
        let corr = samples
            .iter()
            .take(samples.len() - lag)
            .zip(samples.iter().skip(lag))
            .map(|(a, b)| a * b)
            .sum::<f32>();

        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }

    if best_lag == 0 {
        0.0
    } else {
        sample_rate as f32 / best_lag as f32
    }
}

fn pulse_density(samples: &[f32], threshold: f32) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let pulses = samples.iter().filter(|s| s.abs() >= threshold).count();
    pulses as f32 / samples.len() as f32
}

fn score_alarm(rms: f32, zcr: f32, freq: f32) -> f32 {
    clamp01((rms * 2.0) + (zcr * 2.0) + in_range(freq, 700.0, 1800.0, 0.5))
}

fn score_siren(rms: f32, zcr: f32, freq: f32) -> f32 {
    clamp01((rms * 1.8) + (zcr * 2.2) + in_range(freq, 400.0, 1400.0, 0.45))
}

fn score_doorbell(rms: f32, pulse_density: f32, freq: f32) -> f32 {
    clamp01((rms * 1.6) + (pulse_density * 2.2) + in_range(freq, 500.0, 1300.0, 0.4))
}

fn score_phone_ring(rms: f32, pulse_density: f32, freq: f32) -> f32 {
    clamp01((rms * 1.5) + (pulse_density * 2.4) + in_range(freq, 600.0, 2200.0, 0.4))
}

fn score_dog_bark(peak: f32, rms: f32, pulse_density: f32, freq: f32) -> f32 {
    let impulsiveness = (peak / (rms + 0.001)).min(6.0) / 6.0;
    clamp01((impulsiveness * 1.8) + (pulse_density * 1.5) + in_range(freq, 250.0, 900.0, 0.45))
}

fn score_baby_cry(rms: f32, zcr: f32, freq: f32) -> f32 {
    clamp01((rms * 1.6) + (zcr * 1.2) + in_range(freq, 300.0, 700.0, 0.55))
}

fn score_knocking(peak: f32, rms: f32, pulse_density: f32) -> f32 {
    let impulsiveness = (peak / (rms + 0.001)).min(6.0) / 6.0;
    clamp01((impulsiveness * 2.0) + (pulse_density * 1.8))
}

fn score_applause(rms: f32, zcr: f32, pulse_density: f32) -> f32 {
    clamp01((rms * 1.2) + (zcr * 2.8) + (pulse_density * 1.2))
}

fn in_range(value: f32, min: f32, max: f32, weight: f32) -> f32 {
    if value >= min && value <= max {
        weight
    } else {
        0.0
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
    fn test_detector_returns_candidates_when_enabled() {
        let mut detector = SoundDetector::new();
        detector.enabled = true;
        detector.categories = vec![SoundCategory::Doorbell];
        let events = detector.detect_sounds(&[0.3; 16000], 16000);
        assert!(events.len() <= 1);
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
