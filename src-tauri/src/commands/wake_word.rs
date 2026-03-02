//! Tauri commands for wake-word detection settings.

use crate::managers::wake_word::{WakeWordAction, WakeWordSettings};
use crate::settings::{get_settings, write_settings};
use tauri::AppHandle;

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
    settings.wake_word.detection_threshold = threshold.clamp(0.0, 1.0);
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
