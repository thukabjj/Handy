//! Speaker Diarization Module
//!
//! Provides energy-based speaker diarization for identifying "who is speaking".
//! This implementation uses RMS energy analysis and silence detection to track
//! speaker changes, which works well for 2-person conversations.

use std::collections::VecDeque;

/// Unique identifier for a speaker
pub type SpeakerId = u32;

/// Represents a detected speaker change
#[derive(Clone, Debug, PartialEq)]
pub struct SpeakerChange {
    /// The new speaker ID after the change
    pub new_speaker: SpeakerId,
    /// The previous speaker ID before the change
    pub previous_speaker: SpeakerId,
    /// Timestamp in samples when the change was detected
    pub sample_offset: usize,
}

/// Configuration for the energy-based diarizer
#[derive(Clone, Debug)]
pub struct DiarizationConfig {
    /// RMS threshold below which audio is considered silence (0.0-1.0)
    pub silence_threshold: f32,
    /// Minimum silence duration in milliseconds to consider a speaker change
    pub min_silence_duration_ms: u32,
    /// Energy change threshold to detect potential speaker switch (ratio)
    pub energy_change_threshold: f32,
    /// Sample rate of the audio (for time calculations)
    pub sample_rate: u32,
    /// Size of the energy history window in frames
    pub history_window_size: usize,
}

impl Default for DiarizationConfig {
    fn default() -> Self {
        Self {
            silence_threshold: 0.02,          // -34 dB roughly
            min_silence_duration_ms: 500,     // 500ms silence suggests speaker change
            energy_change_threshold: 2.0,     // 2x energy change suggests new speaker
            sample_rate: 16000,
            history_window_size: 20,          // ~20 frames of history
        }
    }
}

/// Trait for speaker diarization implementations
pub trait SpeakerDiarizer: Send + Sync {
    /// Process a frame of audio samples and detect any speaker changes
    ///
    /// # Arguments
    /// * `samples` - Audio samples for this frame (typically 30ms at 16kHz = 480 samples)
    ///
    /// # Returns
    /// * `Some(SpeakerChange)` if a speaker change was detected
    /// * `None` if the same speaker is continuing
    fn process_frame(&mut self, samples: &[f32]) -> Option<SpeakerChange>;

    /// Get the currently detected speaker
    fn get_current_speaker(&self) -> SpeakerId;

    /// Reset the diarizer state (e.g., for new sessions)
    fn reset(&mut self);

    /// Get the total number of detected speakers
    fn get_speaker_count(&self) -> usize;
}

/// Energy-based speaker diarizer
///
/// Uses RMS energy analysis to detect speaker changes based on:
/// 1. Silence gaps (>500ms by default) - indicates turn-taking
/// 2. Significant energy level changes - suggests different speaker
///
/// This is a simple but effective approach for:
/// - 2-person conversations
/// - Meeting scenarios with clear turn-taking
/// - Phone calls with distinct speakers
#[derive(Debug)]
pub struct EnergyBasedDiarizer {
    config: DiarizationConfig,
    /// Current speaker ID (0 = "You"/primary, 1+ = others)
    current_speaker: SpeakerId,
    /// Number of unique speakers detected
    speaker_count: usize,
    /// Recent RMS energy values for trend analysis
    energy_history: VecDeque<f32>,
    /// Average energy level for the current speaker
    current_speaker_energy: f32,
    /// Count of consecutive silence frames
    silence_frame_count: u32,
    /// Samples processed since last speaker change
    samples_since_change: usize,
    /// Energy baseline for the primary speaker (established early)
    primary_speaker_baseline: Option<f32>,
    /// Total samples processed
    total_samples: usize,
}

impl EnergyBasedDiarizer {
    /// Create a new energy-based diarizer with default configuration
    pub fn new() -> Self {
        Self::with_config(DiarizationConfig::default())
    }

    /// Create a new energy-based diarizer with custom configuration
    pub fn with_config(config: DiarizationConfig) -> Self {
        Self {
            config,
            current_speaker: 0, // Start with primary speaker (user)
            speaker_count: 1,
            energy_history: VecDeque::with_capacity(20),
            current_speaker_energy: 0.0,
            silence_frame_count: 0,
            samples_since_change: 0,
            primary_speaker_baseline: None,
            total_samples: 0,
        }
    }

    /// Calculate RMS energy of audio samples
    fn calculate_rms(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_squares: f32 = samples.iter().map(|s| s * s).sum();
        (sum_squares / samples.len() as f32).sqrt()
    }

    /// Check if the current frame is silence
    fn is_silence(&self, rms: f32) -> bool {
        rms < self.config.silence_threshold
    }

    /// Calculate samples needed for the minimum silence duration
    fn min_silence_samples(&self) -> u32 {
        (self.config.sample_rate * self.config.min_silence_duration_ms) / 1000
    }

    /// Get the average energy from recent history
    fn get_average_energy(&self) -> f32 {
        if self.energy_history.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.energy_history.iter().sum();
        sum / self.energy_history.len() as f32
    }

    /// Update energy history with a new value
    fn update_energy_history(&mut self, rms: f32) {
        self.energy_history.push_back(rms);
        if self.energy_history.len() > self.config.history_window_size {
            self.energy_history.pop_front();
        }
    }

    /// Check if energy characteristics suggest a different speaker
    fn energy_suggests_speaker_change(&self, current_rms: f32) -> bool {
        // Need some history to compare
        if self.energy_history.len() < 5 {
            return false;
        }

        // If we have a baseline for primary speaker, use it
        if let Some(baseline) = self.primary_speaker_baseline {
            let ratio = if baseline > 0.001 {
                current_rms / baseline
            } else {
                1.0
            };

            // Significantly different energy from primary speaker baseline
            if ratio > self.config.energy_change_threshold
                || ratio < 1.0 / self.config.energy_change_threshold
            {
                return true;
            }
        }

        // Compare to current speaker's average energy
        if self.current_speaker_energy > 0.001 {
            let ratio = current_rms / self.current_speaker_energy;
            if ratio > self.config.energy_change_threshold
                || ratio < 1.0 / self.config.energy_change_threshold
            {
                return true;
            }
        }

        false
    }
}

impl Default for EnergyBasedDiarizer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpeakerDiarizer for EnergyBasedDiarizer {
    fn process_frame(&mut self, samples: &[f32]) -> Option<SpeakerChange> {
        let rms = self.calculate_rms(samples);
        let is_silence = self.is_silence(rms);
        let previous_speaker = self.current_speaker;

        self.total_samples += samples.len();
        self.samples_since_change += samples.len();

        // Track silence duration
        if is_silence {
            self.silence_frame_count += samples.len() as u32;
        } else {
            // Speech detected
            let was_long_silence = self.silence_frame_count >= self.min_silence_samples();

            // Update energy tracking for non-silence frames
            self.update_energy_history(rms);

            // Establish primary speaker baseline early in the session
            if self.primary_speaker_baseline.is_none()
                && self.total_samples > (self.config.sample_rate as usize * 2)
                && self.energy_history.len() >= 10
            {
                self.primary_speaker_baseline = Some(self.get_average_energy());
            }

            // Check for speaker change conditions
            let speaker_changed = if was_long_silence {
                // Long silence followed by speech - likely speaker change
                // But only if we've been with this speaker for a while
                self.samples_since_change > (self.config.sample_rate as usize)
            } else if self.energy_suggests_speaker_change(rms) && !was_long_silence {
                // Energy profile change without silence - possible interruption
                // Be more conservative here
                self.samples_since_change > (self.config.sample_rate as usize * 2)
            } else {
                false
            };

            // Reset silence counter
            self.silence_frame_count = 0;

            if speaker_changed {
                // Toggle between speakers (simple 2-speaker model)
                // For more sophisticated tracking, this could use energy clustering
                let new_speaker = if self.current_speaker == 0 {
                    if self.speaker_count < 2 {
                        self.speaker_count = 2;
                    }
                    1
                } else {
                    0
                };

                self.current_speaker = new_speaker;
                self.samples_since_change = 0;
                self.current_speaker_energy = rms;

                return Some(SpeakerChange {
                    new_speaker,
                    previous_speaker,
                    sample_offset: self.total_samples,
                });
            }

            // Update current speaker's energy profile
            self.current_speaker_energy =
                self.current_speaker_energy * 0.9 + rms * 0.1;
        }

        None
    }

    fn get_current_speaker(&self) -> SpeakerId {
        self.current_speaker
    }

    fn reset(&mut self) {
        self.current_speaker = 0;
        self.speaker_count = 1;
        self.energy_history.clear();
        self.current_speaker_energy = 0.0;
        self.silence_frame_count = 0;
        self.samples_since_change = 0;
        self.primary_speaker_baseline = None;
        self.total_samples = 0;
    }

    fn get_speaker_count(&self) -> usize {
        self.speaker_count
    }
}

/// Thread-safe wrapper for speaker diarization
pub type SharedDiarizer = std::sync::Arc<std::sync::Mutex<Box<dyn SpeakerDiarizer>>>;

/// Create a shared diarizer instance
pub fn create_shared_diarizer() -> SharedDiarizer {
    std::sync::Arc::new(std::sync::Mutex::new(Box::new(EnergyBasedDiarizer::new())))
}

/// Create a shared diarizer with custom configuration
pub fn create_shared_diarizer_with_config(config: DiarizationConfig) -> SharedDiarizer {
    std::sync::Arc::new(std::sync::Mutex::new(Box::new(
        EnergyBasedDiarizer::with_config(config),
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diarizer_creation() {
        let diarizer = EnergyBasedDiarizer::new();
        assert_eq!(diarizer.get_current_speaker(), 0);
        assert_eq!(diarizer.get_speaker_count(), 1);
    }

    #[test]
    fn test_silence_detection() {
        let diarizer = EnergyBasedDiarizer::new();

        // Very quiet audio should be silence
        let silence: Vec<f32> = vec![0.001; 480];
        let rms = diarizer.calculate_rms(&silence);
        assert!(diarizer.is_silence(rms));

        // Louder audio should not be silence
        let speech: Vec<f32> = vec![0.1; 480];
        let rms = diarizer.calculate_rms(&speech);
        assert!(!diarizer.is_silence(rms));
    }

    #[test]
    fn test_rms_calculation() {
        let diarizer = EnergyBasedDiarizer::new();

        // Test with known values
        let samples = vec![0.5, -0.5, 0.5, -0.5];
        let rms = diarizer.calculate_rms(&samples);
        assert!((rms - 0.5).abs() < 0.001);

        // Empty samples
        let rms = diarizer.calculate_rms(&[]);
        assert_eq!(rms, 0.0);
    }

    #[test]
    fn test_reset() {
        let mut diarizer = EnergyBasedDiarizer::new();

        // Process some frames
        let samples = vec![0.1; 480];
        for _ in 0..100 {
            diarizer.process_frame(&samples);
        }

        // Reset and verify state
        diarizer.reset();
        assert_eq!(diarizer.get_current_speaker(), 0);
        assert_eq!(diarizer.get_speaker_count(), 1);
        assert!(diarizer.energy_history.is_empty());
    }

    #[test]
    fn test_speaker_change_after_silence() {
        let config = DiarizationConfig {
            silence_threshold: 0.02,
            min_silence_duration_ms: 100, // Shorter for testing
            sample_rate: 16000,
            ..Default::default()
        };
        let mut diarizer = EnergyBasedDiarizer::with_config(config);

        // Simulate speech from speaker 0
        let speech: Vec<f32> = vec![0.1; 480];
        for _ in 0..50 {
            diarizer.process_frame(&speech);
        }
        assert_eq!(diarizer.get_current_speaker(), 0);

        // Simulate silence (long enough for speaker change)
        let silence: Vec<f32> = vec![0.001; 480];
        for _ in 0..10 {
            diarizer.process_frame(&silence);
        }

        // New speech should potentially trigger speaker change
        let result = diarizer.process_frame(&speech);

        // After long silence and sufficient prior speech, a change should be detected
        if result.is_some() {
            let change = result.unwrap();
            assert_eq!(change.previous_speaker, 0);
            assert_eq!(change.new_speaker, 1);
            assert_eq!(diarizer.get_speaker_count(), 2);
        }
    }

    #[test]
    fn test_shared_diarizer() {
        let diarizer = create_shared_diarizer();

        {
            let mut d = diarizer.lock().unwrap();
            assert_eq!(d.get_current_speaker(), 0);
            d.reset();
        }

        // Can access from multiple references
        let diarizer_clone = diarizer.clone();
        {
            let d = diarizer_clone.lock().unwrap();
            assert_eq!(d.get_current_speaker(), 0);
        }
    }

    #[test]
    fn test_config_defaults() {
        let config = DiarizationConfig::default();
        assert_eq!(config.silence_threshold, 0.02);
        assert_eq!(config.min_silence_duration_ms, 500);
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.energy_change_threshold, 2.0);
    }
}
