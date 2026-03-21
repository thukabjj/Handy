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

/// Voice authentication mode for wake-word detection.
#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum VoiceAuthMode {
    /// Disable speaker verification.
    #[default]
    Disabled,
    /// Only accept wake phrase when it matches the enrolled owner voice.
    OwnerOnly,
}

/// Lightweight voice profile computed from enrollment samples.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeVoiceProfile {
    /// Mean feature vector for the enrolled speaker.
    pub centroid: Vec<f32>,
    /// Number of samples used to build the profile.
    pub sample_count: u32,
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
    /// Enable speaker verification for owner-only wake word.
    #[serde(default)]
    pub voice_auth_enabled: bool,
    /// Voice authentication strategy.
    #[serde(default)]
    pub voice_auth_mode: VoiceAuthMode,
    /// Minimum similarity threshold for speaker verification (0.0 - 1.0).
    #[serde(default = "default_speaker_threshold")]
    pub speaker_threshold: f32,
    /// Minimum audio duration required for speaker verification.
    #[serde(default = "default_quality_min_duration_ms")]
    pub quality_min_duration_ms: u32,
    /// Optional enrolled profile for speaker verification.
    #[serde(default)]
    pub voice_profile: Option<WakeVoiceProfile>,
    /// KWS acceptance threshold (0.0 - 1.0).
    #[serde(default = "default_kws_threshold")]
    pub kws_threshold: f32,
    /// Anti-spoof threshold (0.0 - 1.0). Used only when voice auth is enabled.
    #[serde(default = "default_spoof_threshold")]
    pub spoof_threshold: f32,
    /// Enable simple voice activity gating before wake detection.
    #[serde(default = "default_vad_enabled")]
    pub vad_enabled: bool,
    /// Target false accepts per hour used for calibration suggestions.
    #[serde(default = "default_target_far_per_hour")]
    pub target_far_per_hour: f32,
    /// If true, owner-only mode falls back to phrase-only detection when no profile is enrolled.
    #[serde(default = "default_fallback_when_no_profile")]
    pub fallback_when_no_profile: bool,
}

fn default_speaker_threshold() -> f32 {
    0.80
}

fn default_quality_min_duration_ms() -> u32 {
    600
}

fn default_kws_threshold() -> f32 {
    0.70
}

fn default_spoof_threshold() -> f32 {
    0.55
}

fn default_vad_enabled() -> bool {
    true
}

fn default_target_far_per_hour() -> f32 {
    0.10
}

fn default_fallback_when_no_profile() -> bool {
    true
}

impl Default for WakeWordSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            wake_phrase: "Hey Handy".to_string(),
            case_insensitive: true,
            trigger_action: WakeWordAction::StartActiveListening,
            detection_threshold: 0.7,
            cooldown_seconds: 5,
            voice_auth_enabled: false,
            voice_auth_mode: VoiceAuthMode::Disabled,
            speaker_threshold: default_speaker_threshold(),
            quality_min_duration_ms: default_quality_min_duration_ms(),
            voice_profile: None,
            kws_threshold: default_kws_threshold(),
            spoof_threshold: default_spoof_threshold(),
            vad_enabled: default_vad_enabled(),
            target_far_per_hour: default_target_far_per_hour(),
            fallback_when_no_profile: default_fallback_when_no_profile(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WakeRejectionReason {
    Disabled,
    CooldownActive,
    PhraseNotDetected,
    MissingVoiceProfile,
    AudioTooShort,
    LowSignal,
    SpoofSuspected,
    SpeakerMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeDetectionResult {
    pub detected: bool,
    pub phrase_detected: bool,
    pub phrase_score: f32,
    pub kws_score: f32,
    pub speaker_score: Option<f32>,
    pub spoof_score: Option<f32>,
    pub rejection_reason: Option<WakeRejectionReason>,
}

/// Wake-word detector using transcript matching.
/// This is a simple implementation that looks for the wake phrase
/// in transcribed audio segments.
pub struct WakeWordDetector {
    settings: WakeWordSettings,
    is_enabled: Arc<AtomicBool>,
    last_detection_time: std::sync::Mutex<std::time::Instant>,
}

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
    #[allow(dead_code)]
    pub fn set_enabled(&self, enabled: bool) {
        self.is_enabled.store(enabled, Ordering::Relaxed);
        info!(
            "Wake-word detection {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if the given transcript contains the wake phrase.
    /// Returns true if detected and cooldown has passed.
    pub fn detect(&self, transcript: &str, samples: Option<&[f32]>) -> WakeDetectionResult {
        info!(
            "[wake-detector] detect invoked enabled={} phrase='{}' transcript='{}'",
            self.is_enabled(),
            self.settings.wake_phrase,
            truncate_for_log(transcript, 120)
        );
        if !self.is_enabled() {
            return WakeDetectionResult {
                detected: false,
                phrase_detected: false,
                phrase_score: 0.0,
                kws_score: 0.0,
                speaker_score: None,
                spoof_score: None,
                rejection_reason: Some(WakeRejectionReason::Disabled),
            };
        }

        // Check cooldown
        let now = std::time::Instant::now();
        {
            let last = self.last_detection_time.lock().unwrap();
            let cooldown = std::time::Duration::from_secs(self.settings.cooldown_seconds as u64);
            if now.duration_since(*last) < cooldown {
                debug!("Wake-word cooldown active, ignoring detection");
                return WakeDetectionResult {
                    detected: false,
                    phrase_detected: false,
                    phrase_score: 0.0,
                    kws_score: 0.0,
                    speaker_score: None,
                    spoof_score: None,
                    rejection_reason: Some(WakeRejectionReason::CooldownActive),
                };
            }
        }

        let has_signal = if self.settings.vad_enabled {
            // 0.008 was too strict on many laptop microphones and blocked valid speech.
            samples.map(rms_energy).map(|rms| rms >= 0.002).unwrap_or(true)
        } else {
            true
        };

        // KWS-like phrase scoring.
        let normalized_transcript = if self.settings.case_insensitive {
            normalize_wake_text(transcript)
        } else {
            transcript.to_string()
        };
        let normalized_phrase = if self.settings.case_insensitive {
            normalize_wake_text(&self.settings.wake_phrase)
        } else {
            self.settings.wake_phrase.clone()
        };
        let phrase_score = if self.settings.case_insensitive {
            wake_phrase_score(&normalized_transcript, &normalized_phrase)
        } else {
            strict_wake_phrase_score(&normalized_transcript, &normalized_phrase)
        };
        let mut kws_threshold = self
            .settings
            .kws_threshold
            .max(self.settings.detection_threshold)
            .clamp(0.0, 1.0);
        // Short wake phrases (e.g. "hey handy") are commonly transcribed with slight drift.
        // Keep threshold practical to avoid missed activations.
        if normalized_phrase.split_whitespace().count() <= 2 {
            kws_threshold = kws_threshold.min(0.72);
        }
        info!(
            "[wake-detector] normalized_transcript='{}' normalized_phrase='{}' score={:.3} threshold={:.3} has_signal={}",
            truncate_for_log(&normalized_transcript, 120),
            normalized_phrase,
            phrase_score,
            kws_threshold,
            has_signal
        );
        let phrase_detected = phrase_score >= kws_threshold;

        if !phrase_detected {
            return WakeDetectionResult {
                detected: false,
                phrase_detected: false,
                phrase_score,
                kws_score: phrase_score,
                speaker_score: None,
                spoof_score: None,
                rejection_reason: Some(WakeRejectionReason::PhraseNotDetected),
            };
        }

        let mut speaker_score = None;
        let mut spoof_score = None;
        if self.settings.voice_auth_enabled
            && self.settings.voice_auth_mode == VoiceAuthMode::OwnerOnly
        {
            let profile = match self.settings.voice_profile.as_ref() {
                Some(profile) => profile,
                None => {
                    if self.settings.fallback_when_no_profile {
                        info!(
                            "[wake-detector] voice auth enabled but no profile enrolled; using phrase-only fallback"
                        );
                        if let Ok(mut last) = self.last_detection_time.lock() {
                            *last = now;
                        }
                        return WakeDetectionResult {
                            detected: true,
                            phrase_detected: true,
                            phrase_score,
                            kws_score: phrase_score,
                            speaker_score: None,
                            spoof_score: None,
                            rejection_reason: None,
                        };
                    }
                    return WakeDetectionResult {
                        detected: false,
                        phrase_detected: true,
                        phrase_score,
                        kws_score: phrase_score,
                        speaker_score: None,
                        spoof_score: None,
                        rejection_reason: Some(WakeRejectionReason::MissingVoiceProfile),
                    };
                }
            };

            let samples = match samples {
                Some(s) => s,
                None => {
                    return WakeDetectionResult {
                        detected: false,
                        phrase_detected: true,
                        phrase_score,
                        kws_score: phrase_score,
                        speaker_score: None,
                        spoof_score: None,
                        rejection_reason: Some(WakeRejectionReason::AudioTooShort),
                    };
                }
            };

            let min_samples =
                (16_000usize * self.settings.quality_min_duration_ms as usize) / 1000usize;
            if samples.len() < min_samples {
                return WakeDetectionResult {
                    detected: false,
                    phrase_detected: true,
                    phrase_score,
                    kws_score: phrase_score,
                    speaker_score: None,
                    spoof_score: None,
                    rejection_reason: Some(WakeRejectionReason::AudioTooShort),
                };
            }

            let spoof = anti_spoof_score(samples);
            spoof_score = Some(spoof);
            info!(
                "[wake-detector] spoof score={:.3} threshold={:.3}",
                spoof, self.settings.spoof_threshold
            );
            if spoof < self.settings.spoof_threshold {
                return WakeDetectionResult {
                    detected: false,
                    phrase_detected: true,
                    phrase_score,
                    kws_score: phrase_score,
                    speaker_score: None,
                    spoof_score,
                    rejection_reason: Some(WakeRejectionReason::SpoofSuspected),
                };
            }

            let score = voice_similarity_score(samples, profile);
            speaker_score = Some(score);
            info!(
                "[wake-detector] speaker score={:.3} threshold={:.3}",
                score, self.settings.speaker_threshold
            );
            if score < self.settings.speaker_threshold {
                return WakeDetectionResult {
                    detected: false,
                    phrase_detected: true,
                    phrase_score,
                    kws_score: phrase_score,
                    speaker_score,
                    spoof_score,
                    rejection_reason: Some(WakeRejectionReason::SpeakerMismatch),
                };
            }
        }

        if phrase_detected {
            info!(
                "Wake-word '{}' detected in transcript",
                self.settings.wake_phrase
            );
            // Update last detection time
            if let Ok(mut last) = self.last_detection_time.lock() {
                *last = now;
            }
        }

        WakeDetectionResult {
            detected: true,
            phrase_detected: true,
            phrase_score,
            kws_score: phrase_score,
            speaker_score,
            spoof_score,
            rejection_reason: None,
        }
    }

    #[allow(dead_code)]
    pub fn detect_in_transcript(&self, transcript: &str) -> bool {
        self.detect(transcript, None).detected
    }

    /// Get the action to perform when wake-word is detected.
    pub fn get_trigger_action(&self) -> WakeWordAction {
        self.settings.trigger_action.clone()
    }

    /// Get the current settings.
    #[allow(dead_code)]
    pub fn get_settings(&self) -> &WakeWordSettings {
        &self.settings
    }
}

fn normalize_wake_text(input: &str) -> String {
    let mut normalized = String::with_capacity(input.len());
    let mut last_was_space = false;
    for ch in input.chars() {
        if ch.is_alphanumeric() {
            normalized.extend(ch.to_lowercase());
            last_was_space = false;
        } else if !last_was_space {
            normalized.push(' ');
            last_was_space = true;
        }
    }
    normalized.trim().to_string()
}

fn truncate_for_log(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }
    format!("{}...", &text[..max_len])
}

fn wake_phrase_score(normalized_transcript: &str, normalized_phrase: &str) -> f32 {
    if normalized_phrase.is_empty() {
        return 0.0;
    }
    if normalized_transcript.contains(normalized_phrase) {
        return 1.0;
    }
    let phrase_tokens: Vec<&str> = normalized_phrase.split_whitespace().collect();
    let transcript_tokens: Vec<&str> = normalized_transcript.split_whitespace().collect();
    if phrase_tokens.is_empty() {
        return 0.0;
    }
    let mut hits = 0usize;
    let mut index = 0usize;
    for token in &phrase_tokens {
        while index < transcript_tokens.len() {
            if transcript_tokens[index] == *token {
                hits += 1;
                index += 1;
                break;
            }
            index += 1;
        }
    }
    let ordered_exact = hits as f32 / phrase_tokens.len() as f32;
    let fuzzy_window = fuzzy_window_phrase_score(&transcript_tokens, &phrase_tokens);
    let joined_similarity = normalized_similarity(
        &transcript_tokens.join(" "),
        &phrase_tokens.join(" "),
    );
    let mut best = ordered_exact.max(fuzzy_window).max(joined_similarity * 0.9);

    // Practical aliases for common wake ASR drift on "hey handy".
    // This keeps wake responsive even when STT substitutes near-homophones.
    if normalized_phrase == "hey handy" {
        for alt in wake_phrase_aliases() {
            let alt_tokens: Vec<&str> = alt.split_whitespace().collect();
            let alias_score = fuzzy_window_phrase_score(&transcript_tokens, &alt_tokens)
                .max(normalized_similarity(normalized_transcript, alt));
            best = best.max(alias_score);
        }
    }

    best
}

fn wake_phrase_aliases() -> &'static [&'static str] {
    &[
        "hey andy",
        "hi handy",
        "hey hendy",
        "hey hindi",
        "okay handy",
        "yo handy",
    ]
}

fn strict_wake_phrase_score(transcript: &str, phrase: &str) -> f32 {
    if phrase.is_empty() {
        return 0.0;
    }
    if transcript.contains(phrase) {
        return 1.0;
    }
    let phrase_tokens: Vec<&str> = phrase.split_whitespace().collect();
    let transcript_tokens: Vec<&str> = transcript.split_whitespace().collect();
    if phrase_tokens.is_empty() {
        return 0.0;
    }
    let mut hits = 0usize;
    let mut index = 0usize;
    for token in &phrase_tokens {
        while index < transcript_tokens.len() {
            if transcript_tokens[index] == *token {
                hits += 1;
                index += 1;
                break;
            }
            index += 1;
        }
    }
    hits as f32 / phrase_tokens.len() as f32
}

fn fuzzy_window_phrase_score(transcript_tokens: &[&str], phrase_tokens: &[&str]) -> f32 {
    if phrase_tokens.is_empty() || transcript_tokens.is_empty() {
        return 0.0;
    }
    let win = phrase_tokens.len();
    if transcript_tokens.len() < win {
        return 0.0;
    }
    let mut best = 0.0f32;
    for start in 0..=(transcript_tokens.len() - win) {
        let mut acc = 0.0f32;
        for i in 0..win {
            acc += normalized_similarity(transcript_tokens[start + i], phrase_tokens[i]);
        }
        best = best.max(acc / win as f32);
    }
    best
}

fn normalized_similarity(a: &str, b: &str) -> f32 {
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let dist = levenshtein_distance(a, b);
    let max_len = a.chars().count().max(b.chars().count()) as f32;
    (1.0 - (dist as f32 / max_len)).clamp(0.0, 1.0)
}

fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    if a_chars.is_empty() {
        return b_chars.len();
    }
    if b_chars.is_empty() {
        return a_chars.len();
    }

    let mut prev: Vec<usize> = (0..=b_chars.len()).collect();
    let mut curr = vec![0usize; b_chars.len() + 1];

    for (i, ac) in a_chars.iter().enumerate() {
        curr[0] = i + 1;
        for (j, bc) in b_chars.iter().enumerate() {
            let cost = if ac == bc { 0 } else { 1 };
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b_chars.len()]
}

fn rms_energy(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn anti_spoof_score(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let features = extract_voice_features(samples);
    if features.len() < 6 {
        return 0.0;
    }
    let zcr = features[3].clamp(0.0, 1.0);
    let frame_var = features[5].clamp(0.0, 1.0);
    let zcr_score = 1.0 - (zcr - 0.08).abs().min(0.08) / 0.08;
    let var_score = (frame_var / 0.12).clamp(0.0, 1.0);
    (0.6 * zcr_score + 0.4 * var_score).clamp(0.0, 1.0)
}

pub fn build_voice_profile(samples: &[Vec<f32>]) -> Result<WakeVoiceProfile, String> {
    if samples.is_empty() {
        return Err("No enrollment samples provided".to_string());
    }

    let feature_vectors: Vec<Vec<f32>> = samples
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| extract_voice_features(s))
        .collect();

    if feature_vectors.is_empty() {
        return Err("No valid enrollment samples provided".to_string());
    }

    let dims = feature_vectors[0].len();
    let mut centroid = vec![0.0f32; dims];
    for vec in &feature_vectors {
        for (i, value) in vec.iter().enumerate() {
            centroid[i] += *value;
        }
    }
    let n = feature_vectors.len() as f32;
    for value in &mut centroid {
        *value /= n;
    }
    normalize_vec(&mut centroid);

    Ok(WakeVoiceProfile {
        centroid,
        sample_count: feature_vectors.len() as u32,
    })
}

pub fn voice_similarity_score(samples: &[f32], profile: &WakeVoiceProfile) -> f32 {
    if samples.is_empty() || profile.centroid.is_empty() {
        return 0.0;
    }
    let mut v = extract_voice_features(samples);
    normalize_vec(&mut v);
    cosine_similarity(&v, &profile.centroid).clamp(0.0, 1.0)
}

fn extract_voice_features(samples: &[f32]) -> Vec<f32> {
    if samples.is_empty() {
        return vec![0.0; 6];
    }

    let len = samples.len() as f32;
    let mut sum_abs = 0.0f32;
    let mut sum_sq = 0.0f32;
    let mut peak = 0.0f32;
    let mut zcr = 0.0f32;
    let mut mean = 0.0f32;

    for (i, s) in samples.iter().enumerate() {
        let abs = s.abs();
        sum_abs += abs;
        sum_sq += s * s;
        peak = peak.max(abs);
        mean += s;
        if i > 0 {
            let prev = samples[i - 1];
            if (prev >= 0.0 && *s < 0.0) || (prev < 0.0 && *s >= 0.0) {
                zcr += 1.0;
            }
        }
    }
    mean /= len;

    let rms = (sum_sq / len).sqrt();
    let mean_abs = sum_abs / len;
    let zcr_rate = zcr / len;

    let mut variance = 0.0f32;
    for s in samples {
        let d = *s - mean;
        variance += d * d;
    }
    variance /= len;

    let mut frame_energy_var = 0.0f32;
    let frame_size = 320usize;
    let frame_count = (samples.len() / frame_size).max(1);
    let mut frame_energies = Vec::with_capacity(frame_count);
    for frame in samples.chunks(frame_size) {
        let mut e = 0.0f32;
        for s in frame {
            e += s * s;
        }
        frame_energies.push((e / frame.len().max(1) as f32).sqrt());
    }
    let frame_mean = frame_energies.iter().copied().sum::<f32>() / frame_energies.len() as f32;
    for e in &frame_energies {
        let d = *e - frame_mean;
        frame_energy_var += d * d;
    }
    frame_energy_var /= frame_energies.len() as f32;

    vec![
        mean_abs,
        rms,
        peak,
        zcr_rate,
        variance.sqrt(),
        frame_energy_var,
    ]
}

fn normalize_vec(v: &mut [f32]) {
    let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-6 {
        for value in v {
            *value /= norm;
        }
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let len = a.len().min(b.len());
    if len == 0 {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..len {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na <= 1e-6 || nb <= 1e-6 {
        return 0.0;
    }
    (dot / (na.sqrt() * nb.sqrt()) + 1.0) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn project_audio_fixture(file_name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("tests")
            .join("audio")
            .join(file_name)
    }

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

    #[test]
    fn test_voice_profile_and_similarity() {
        let sample_a = vec![0.1f32; 16_000];
        let sample_b = vec![0.12f32; 16_000];
        let profile = build_voice_profile(&[sample_a.clone(), sample_b]).unwrap();
        let score = voice_similarity_score(&sample_a, &profile);
        assert!(score > 0.7, "score too low: {score}");
    }

    #[test]
    fn test_wake_audio_fixtures_are_valid_and_non_silent() {
        let fixture_names = [
            "wake_up_command.wav",
            "wake_up_start_listening.wav",
            "wake_up_question.wav",
            "non_wake_control.wav",
        ];

        for name in fixture_names {
            let path = project_audio_fixture(name);
            assert!(path.exists(), "missing fixture: {}", path.display());

            let mut reader = hound::WavReader::open(&path)
                .unwrap_or_else(|_| panic!("failed to open wav fixture: {}", path.display()));
            let spec = reader.spec();
            assert_eq!(
                spec.sample_rate, 16_000,
                "unexpected sample rate for {name}"
            );
            assert_eq!(spec.channels, 1, "unexpected channel count for {name}");

            let samples: Vec<i16> = reader.samples::<i16>().filter_map(Result::ok).collect();
            assert!(!samples.is_empty(), "fixture has no samples: {name}");

            // Basic energy check so files are not silent placeholders.
            let mean_abs = samples
                .iter()
                .map(|s| (*s as i32).unsigned_abs() as f64)
                .sum::<f64>()
                / samples.len() as f64;
            assert!(
                mean_abs > 20.0,
                "fixture appears silent or near-silent: {name} (mean_abs={mean_abs})"
            );
        }
    }
}
