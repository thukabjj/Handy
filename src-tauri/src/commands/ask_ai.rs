//! Tauri commands for Ask AI feature

use crate::managers::ask_ai::{AskAiConversation, AskAiManager, AskAiState};
use crate::managers::ask_ai_history::AskAiHistoryManager;
use crate::overlay::{hide_recording_overlay, reset_overlay_size};
use crate::settings::active_listening::LlmProvider;
use crate::settings::{get_settings, write_settings};
use log::debug;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

/// Get the current Ask AI state
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_state(app: AppHandle) -> AskAiState {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.get_state()
}

/// Check if an Ask AI session is currently active
#[tauri::command]
#[specta::specta]
pub fn is_ask_ai_active(app: AppHandle) -> bool {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.is_active()
}

/// Get the current question (if any)
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_question(app: AppHandle) -> Option<String> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.get_question()
}

/// Get the current response text
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_response(app: AppHandle) -> String {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.get_response()
}

/// Get the current conversation
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_conversation(app: AppHandle) -> Option<AskAiConversation> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.get_conversation()
}

/// Check if we can start a new recording (idle, complete, or conversation active)
#[tauri::command]
#[specta::specta]
pub fn can_start_ask_ai_recording(app: AppHandle) -> bool {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.can_start_recording()
}

/// Cancel the current Ask AI session
#[tauri::command]
#[specta::specta]
pub fn cancel_ask_ai_session(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.cancel();
    reset_overlay_size(&app);
    hide_recording_overlay(&app);
    debug!("Ask AI session cancelled via command");
    Ok(())
}

/// Reset Ask AI to idle state and clear conversation
#[tauri::command]
#[specta::specta]
pub fn reset_ask_ai_session(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.reset();
    reset_overlay_size(&app);
    hide_recording_overlay(&app);
    debug!("Ask AI session reset via command");
    Ok(())
}

/// Dismiss the Ask AI overlay but keep conversation for potential resume
#[tauri::command]
#[specta::specta]
pub fn dismiss_ask_ai_session(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.dismiss();
    reset_overlay_size(&app);
    hide_recording_overlay(&app);
    debug!("Ask AI session dismissed via command");
    Ok(())
}

/// Start a new conversation (clears existing conversation)
#[tauri::command]
#[specta::specta]
pub fn start_new_ask_ai_conversation(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiManager>>();
    manager.start_new_conversation()?;
    debug!("Started new Ask AI conversation via command");
    Ok(())
}

/// Enable or disable Ask AI feature
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_enabled_setting(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.ask_ai.enabled = enabled;
    write_settings(&app, settings);
    debug!("Ask AI enabled: {}", enabled);
    Ok(())
}

/// Change Ask AI Ollama base URL
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_ollama_base_url_setting(
    app: AppHandle,
    base_url: String,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.ollama_base_url = base_url.clone();
    settings.ask_ai.ollama_base_url = base_url.clone();
    write_settings(&app, settings);
    debug!(
        "Centralized Ollama base URL updated via Ask AI command: {}",
        base_url
    );
    Ok(())
}

/// Change Ask AI Ollama model
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_ollama_model_setting(app: AppHandle, model: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.ask_ai.ollama_model = model.clone();
    write_settings(&app, settings);
    debug!("Ask AI Ollama model changed to: {}", model);
    Ok(())
}

// ---- LLM Provider Settings commands ----

/// Change the LLM provider for Ask AI
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_llm_provider(app: AppHandle, provider: LlmProvider) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.active_listening.llm_provider = provider.clone();
    settings.ask_ai.llm_provider = provider;
    write_settings(&app, settings);
    debug!("Centralized LLM provider updated via Ask AI command");
    Ok(())
}

/// Change the LLM API key for Ask AI
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_llm_api_key(app: AppHandle, api_key: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    let normalized = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    settings.active_listening.llm_api_key = normalized.clone();
    settings.ask_ai.llm_api_key = normalized;
    write_settings(&app, settings);
    debug!("Centralized LLM API key updated via Ask AI command");
    Ok(())
}

/// Change the LLM model for Ask AI (cloud provider)
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_llm_model(app: AppHandle, model: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.ask_ai.llm_model = if model.is_empty() {
        None
    } else {
        Some(model.clone())
    };
    write_settings(&app, settings);
    debug!("Ask AI LLM model: {}", model);
    Ok(())
}

/// Change the custom LLM base URL for Ask AI
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_llm_base_url(app: AppHandle, base_url: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    let normalized = if base_url.is_empty() {
        None
    } else {
        Some(base_url.clone())
    };
    settings.active_listening.llm_base_url = normalized.clone();
    settings.ask_ai.llm_base_url = normalized;
    write_settings(&app, settings);
    debug!("Centralized LLM base URL updated via Ask AI command: {}", base_url);
    Ok(())
}

/// Change Ask AI system prompt
#[tauri::command]
#[specta::specta]
pub fn change_ask_ai_system_prompt_setting(app: AppHandle, prompt: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.ask_ai.system_prompt = prompt.clone();
    write_settings(&app, settings);
    debug!("Ask AI system prompt updated");
    Ok(())
}

/// Get Ask AI settings (for display in UI)
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_settings(app: AppHandle) -> crate::settings::ask_ai::AskAiSettings {
    let settings = get_settings(&app);
    settings.ask_ai
}

/// Window position and size for Ask AI overlay
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct AskAiWindowBounds {
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub x: Option<f64>,
    pub y: Option<f64>,
}

/// Save Ask AI window position and size
#[tauri::command]
#[specta::specta]
pub fn save_ask_ai_window_bounds(app: AppHandle, bounds: AskAiWindowBounds) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.ask_ai.window_width = bounds.width;
    settings.ask_ai.window_height = bounds.height;
    settings.ask_ai.window_x = bounds.x;
    settings.ask_ai.window_y = bounds.y;
    write_settings(&app, settings);
    debug!("Ask AI window bounds saved: {:?}", bounds);
    Ok(())
}

/// Get Ask AI window position and size
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_window_bounds(app: AppHandle) -> AskAiWindowBounds {
    let settings = get_settings(&app);
    AskAiWindowBounds {
        width: settings.ask_ai.window_width,
        height: settings.ask_ai.window_height,
        x: settings.ask_ai.window_x,
        y: settings.ask_ai.window_y,
    }
}

// ============ Ask AI History Commands ============

/// Save a conversation to history
#[tauri::command]
#[specta::specta]
pub fn save_ask_ai_conversation_to_history(
    app: AppHandle,
    conversation: AskAiConversation,
) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiHistoryManager>>();
    manager
        .save_conversation(&conversation)
        .map_err(|e| format!("Failed to save conversation: {}", e))?;
    debug!("Saved Ask AI conversation {} to history", conversation.id);
    Ok(())
}

/// List recent Ask AI conversations from history
#[tauri::command]
#[specta::specta]
pub fn list_ask_ai_conversations(
    app: AppHandle,
    limit: usize,
) -> Result<Vec<AskAiConversation>, String> {
    let manager = app.state::<Arc<AskAiHistoryManager>>();
    manager
        .list_conversations(limit)
        .map_err(|e| format!("Failed to list conversations: {}", e))
}

/// Get a specific Ask AI conversation from history
#[tauri::command]
#[specta::specta]
pub fn get_ask_ai_conversation_from_history(
    app: AppHandle,
    id: String,
) -> Result<Option<AskAiConversation>, String> {
    let manager = app.state::<Arc<AskAiHistoryManager>>();
    manager
        .get_conversation(&id)
        .map_err(|e| format!("Failed to get conversation: {}", e))
}

/// Delete an Ask AI conversation from history
#[tauri::command]
#[specta::specta]
pub fn delete_ask_ai_conversation_from_history(app: AppHandle, id: String) -> Result<(), String> {
    let manager = app.state::<Arc<AskAiHistoryManager>>();
    manager
        .delete_conversation(&id)
        .map_err(|e| format!("Failed to delete conversation: {}", e))?;
    debug!("Deleted Ask AI conversation {} from history", id);
    Ok(())
}
