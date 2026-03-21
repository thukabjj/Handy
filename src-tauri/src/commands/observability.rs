use crate::settings::observability::{ObservabilityDataMode, ObservabilitySettings};
use crate::settings::{get_settings, write_settings};
use crate::telemetry::{
    TelemetryEvent, TelemetryEventBuilder, TelemetryManager, TelemetryRecordInput,
    TelemetrySnapshot,
};
use chrono::Utc;
use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[derive(Serialize)]
struct DiagnosticsBundle {
    generated_at_ms: i64,
    app_version: String,
    platform: String,
    app_data_dir: String,
    log_dir: String,
    settings_summary: DiagnosticsSettingsSummary,
    telemetry: TelemetrySnapshot,
}

#[derive(Serialize)]
struct DiagnosticsSettingsSummary {
    active_llm_provider: String,
    cloud_model: String,
    ollama_model: String,
    has_cloud_api_key: bool,
    wake_word_enabled: bool,
    ask_ai_enabled: bool,
    active_listening_enabled: bool,
    knowledge_base_enabled: bool,
    screen_vision_enabled: bool,
    suggestions_enabled: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_observability_settings(app: AppHandle) -> ObservabilitySettings {
    get_settings(&app).observability
}

#[tauri::command]
#[specta::specta]
pub fn update_observability_settings(
    app: AppHandle,
    settings: ObservabilitySettings,
) -> Result<(), String> {
    let mut app_settings = get_settings(&app);
    app_settings.observability = settings;
    write_settings(&app, app_settings);

    if let Some(manager) = app.try_state::<Arc<TelemetryManager>>() {
        manager.trim_to_limit();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_observability_enabled_setting(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.observability.enabled = enabled;
    write_settings(&app, settings);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_observability_data_mode_setting(
    app: AppHandle,
    mode: ObservabilityDataMode,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.observability.data_mode = mode;
    write_settings(&app, settings);

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn change_observability_max_events_setting(
    app: AppHandle,
    max_events: u32,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.observability.max_events = max_events;
    write_settings(&app, settings);

    if let Some(manager) = app.try_state::<Arc<TelemetryManager>>() {
        manager.trim_to_limit();
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn get_recent_telemetry_events(
    app: AppHandle,
    limit: Option<u32>,
) -> Result<Vec<TelemetryEvent>, String> {
    let manager = app
        .try_state::<Arc<TelemetryManager>>()
        .ok_or_else(|| "Telemetry manager unavailable".to_string())?;
    Ok(manager.recent_events(limit))
}

#[tauri::command]
#[specta::specta]
pub fn get_telemetry_snapshot(app: AppHandle) -> Result<TelemetrySnapshot, String> {
    let manager = app
        .try_state::<Arc<TelemetryManager>>()
        .ok_or_else(|| "Telemetry manager unavailable".to_string())?;
    Ok(manager.snapshot())
}

#[tauri::command]
#[specta::specta]
pub fn clear_telemetry_events(app: AppHandle) -> Result<(), String> {
    let manager = app
        .try_state::<Arc<TelemetryManager>>()
        .ok_or_else(|| "Telemetry manager unavailable".to_string())?;
    manager.clear();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn record_frontend_telemetry_event(
    app: AppHandle,
    event: TelemetryRecordInput,
) -> Result<(), String> {
    let settings = get_settings(&app);
    if !settings.observability.capture_frontend_events {
        return Ok(());
    }

    let manager = app
        .try_state::<Arc<TelemetryManager>>()
        .ok_or_else(|| "Telemetry manager unavailable".to_string())?;
    manager.record(event);
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn export_telemetry_snapshot_bundle(app: AppHandle) -> Result<String, String> {
    let manager = app
        .try_state::<Arc<TelemetryManager>>()
        .ok_or_else(|| "Telemetry manager unavailable".to_string())?;
    let snapshot = manager.snapshot();
    let settings = get_settings(&app);
    let app_data_dir = crate::portable::app_data_dir(&app)
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    let log_dir =
        crate::portable::app_log_dir(&app).map_err(|e| format!("Failed to resolve log dir: {}", e))?;
    let diagnostics_dir = app_data_dir.join("diagnostics");
    std::fs::create_dir_all(&diagnostics_dir)
        .map_err(|e| format!("Failed to create diagnostics dir: {}", e))?;

    let bundle = DiagnosticsBundle {
        generated_at_ms: Utc::now().timestamp_millis(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        platform: std::env::consts::OS.to_string(),
        app_data_dir: app_data_dir.to_string_lossy().to_string(),
        log_dir: log_dir.to_string_lossy().to_string(),
        settings_summary: DiagnosticsSettingsSummary {
            active_llm_provider: format!("{:?}", settings.active_listening.llm_provider),
            cloud_model: settings.active_listening.llm_model.unwrap_or_default(),
            ollama_model: settings.active_listening.ollama_model,
            has_cloud_api_key: settings
                .active_listening
                .llm_api_key
                .as_ref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false),
            wake_word_enabled: settings.wake_word.enabled,
            ask_ai_enabled: settings.ask_ai.enabled,
            active_listening_enabled: settings.active_listening.enabled,
            knowledge_base_enabled: settings.knowledge_base.enabled,
            screen_vision_enabled: settings.screen_vision.enabled,
            suggestions_enabled: settings.suggestions.enabled,
        },
        telemetry: snapshot,
    };

    let file_path = diagnostics_dir.join(format!(
        "observability-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    let content = serde_json::to_string_pretty(&bundle)
        .map_err(|e| format!("Failed to serialize diagnostics bundle: {}", e))?;
    std::fs::write(&file_path, content)
        .map_err(|e| format!("Failed to write diagnostics bundle: {}", e))?;

    TelemetryEventBuilder::new("observability", "bundle_exported")
        .message("Observability bundle exported")
        .attr("path", file_path.to_string_lossy().to_string())
        .emit();

    Ok(file_path.to_string_lossy().to_string())
}
