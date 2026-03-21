//! Tauri commands for wake-word detection settings.

use crate::managers::wake_word::{
    build_voice_profile, voice_similarity_score, VoiceAuthMode, WakeVoiceProfile, WakeWordAction,
    WakeWordSettings,
};
use crate::managers::{
    active_listening::ActiveListeningManager, audio::AudioRecordingManager,
    transcription::TranscriptionManager,
};
use crate::settings::{get_settings, write_settings};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

/// Get the current wake-word settings.
#[tauri::command]
#[specta::specta]
pub fn get_wake_word_settings(app: AppHandle) -> WakeWordSettings {
    get_settings(&app).wake_word
}

/// Update wake-word settings.
#[tauri::command]
#[specta::specta]
pub fn update_wake_word_settings(app: AppHandle, settings: WakeWordSettings) {
    let mut app_settings = get_settings(&app);
    app_settings.wake_word = settings;
    write_settings(&app, app_settings);
}

/// Enable or disable wake-word detection.
#[tauri::command]
#[specta::specta]
pub fn change_wake_word_enabled_setting(app: AppHandle, enabled: bool) {
    let mut settings = get_settings(&app);
    settings.wake_word.enabled = enabled;
    write_settings(&app, settings);
    if let Err(err) = sync_wake_word_monitoring(&app) {
        log::warn!("Failed to sync wake-word monitoring state: {}", err);
    }
}

/// Change the wake phrase.
#[tauri::command]
#[specta::specta]
pub fn change_wake_phrase_setting(app: AppHandle, phrase: String) {
    let mut settings = get_settings(&app);
    settings.wake_word.wake_phrase = phrase;
    write_settings(&app, settings);
}

/// Change the wake-word trigger action.
#[tauri::command]
#[specta::specta]
pub fn change_wake_word_action_setting(app: AppHandle, action: WakeWordAction) {
    let mut settings = get_settings(&app);
    settings.wake_word.trigger_action = action;
    write_settings(&app, settings);
}

/// Change the detection threshold.
#[tauri::command]
#[specta::specta]
pub fn change_wake_word_threshold_setting(app: AppHandle, threshold: f32) {
    let mut settings = get_settings(&app);
    let value = threshold.clamp(0.0, 1.0);
    settings.wake_word.detection_threshold = value;
    settings.wake_word.kws_threshold = value;
    write_settings(&app, settings);
}

/// Change the cooldown period between detections.
#[tauri::command]
#[specta::specta]
pub fn change_wake_word_cooldown_setting(app: AppHandle, seconds: u32) {
    let mut settings = get_settings(&app);
    settings.wake_word.cooldown_seconds = seconds;
    write_settings(&app, settings);
}

#[derive(Debug, Default)]
struct EnrollmentSession {
    active: bool,
    samples: Vec<Vec<f32>>,
}

static ENROLLMENT_SESSION: Lazy<Mutex<EnrollmentSession>> =
    Lazy::new(|| Mutex::new(EnrollmentSession::default()));

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeVoiceProfileStatus {
    pub enrolled: bool,
    pub sample_count: u32,
    pub voice_auth_enabled: bool,
    pub mode: VoiceAuthMode,
    pub enrollment_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeVoiceAuthTestResult {
    pub accepted: bool,
    pub score: f32,
    pub threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct WakeCalibrationResult {
    pub suggested_kws_threshold: f32,
    pub suggested_speaker_threshold: f32,
    pub suggested_spoof_threshold: f32,
    pub target_far_per_hour: f32,
    pub note: String,
}

/// Enable or disable owner-only voice authentication.
#[tauri::command]
#[specta::specta]
pub fn change_wake_word_voice_auth_enabled_setting(app: AppHandle, enabled: bool) {
    let mut settings = get_settings(&app);
    settings.wake_word.voice_auth_enabled = enabled;
    settings.wake_word.voice_auth_mode = if enabled {
        VoiceAuthMode::OwnerOnly
    } else {
        VoiceAuthMode::Disabled
    };
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn change_wake_word_kws_threshold_setting(app: AppHandle, threshold: f32) {
    let mut settings = get_settings(&app);
    let value = threshold.clamp(0.0, 1.0);
    settings.wake_word.kws_threshold = value;
    settings.wake_word.detection_threshold = value;
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn change_wake_word_spoof_threshold_setting(app: AppHandle, threshold: f32) {
    let mut settings = get_settings(&app);
    settings.wake_word.spoof_threshold = threshold.clamp(0.0, 1.0);
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn change_wake_word_vad_enabled_setting(app: AppHandle, enabled: bool) {
    let mut settings = get_settings(&app);
    settings.wake_word.vad_enabled = enabled;
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn change_wake_word_target_far_setting(app: AppHandle, target_far_per_hour: f32) {
    let mut settings = get_settings(&app);
    settings.wake_word.target_far_per_hour = target_far_per_hour.clamp(0.01, 2.0);
    write_settings(&app, settings);
}

#[tauri::command]
#[specta::specta]
pub fn run_wake_calibration_session(app: AppHandle) -> WakeCalibrationResult {
    let mut settings = get_settings(&app);
    let target_far = settings.wake_word.target_far_per_hour.clamp(0.01, 2.0);
    let kws = if target_far <= 0.1 {
        0.85
    } else if target_far <= 0.5 {
        0.80
    } else {
        0.75
    };
    let speaker = if settings.wake_word.voice_auth_enabled {
        if target_far <= 0.1 { 0.88 } else { 0.84 }
    } else {
        settings.wake_word.speaker_threshold
    };
    let spoof = if settings.wake_word.voice_auth_enabled {
        if target_far <= 0.1 { 0.62 } else { 0.55 }
    } else {
        settings.wake_word.spoof_threshold
    };

    settings.wake_word.kws_threshold = kws;
    settings.wake_word.detection_threshold = kws;
    settings.wake_word.speaker_threshold = speaker;
    settings.wake_word.spoof_threshold = spoof;
    write_settings(&app, settings);

    WakeCalibrationResult {
        suggested_kws_threshold: kws,
        suggested_speaker_threshold: speaker,
        suggested_spoof_threshold: spoof,
        target_far_per_hour: target_far,
        note: "Calibration applied from FAR target. Re-test in your room and adjust if needed."
            .to_string(),
    }
}

/// Start owner voice enrollment flow.
#[tauri::command]
#[specta::specta]
pub fn start_wake_voice_enrollment() {
    if let Ok(mut session) = ENROLLMENT_SESSION.lock() {
        session.active = true;
        session.samples.clear();
    }
}

/// Capture one enrollment sample from a WAV file path.
#[tauri::command]
#[specta::specta]
pub fn capture_wake_voice_sample(audio_path: String) -> Result<u32, String> {
    let samples = load_wav_samples(&audio_path)?;
    let mut session = ENROLLMENT_SESSION
        .lock()
        .map_err(|_| "Failed to lock enrollment session".to_string())?;
    if !session.active {
        return Err("Enrollment session is not active".to_string());
    }
    session.samples.push(samples);
    Ok(session.samples.len() as u32)
}

/// Finish enrollment and persist profile into wake word settings.
#[tauri::command]
#[specta::specta]
pub fn finish_wake_voice_enrollment(app: AppHandle) -> Result<WakeVoiceProfileStatus, String> {
    let mut session = ENROLLMENT_SESSION
        .lock()
        .map_err(|_| "Failed to lock enrollment session".to_string())?;
    if !session.active {
        return Err("Enrollment session is not active".to_string());
    }
    if session.samples.len() < 3 {
        return Err("At least 3 samples are required for enrollment".to_string());
    }

    let profile = build_voice_profile(&session.samples)?;
    session.active = false;
    session.samples.clear();

    let mut settings = get_settings(&app);
    settings.wake_word.voice_profile = Some(profile.clone());
    settings.wake_word.voice_auth_enabled = true;
    settings.wake_word.voice_auth_mode = VoiceAuthMode::OwnerOnly;
    write_settings(&app, settings.clone());

    Ok(WakeVoiceProfileStatus {
        enrolled: true,
        sample_count: profile.sample_count,
        voice_auth_enabled: settings.wake_word.voice_auth_enabled,
        mode: settings.wake_word.voice_auth_mode,
        enrollment_active: false,
    })
}

/// Clear enrolled owner voice profile.
#[tauri::command]
#[specta::specta]
pub fn reset_wake_voice_enrollment(app: AppHandle) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.wake_word.voice_profile = None;
    settings.wake_word.voice_auth_enabled = false;
    settings.wake_word.voice_auth_mode = VoiceAuthMode::Disabled;
    write_settings(&app, settings);

    if let Ok(mut session) = ENROLLMENT_SESSION.lock() {
        session.active = false;
        session.samples.clear();
    }
    Ok(())
}

/// Get owner voice profile enrollment status.
#[tauri::command]
#[specta::specta]
pub fn get_wake_voice_profile_status(app: AppHandle) -> WakeVoiceProfileStatus {
    let settings = get_settings(&app);
    let enrollment_active = ENROLLMENT_SESSION.lock().map(|s| s.active).unwrap_or(false);
    let profile: Option<&WakeVoiceProfile> = settings.wake_word.voice_profile.as_ref();
    WakeVoiceProfileStatus {
        enrolled: profile.is_some(),
        sample_count: profile.map(|p| p.sample_count).unwrap_or(0),
        voice_auth_enabled: settings.wake_word.voice_auth_enabled,
        mode: settings.wake_word.voice_auth_mode,
        enrollment_active,
    }
}

/// Test voice authentication score against enrolled profile using a WAV sample.
#[tauri::command]
#[specta::specta]
pub fn test_wake_voice_auth(
    audio_path: String,
    app: AppHandle,
) -> Result<WakeVoiceAuthTestResult, String> {
    let settings = get_settings(&app);
    let profile = settings
        .wake_word
        .voice_profile
        .as_ref()
        .ok_or_else(|| "No enrolled voice profile found".to_string())?;
    let samples = load_wav_samples(&audio_path)?;
    let score = voice_similarity_score(&samples, profile);
    Ok(WakeVoiceAuthTestResult {
        accepted: score >= settings.wake_word.speaker_threshold,
        score,
        threshold: settings.wake_word.speaker_threshold,
    })
}

fn load_wav_samples(path: &str) -> Result<Vec<f32>, String> {
    let mut reader = hound::WavReader::open(path)
        .map_err(|e| format!("Failed to open WAV file '{}': {}", path, e))?;
    let spec = reader.spec();
    if spec.channels != 1 {
        return Err("Only mono WAV files are supported for enrollment".to_string());
    }
    let samples: Vec<f32> = reader
        .samples::<i16>()
        .filter_map(Result::ok)
        .map(|s| s as f32 / i16::MAX as f32)
        .collect();
    if samples.is_empty() {
        return Err("WAV file contains no samples".to_string());
    }
    Ok(samples)
}

pub fn sync_wake_word_monitoring(app: &AppHandle) -> Result<(), String> {
    let settings = get_settings(app);
    let wake_enabled = settings.wake_word.enabled;
    let transcription_manager = app.state::<Arc<TranscriptionManager>>();
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    let audio_manager = app.state::<Arc<AudioRecordingManager>>();
    let al_state = al_manager.get_state();

    log::info!(
        "[wake-sync] enabled={} state={:?} phrase='{}' action={:?} provider={:?} model_local='{}' model_cloud='{}'",
        wake_enabled,
        al_state,
        settings.wake_word.wake_phrase,
        settings.wake_word.trigger_action,
        settings.active_listening.llm_provider,
        settings.active_listening.ollama_model,
        settings.active_listening.llm_model.clone().unwrap_or_default()
    );

    if wake_enabled {
        // Wake-word detection depends on STT; proactively load the model.
        transcription_manager.initiate_model_load();
        log::info!("[wake-sync] requested model preload");
        if al_state == crate::managers::active_listening::ActiveListeningState::Idle {
            let al_manager_clone = al_manager.inner().clone();
            let callback = Arc::new(move |samples: &[f32]| {
                al_manager_clone.push_audio_samples(samples);
            });
            audio_manager
                .start_active_listening(callback)
                .map_err(|e| format!("Failed to start wake-word monitoring: {}", e))?;
            log::info!("[wake-sync] passive wake monitoring started");
        }
    } else if al_state == crate::managers::active_listening::ActiveListeningState::Idle {
        audio_manager
            .stop_active_listening()
            .map_err(|e| format!("Failed to stop wake-word monitoring: {}", e))?;
        log::info!("[wake-sync] passive wake monitoring stopped");
    }

    Ok(())
}
