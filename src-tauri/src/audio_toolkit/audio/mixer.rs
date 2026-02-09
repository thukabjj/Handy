//! Audio mixer for combining multiple audio sources.
//!
//! This module provides functionality to mix audio from multiple sources
//! (e.g., microphone and system audio) into a single stream for transcription.
//!
//! ## Features
//!
//! - Mix ratio control (0.0 = source A only, 1.0 = source B only)
//! - Automatic level normalization to prevent clipping
//! - Thread-safe buffer management
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::audio_toolkit::audio::mixer::AudioMixer;
//!
//! let mut mixer = AudioMixer::new(0.5); // 50/50 mix
//!
//! // Push samples from different sources
//! mixer.push_mic(&mic_samples);
//! mixer.push_system(&system_samples);
//!
//! // Get mixed output
//! let mixed = mixer.mix();
//! ```

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Default buffer size in samples (roughly 100ms at 16kHz)
const DEFAULT_BUFFER_SIZE: usize = 1600;

/// Maximum buffer size to prevent memory issues (roughly 5 seconds at 16kHz)
const MAX_BUFFER_SIZE: usize = 80000;

/// Audio mixer for combining two audio sources
pub struct AudioMixer {
    /// Buffer for microphone audio samples
    mic_buffer: VecDeque<f32>,
    /// Buffer for system audio samples
    system_buffer: VecDeque<f32>,
    /// Mix ratio: 0.0 = mic only, 1.0 = system only, 0.5 = equal mix
    mix_ratio: f32,
    /// Whether to normalize output to prevent clipping
    normalize: bool,
}

impl AudioMixer {
    /// Create a new audio mixer with the specified mix ratio
    ///
    /// # Arguments
    ///
    /// * `mix_ratio` - Mix balance: 0.0 = microphone only, 1.0 = system audio only
    pub fn new(mix_ratio: f32) -> Self {
        Self {
            mic_buffer: VecDeque::with_capacity(DEFAULT_BUFFER_SIZE),
            system_buffer: VecDeque::with_capacity(DEFAULT_BUFFER_SIZE),
            mix_ratio: mix_ratio.clamp(0.0, 1.0),
            normalize: true,
        }
    }

    /// Set the mix ratio
    ///
    /// # Arguments
    ///
    /// * `ratio` - Mix balance: 0.0 = microphone only, 1.0 = system audio only
    pub fn set_mix_ratio(&mut self, ratio: f32) {
        self.mix_ratio = ratio.clamp(0.0, 1.0);
    }

    /// Get the current mix ratio
    pub fn mix_ratio(&self) -> f32 {
        self.mix_ratio
    }

    /// Enable or disable output normalization
    pub fn set_normalize(&mut self, normalize: bool) {
        self.normalize = normalize;
    }

    /// Push microphone audio samples to the buffer
    pub fn push_mic(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.mic_buffer.push_back(sample);
        }

        // Prevent buffer overflow
        while self.mic_buffer.len() > MAX_BUFFER_SIZE {
            self.mic_buffer.pop_front();
        }
    }

    /// Push system audio samples to the buffer
    pub fn push_system(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.system_buffer.push_back(sample);
        }

        // Prevent buffer overflow
        while self.system_buffer.len() > MAX_BUFFER_SIZE {
            self.system_buffer.pop_front();
        }
    }

    /// Get the number of samples available for mixing
    ///
    /// Returns the minimum of both buffer sizes, since we need samples from both
    /// to produce a proper mix.
    pub fn available_samples(&self) -> usize {
        self.mic_buffer.len().min(self.system_buffer.len())
    }

    /// Mix available samples and return the result
    ///
    /// This will consume samples from both buffers up to the available count.
    /// If one source has more samples than the other, the excess remains buffered.
    pub fn mix(&mut self) -> Vec<f32> {
        let count = self.available_samples();
        if count == 0 {
            return Vec::new();
        }

        let mut output = Vec::with_capacity(count);
        let mic_weight = 1.0 - self.mix_ratio;
        let system_weight = self.mix_ratio;

        for _ in 0..count {
            let mic_sample = self.mic_buffer.pop_front().unwrap_or(0.0);
            let system_sample = self.system_buffer.pop_front().unwrap_or(0.0);

            // Weighted mix
            let mixed = mic_sample * mic_weight + system_sample * system_weight;
            output.push(mixed);
        }

        // Normalize if enabled
        if self.normalize && !output.is_empty() {
            self.normalize_samples(&mut output);
        }

        output
    }

    /// Mix and drain all available samples from a specific source
    ///
    /// Use this when only one source is active (e.g., microphone-only mode)
    pub fn drain_mic(&mut self) -> Vec<f32> {
        let mut output: Vec<f32> = self.mic_buffer.drain(..).collect();
        if self.normalize && !output.is_empty() {
            self.normalize_samples(&mut output);
        }
        output
    }

    /// Mix and drain all available samples from system audio
    ///
    /// Use this when only system audio is active
    pub fn drain_system(&mut self) -> Vec<f32> {
        let mut output: Vec<f32> = self.system_buffer.drain(..).collect();
        if self.normalize && !output.is_empty() {
            self.normalize_samples(&mut output);
        }
        output
    }

    /// Clear all buffers
    pub fn clear(&mut self) {
        self.mic_buffer.clear();
        self.system_buffer.clear();
    }

    /// Normalize samples to prevent clipping
    fn normalize_samples(&self, samples: &mut [f32]) {
        // Find the maximum absolute value
        let max_abs = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);

        // If max is above 1.0, normalize to prevent clipping
        if max_abs > 1.0 {
            let scale = 1.0 / max_abs;
            for sample in samples.iter_mut() {
                *sample *= scale;
            }
        }
    }
}

impl Default for AudioMixer {
    fn default() -> Self {
        Self::new(0.5) // Equal mix by default
    }
}

/// Thread-safe wrapper around AudioMixer
pub struct SharedAudioMixer {
    inner: Arc<Mutex<AudioMixer>>,
}

impl SharedAudioMixer {
    /// Create a new shared audio mixer
    pub fn new(mix_ratio: f32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioMixer::new(mix_ratio))),
        }
    }

    /// Push microphone samples (thread-safe)
    pub fn push_mic(&self, samples: &[f32]) {
        if let Ok(mut mixer) = self.inner.lock() {
            mixer.push_mic(samples);
        }
    }

    /// Push system audio samples (thread-safe)
    pub fn push_system(&self, samples: &[f32]) {
        if let Ok(mut mixer) = self.inner.lock() {
            mixer.push_system(samples);
        }
    }

    /// Set mix ratio (thread-safe)
    pub fn set_mix_ratio(&self, ratio: f32) {
        if let Ok(mut mixer) = self.inner.lock() {
            mixer.set_mix_ratio(ratio);
        }
    }

    /// Get mixed samples (thread-safe)
    pub fn mix(&self) -> Vec<f32> {
        if let Ok(mut mixer) = self.inner.lock() {
            mixer.mix()
        } else {
            Vec::new()
        }
    }

    /// Clear all buffers (thread-safe)
    pub fn clear(&self) {
        if let Ok(mut mixer) = self.inner.lock() {
            mixer.clear();
        }
    }

    /// Clone the inner Arc for sharing across threads
    pub fn clone_inner(&self) -> Arc<Mutex<AudioMixer>> {
        self.inner.clone()
    }
}

impl Clone for SharedAudioMixer {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl Default for SharedAudioMixer {
    fn default() -> Self {
        Self::new(0.5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mixer_creation() {
        let mixer = AudioMixer::new(0.5);
        assert_eq!(mixer.mix_ratio(), 0.5);
    }

    #[test]
    fn test_mix_ratio_clamping() {
        let mut mixer = AudioMixer::new(1.5);
        assert_eq!(mixer.mix_ratio(), 1.0);

        mixer.set_mix_ratio(-0.5);
        assert_eq!(mixer.mix_ratio(), 0.0);
    }

    #[test]
    fn test_push_and_mix() {
        let mut mixer = AudioMixer::new(0.5);

        let mic_samples = vec![0.5, 0.5, 0.5];
        let system_samples = vec![0.3, 0.3, 0.3];

        mixer.push_mic(&mic_samples);
        mixer.push_system(&system_samples);

        let mixed = mixer.mix();
        assert_eq!(mixed.len(), 3);

        // With 0.5 ratio: (0.5 * 0.5) + (0.3 * 0.5) = 0.4
        for sample in mixed {
            assert!((sample - 0.4).abs() < 0.001);
        }
    }

    #[test]
    fn test_mic_only_mix() {
        let mut mixer = AudioMixer::new(0.0); // Mic only

        let mic_samples = vec![0.5, 0.5, 0.5];
        let system_samples = vec![0.3, 0.3, 0.3];

        mixer.push_mic(&mic_samples);
        mixer.push_system(&system_samples);

        let mixed = mixer.mix();

        // With 0.0 ratio: (0.5 * 1.0) + (0.3 * 0.0) = 0.5
        for sample in mixed {
            assert!((sample - 0.5).abs() < 0.001);
        }
    }

    #[test]
    fn test_system_only_mix() {
        let mut mixer = AudioMixer::new(1.0); // System only

        let mic_samples = vec![0.5, 0.5, 0.5];
        let system_samples = vec![0.3, 0.3, 0.3];

        mixer.push_mic(&mic_samples);
        mixer.push_system(&system_samples);

        let mixed = mixer.mix();

        // With 1.0 ratio: (0.5 * 0.0) + (0.3 * 1.0) = 0.3
        for sample in mixed {
            assert!((sample - 0.3).abs() < 0.001);
        }
    }

    #[test]
    fn test_normalization() {
        let mut mixer = AudioMixer::new(0.5);
        mixer.set_normalize(true);

        // Push samples that would clip when mixed
        let loud_samples = vec![0.8, 0.9, 1.0];
        mixer.push_mic(&loud_samples);
        mixer.push_system(&loud_samples);

        let mixed = mixer.mix();

        // All samples should be <= 1.0
        for sample in mixed {
            assert!(sample.abs() <= 1.0);
        }
    }

    #[test]
    fn test_unequal_buffers() {
        let mut mixer = AudioMixer::new(0.5);

        mixer.push_mic(&[0.5, 0.5, 0.5, 0.5, 0.5]);
        mixer.push_system(&[0.3, 0.3, 0.3]);

        // Should only mix the available samples from both
        assert_eq!(mixer.available_samples(), 3);

        let mixed = mixer.mix();
        assert_eq!(mixed.len(), 3);

        // Remaining mic samples should still be in buffer
        assert_eq!(mixer.available_samples(), 0);
        assert_eq!(mixer.mic_buffer.len(), 2);
    }

    #[test]
    fn test_drain_mic() {
        let mut mixer = AudioMixer::new(0.5);

        mixer.push_mic(&[0.5, 0.5, 0.5]);
        let drained = mixer.drain_mic();

        assert_eq!(drained.len(), 3);
        assert_eq!(mixer.mic_buffer.len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut mixer = AudioMixer::new(0.5);

        mixer.push_mic(&[0.5, 0.5, 0.5]);
        mixer.push_system(&[0.3, 0.3, 0.3]);

        mixer.clear();

        assert_eq!(mixer.mic_buffer.len(), 0);
        assert_eq!(mixer.system_buffer.len(), 0);
    }

    #[test]
    fn test_shared_mixer() {
        let mixer = SharedAudioMixer::new(0.5);

        mixer.push_mic(&[0.5, 0.5, 0.5]);
        mixer.push_system(&[0.3, 0.3, 0.3]);

        let mixed = mixer.mix();
        assert_eq!(mixed.len(), 3);
    }

    #[test]
    fn test_shared_mixer_clone() {
        let mixer1 = SharedAudioMixer::new(0.5);
        let mixer2 = mixer1.clone();

        mixer1.push_mic(&[0.5, 0.5, 0.5]);

        // Both should see the same data since they share the inner Arc
        mixer2.push_system(&[0.3, 0.3, 0.3]);

        let mixed = mixer1.mix();
        assert_eq!(mixed.len(), 3);
    }
}
