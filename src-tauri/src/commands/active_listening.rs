//! Tauri commands for Active Listening feature

use crate::audio_toolkit::audio::loopback::{LoopbackCapture, LoopbackSupport};
use crate::managers::active_listening::{
    ActiveListeningManager, ActiveListeningSession, ActiveListeningState, MeetingSummary,
};
use crate::managers::audio::AudioRecordingManager;
use crate::ollama_client::OllamaClient;
use crate::settings::{
    get_settings, write_settings, ActiveListeningPrompt, AudioSourceType, PromptCategory,
};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Ollama model information
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct OllamaModel {
    pub name: String,
}

/// Loopback device information for the frontend
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct LoopbackDeviceInfoDto {
    pub id: String,
    pub name: String,
    pub is_default: bool,
}

/// Loopback support level for the frontend
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum LoopbackSupportLevel {
    /// Full native support for loopback capture
    Native,
    /// Requires additional software/virtual audio device
    RequiresVirtualDevice,
    /// Not supported on this platform
    NotSupported,
}

/// Start an active listening session
#[tauri::command]
#[specta::specta]
pub fn start_active_listening_session(
    app: AppHandle,
    topic: Option<String>,
) -> Result<String, String> {
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    let audio_manager = app.state::<Arc<AudioRecordingManager>>();

    // First start the session in the manager
    let session_id = al_manager.start_session(topic)?;

    // Create callback to forward audio samples to the active listening manager
    let al_manager_clone = al_manager.inner().clone();
    let callback = Arc::new(move |samples: &[f32]| {
        al_manager_clone.push_audio_samples(samples);
    });

    // Start active listening mode in the audio manager
    audio_manager
        .start_active_listening(callback)
        .map_err(|e| format!("Failed to start active listening: {}", e))?;

    info!("Active listening session started: {}", session_id);
    Ok(session_id)
}

/// Stop the current active listening session
#[tauri::command]
#[specta::specta]
pub fn stop_active_listening_session(
    app: AppHandle,
) -> Result<Option<ActiveListeningSession>, String> {
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    let audio_manager = app.state::<Arc<AudioRecordingManager>>();

    // Flush any remaining audio
    al_manager.flush_segment();

    // Stop active listening in audio manager
    audio_manager
        .stop_active_listening()
        .map_err(|e| format!("Failed to stop active listening: {}", e))?;

    // Stop the session
    let session = al_manager.stop_session()?;

    if let Some(ref s) = session {
        info!(
            "Active listening session stopped: {} with {} insights",
            s.id,
            s.insights.len()
        );
    }

    Ok(session)
}

/// Get the current active listening state
#[tauri::command]
#[specta::specta]
pub fn get_active_listening_state(app: AppHandle) -> ActiveListeningState {
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    al_manager.get_state()
}

/// Get the current active listening session info
#[tauri::command]
#[specta::specta]
pub fn get_active_listening_session(app: AppHandle) -> Option<ActiveListeningSession> {
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    al_manager.get_current_session()
}

/// Check if Ollama server is available
#[tauri::command]
#[specta::specta]
pub async fn check_ollama_connection(app: AppHandle) -> Result<bool, String> {
    let settings = get_settings(&app);
    let client = OllamaClient::new(&settings.active_listening.ollama_base_url)?;
    client.health_check().await
}

/// Fetch available models from Ollama
#[tauri::command]
#[specta::specta]
pub async fn fetch_ollama_models(app: AppHandle) -> Result<Vec<OllamaModel>, String> {
    let settings = get_settings(&app);
    let client = OllamaClient::new(&settings.active_listening.ollama_base_url)?;

    let model_names = client.list_models().await?;
    let models: Vec<OllamaModel> = model_names
        .into_iter()
        .map(|name| OllamaModel { name })
        .collect();

    debug!("Found {} Ollama models", models.len());
    Ok(models)
}

// ---- Settings commands ----

/// Enable or disable active listening
#[tauri::command]
#[specta::specta]
pub fn change_active_listening_enabled_setting(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.enabled = enabled;
    write_settings(&app, settings);
    debug!("Active listening enabled: {}", enabled);
    Ok(())
}

/// Change the segment duration
#[tauri::command]
#[specta::specta]
pub fn change_active_listening_segment_duration_setting(
    app: AppHandle,
    duration_seconds: u32,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.segment_duration_seconds = duration_seconds;
    write_settings(&app, settings);
    debug!("Active listening segment duration: {}s", duration_seconds);
    Ok(())
}

/// Change the Ollama base URL
#[tauri::command]
#[specta::specta]
pub fn change_ollama_base_url_setting(app: AppHandle, base_url: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.ollama_base_url = base_url.clone();
    write_settings(&app, settings);
    debug!("Ollama base URL: {}", base_url);
    Ok(())
}

/// Change the Ollama model
#[tauri::command]
#[specta::specta]
pub fn change_ollama_model_setting(app: AppHandle, model: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.ollama_model = model.clone();
    write_settings(&app, settings);
    debug!("Ollama model: {}", model);
    Ok(())
}

/// Change the context window size
#[tauri::command]
#[specta::specta]
pub fn change_active_listening_context_window_setting(
    app: AppHandle,
    size: usize,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.context_window_size = size;
    write_settings(&app, settings);
    debug!("Active listening context window: {}", size);
    Ok(())
}

// ---- Audio Source Settings commands ----

/// Change the audio source type for active listening
#[tauri::command]
#[specta::specta]
pub fn change_audio_source_type_setting(
    app: AppHandle,
    source_type: AudioSourceType,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.audio_source_type = source_type;
    write_settings(&app, settings);
    debug!("Audio source type: {:?}", source_type);
    Ok(())
}

/// Change the audio mix ratio (0.0 = microphone only, 1.0 = system audio only)
#[tauri::command]
#[specta::specta]
pub fn change_audio_mix_ratio_setting(app: AppHandle, mix_ratio: f32) -> Result<(), String> {
    // Validate mix ratio is between 0.0 and 1.0
    if !(0.0..=1.0).contains(&mix_ratio) {
        return Err("Mix ratio must be between 0.0 and 1.0".to_string());
    }

    let mut settings = get_settings(&app);
    settings.active_listening.audio_mix_settings.mix_ratio = mix_ratio;
    write_settings(&app, settings);
    debug!("Audio mix ratio: {}", mix_ratio);
    Ok(())
}

/// Get the current audio source type
#[tauri::command]
#[specta::specta]
pub fn get_audio_source_type(app: AppHandle) -> AudioSourceType {
    let settings = get_settings(&app);
    settings.active_listening.audio_source_type
}

/// Get the current audio mix ratio
#[tauri::command]
#[specta::specta]
pub fn get_audio_mix_ratio(app: AppHandle) -> f32 {
    let settings = get_settings(&app);
    settings.active_listening.audio_mix_settings.mix_ratio
}

// ---- Prompt CRUD commands ----

/// Add a new active listening prompt
#[tauri::command]
#[specta::specta]
pub fn add_active_listening_prompt(
    app: AppHandle,
    name: String,
    prompt_template: String,
) -> Result<ActiveListeningPrompt, String> {
    let mut settings = get_settings(&app);

    let prompt = ActiveListeningPrompt {
        id: format!("al_prompt_{}", chrono::Utc::now().timestamp_millis()),
        name,
        prompt_template,
        created_at: chrono::Utc::now().timestamp_millis(),
        is_default: false,
        category: PromptCategory::Custom,
    };

    settings.active_listening.prompts.push(prompt.clone());
    write_settings(&app, settings);

    debug!("Created active listening prompt: {}", prompt.id);
    Ok(prompt)
}

/// Update an existing active listening prompt
#[tauri::command]
#[specta::specta]
pub fn update_active_listening_prompt(
    app: AppHandle,
    id: String,
    name: String,
    prompt_template: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app);

    let prompt = settings
        .active_listening
        .get_prompt_mut(&id)
        .ok_or_else(|| format!("Prompt not found: {}", id))?;

    prompt.name = name;
    prompt.prompt_template = prompt_template;

    write_settings(&app, settings);
    debug!("Updated active listening prompt: {}", id);
    Ok(())
}

/// Delete an active listening prompt
#[tauri::command]
#[specta::specta]
pub fn delete_active_listening_prompt(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = get_settings(&app);

    // Don't allow deleting the last prompt
    if settings.active_listening.prompts.len() <= 1 {
        return Err("Cannot delete the last prompt".to_string());
    }

    // Don't allow deleting default prompts
    if let Some(prompt) = settings.active_listening.get_prompt(&id) {
        if prompt.is_default {
            return Err("Cannot delete default prompts".to_string());
        }
    }

    settings.active_listening.prompts.retain(|p| p.id != id);

    // If the deleted prompt was selected, select another one
    if settings.active_listening.selected_prompt_id == Some(id.clone()) {
        settings.active_listening.selected_prompt_id = settings
            .active_listening
            .prompts
            .first()
            .map(|p| p.id.clone());
    }

    write_settings(&app, settings);
    debug!("Deleted active listening prompt: {}", id);
    Ok(())
}

/// Set the selected active listening prompt
#[tauri::command]
#[specta::specta]
pub fn set_active_listening_selected_prompt(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = get_settings(&app);

    // Verify the prompt exists
    if settings.active_listening.get_prompt(&id).is_none() {
        return Err(format!("Prompt not found: {}", id));
    }

    settings.active_listening.selected_prompt_id = Some(id.clone());
    write_settings(&app, settings);
    debug!("Selected active listening prompt: {}", id);
    Ok(())
}

// ---- Loopback/System Audio commands ----

/// Get the loopback support level for the current platform
#[tauri::command]
#[specta::specta]
pub fn get_loopback_support_level() -> LoopbackSupportLevel {
    match LoopbackCapture::support_level() {
        LoopbackSupport::Native => LoopbackSupportLevel::Native,
        LoopbackSupport::RequiresVirtualDevice => LoopbackSupportLevel::RequiresVirtualDevice,
        LoopbackSupport::NotSupported => LoopbackSupportLevel::NotSupported,
    }
}

/// Check if loopback capture is supported on this platform
#[tauri::command]
#[specta::specta]
pub fn is_loopback_supported() -> bool {
    LoopbackCapture::is_supported()
}

/// List available loopback devices
#[tauri::command]
#[specta::specta]
pub fn list_loopback_devices() -> Result<Vec<LoopbackDeviceInfoDto>, String> {
    match LoopbackCapture::list_devices() {
        Ok(devices) => {
            let dto_devices: Vec<LoopbackDeviceInfoDto> = devices
                .into_iter()
                .map(|d| LoopbackDeviceInfoDto {
                    id: d.id,
                    name: d.name,
                    is_default: d.is_default,
                })
                .collect();
            debug!("Found {} loopback devices", dto_devices.len());
            Ok(dto_devices)
        }
        Err(e) => {
            debug!("No loopback devices found: {}", e);
            Ok(Vec::new()) // Return empty list instead of error for better UX
        }
    }
}

// ---- Meeting Summary commands ----

/// Generate a comprehensive summary from a completed session
#[tauri::command]
#[specta::specta]
pub async fn generate_meeting_summary(
    app: AppHandle,
    session: ActiveListeningSession,
) -> Result<MeetingSummary, String> {
    let al_manager = app.state::<Arc<ActiveListeningManager>>();
    al_manager.generate_session_summary(&session).await
}

/// Export meeting summary to different formats
#[tauri::command]
#[specta::specta]
pub fn export_meeting_summary(
    summary: MeetingSummary,
    format: String,
) -> Result<String, String> {
    match format.as_str() {
        "markdown" => Ok(export_summary_to_markdown(&summary)),
        "text" => Ok(export_summary_to_text(&summary)),
        "json" => serde_json::to_string_pretty(&summary)
            .map_err(|e| format!("Failed to serialize summary: {}", e)),
        _ => Err(format!("Unsupported export format: {}", format)),
    }
}

/// Format summary as Markdown
fn export_summary_to_markdown(summary: &MeetingSummary) -> String {
    let mut md = String::new();

    md.push_str("# Meeting Summary\n\n");
    md.push_str(&format!("**Duration:** {} minutes\n\n", summary.duration_minutes));

    md.push_str("## Executive Summary\n\n");
    md.push_str(&summary.executive_summary);
    md.push_str("\n\n");

    if !summary.decisions.is_empty() {
        md.push_str("## Key Decisions\n\n");
        for decision in &summary.decisions {
            md.push_str(&format!("- {}\n", decision));
        }
        md.push('\n');
    }

    if !summary.action_items.is_empty() {
        md.push_str("## Action Items\n\n");
        for item in &summary.action_items {
            let mut line = format!("- [ ] {}", item.description);
            if let Some(assignee) = &item.assignee {
                line.push_str(&format!(" (@{})", assignee));
            }
            if let Some(deadline) = &item.deadline {
                line.push_str(&format!(" [Due: {}]", deadline));
            }
            md.push_str(&line);
            md.push('\n');
        }
        md.push('\n');
    }

    if !summary.topics.is_empty() {
        md.push_str("## Topics Discussed\n\n");
        for topic in &summary.topics {
            md.push_str(&format!("- {}\n", topic));
        }
        md.push('\n');
    }

    if !summary.follow_ups.is_empty() {
        md.push_str("## Follow-up Questions\n\n");
        for question in &summary.follow_ups {
            md.push_str(&format!("- {}\n", question));
        }
        md.push('\n');
    }

    md
}

/// Format summary as plain text
fn export_summary_to_text(summary: &MeetingSummary) -> String {
    let mut text = String::new();

    text.push_str("MEETING SUMMARY\n");
    text.push_str(&"=".repeat(50));
    text.push('\n');
    text.push_str(&format!("Duration: {} minutes\n\n", summary.duration_minutes));

    text.push_str("EXECUTIVE SUMMARY\n");
    text.push_str(&"-".repeat(30));
    text.push('\n');
    text.push_str(&summary.executive_summary);
    text.push_str("\n\n");

    if !summary.decisions.is_empty() {
        text.push_str("KEY DECISIONS\n");
        text.push_str(&"-".repeat(30));
        text.push('\n');
        for (i, decision) in summary.decisions.iter().enumerate() {
            text.push_str(&format!("{}. {}\n", i + 1, decision));
        }
        text.push('\n');
    }

    if !summary.action_items.is_empty() {
        text.push_str("ACTION ITEMS\n");
        text.push_str(&"-".repeat(30));
        text.push('\n');
        for (i, item) in summary.action_items.iter().enumerate() {
            let mut line = format!("{}. {}", i + 1, item.description);
            if let Some(assignee) = &item.assignee {
                line.push_str(&format!(" (Assignee: {})", assignee));
            }
            if let Some(deadline) = &item.deadline {
                line.push_str(&format!(" [Due: {}]", deadline));
            }
            text.push_str(&line);
            text.push('\n');
        }
        text.push('\n');
    }

    if !summary.topics.is_empty() {
        text.push_str("TOPICS DISCUSSED\n");
        text.push_str(&"-".repeat(30));
        text.push('\n');
        for (i, topic) in summary.topics.iter().enumerate() {
            text.push_str(&format!("{}. {}\n", i + 1, topic));
        }
        text.push('\n');
    }

    if !summary.follow_ups.is_empty() {
        text.push_str("FOLLOW-UP QUESTIONS\n");
        text.push_str(&"-".repeat(30));
        text.push('\n');
        for (i, question) in summary.follow_ups.iter().enumerate() {
            text.push_str(&format!("{}. {}\n", i + 1, question));
        }
        text.push('\n');
    }

    text
}
