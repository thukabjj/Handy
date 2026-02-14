//! Tauri commands for Environmental Sound Detection settings

use crate::audio_toolkit::SoundDetector;
use crate::settings::sound_detection::{SoundCategory, SoundDetectionSettings};
use crate::settings::{get_settings, write_settings};
use std::sync::Mutex;
use tauri::{AppHandle, State};

/// Get current sound detection settings
#[tauri::command]
#[specta::specta]
pub fn get_sound_detection_settings(app: AppHandle) -> Result<SoundDetectionSettings, String> {
    let settings = get_settings(&app);
    Ok(settings.sound_detection)
}

/// Enable or disable sound detection
#[tauri::command]
#[specta::specta]
pub fn change_sound_detection_enabled(
    app: AppHandle,
    enabled: bool,
    detector: State<'_, Mutex<SoundDetector>>,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.sound_detection.enabled = enabled;
    if let Ok(mut det) = detector.lock() {
        det.update_settings(&settings.sound_detection);
    }
    write_settings(&app, settings);
    Ok(())
}

/// Update the detection confidence threshold
#[tauri::command]
#[specta::specta]
pub fn change_sound_detection_threshold(
    app: AppHandle,
    threshold: f32,
    detector: State<'_, Mutex<SoundDetector>>,
) -> Result<(), String> {
    if !(0.0..=1.0).contains(&threshold) {
        return Err("Threshold must be between 0.0 and 1.0".to_string());
    }
    let mut settings = get_settings(&app);
    settings.sound_detection.threshold = threshold;
    if let Ok(mut det) = detector.lock() {
        det.update_settings(&settings.sound_detection);
    }
    write_settings(&app, settings);
    Ok(())
}

/// Update which sound categories to detect
#[tauri::command]
#[specta::specta]
pub fn change_sound_detection_categories(
    app: AppHandle,
    categories: Vec<SoundCategory>,
    detector: State<'_, Mutex<SoundDetector>>,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.sound_detection.categories = categories;
    if let Ok(mut det) = detector.lock() {
        det.update_settings(&settings.sound_detection);
    }
    write_settings(&app, settings);
    Ok(())
}

/// Enable or disable sound detection notifications
#[tauri::command]
#[specta::specta]
pub fn change_sound_detection_notification(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.sound_detection.notification_enabled = enabled;
    write_settings(&app, settings);
    Ok(())
}
