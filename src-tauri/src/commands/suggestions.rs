//! Commands for the Suggestions feature
//!
//! Provides Tauri commands for managing quick responses and suggestions settings.

use crate::managers::suggestion_engine::SuggestionEngine;
use crate::settings::{get_settings, write_settings, QuickResponse, SuggestionsSettings};
use tauri::{AppHandle, Manager};

/// Get the current suggestions settings
#[tauri::command]
#[specta::specta]
pub fn get_suggestions_settings(app: AppHandle) -> Result<SuggestionsSettings, String> {
    let settings = get_settings(&app);
    Ok(settings.suggestions)
}

/// Update the suggestions settings
#[tauri::command]
#[specta::specta]
pub async fn update_suggestions_settings(
    app: AppHandle,
    settings: SuggestionsSettings,
) -> Result<(), String> {
    // Update persistent settings
    let mut app_settings = get_settings(&app);
    app_settings.suggestions = settings.clone();
    write_settings(&app, app_settings);

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings).await;
    }

    Ok(())
}

/// Enable or disable the suggestions feature
#[tauri::command]
#[specta::specta]
pub async fn change_suggestions_enabled_setting(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.enabled = enabled;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Get all quick responses
#[tauri::command]
#[specta::specta]
pub fn get_quick_responses(app: AppHandle) -> Result<Vec<QuickResponse>, String> {
    let settings = get_settings(&app);
    Ok(settings.suggestions.quick_responses)
}

/// Get quick responses by category
#[tauri::command]
#[specta::specta]
pub fn get_quick_responses_by_category(
    app: AppHandle,
    category: String,
) -> Result<Vec<QuickResponse>, String> {
    let settings = get_settings(&app);
    let filtered: Vec<QuickResponse> = settings
        .suggestions
        .quick_responses
        .into_iter()
        .filter(|qr| qr.category == category && qr.enabled)
        .collect();
    Ok(filtered)
}

/// Add a new quick response
#[tauri::command]
#[specta::specta]
pub async fn add_quick_response(
    app: AppHandle,
    response: QuickResponse,
) -> Result<QuickResponse, String> {
    let mut settings = get_settings(&app);

    // Check for duplicate ID
    if settings
        .suggestions
        .quick_responses
        .iter()
        .any(|qr| qr.id == response.id)
    {
        return Err("Quick response with this ID already exists".to_string());
    }

    settings.suggestions.quick_responses.push(response.clone());
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.add_quick_response(response.clone()).await;
    }

    Ok(response)
}

/// Update an existing quick response
#[tauri::command]
#[specta::specta]
pub async fn update_quick_response(
    app: AppHandle,
    response: QuickResponse,
) -> Result<QuickResponse, String> {
    let mut settings = get_settings(&app);

    // Find and update the quick response
    if let Some(existing) = settings
        .suggestions
        .quick_responses
        .iter_mut()
        .find(|qr| qr.id == response.id)
    {
        *existing = response.clone();
        write_settings(&app, settings.clone());

        // Update the suggestion engine if it exists
        if let Some(engine) = app.try_state::<SuggestionEngine>() {
            engine.update_quick_response(response.clone()).await;
        }

        Ok(response)
    } else {
        Err("Quick response not found".to_string())
    }
}

/// Delete a quick response
#[tauri::command]
#[specta::specta]
pub async fn delete_quick_response(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = get_settings(&app);
    let initial_len = settings.suggestions.quick_responses.len();

    settings
        .suggestions
        .quick_responses
        .retain(|qr| qr.id != id);

    if settings.suggestions.quick_responses.len() < initial_len {
        write_settings(&app, settings);

        // Update the suggestion engine if it exists
        if let Some(engine) = app.try_state::<SuggestionEngine>() {
            engine.delete_quick_response(&id).await;
        }

        Ok(())
    } else {
        Err("Quick response not found".to_string())
    }
}

/// Toggle a quick response's enabled state
#[tauri::command]
#[specta::specta]
pub async fn toggle_quick_response(app: AppHandle, id: String) -> Result<bool, String> {
    let mut settings = get_settings(&app);

    if let Some(qr) = settings
        .suggestions
        .quick_responses
        .iter_mut()
        .find(|qr| qr.id == id)
    {
        qr.enabled = !qr.enabled;
        let new_state = qr.enabled;
        write_settings(&app, settings);

        // Update the suggestion engine if it exists
        if let Some(engine) = app.try_state::<SuggestionEngine>() {
            engine.toggle_quick_response(&id).await;
        }

        Ok(new_state)
    } else {
        Err("Quick response not found".to_string())
    }
}

/// Enable or disable RAG suggestions
#[tauri::command]
#[specta::specta]
pub async fn change_rag_suggestions_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.rag_suggestions_enabled = enabled;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Enable or disable LLM suggestions
#[tauri::command]
#[specta::specta]
pub async fn change_llm_suggestions_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.llm_suggestions_enabled = enabled;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Update max suggestions count
#[tauri::command]
#[specta::specta]
pub async fn change_max_suggestions(app: AppHandle, max_suggestions: usize) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.max_suggestions = max_suggestions;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Update minimum confidence threshold
#[tauri::command]
#[specta::specta]
pub async fn change_min_confidence(app: AppHandle, min_confidence: f32) -> Result<(), String> {
    if !(0.0..=1.0).contains(&min_confidence) {
        return Err("Confidence must be between 0.0 and 1.0".to_string());
    }

    let mut settings = get_settings(&app);
    settings.suggestions.min_confidence = min_confidence;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Update auto-dismiss on copy setting
#[tauri::command]
#[specta::specta]
pub async fn change_auto_dismiss_on_copy(
    app: AppHandle,
    auto_dismiss: bool,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.auto_dismiss_on_copy = auto_dismiss;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}

/// Update display duration setting
#[tauri::command]
#[specta::specta]
pub async fn change_display_duration(
    app: AppHandle,
    duration_seconds: u32,
) -> Result<(), String> {
    let mut settings = get_settings(&app);
    settings.suggestions.display_duration_seconds = duration_seconds;
    write_settings(&app, settings.clone());

    // Update the suggestion engine if it exists
    if let Some(engine) = app.try_state::<SuggestionEngine>() {
        engine.update_settings(settings.suggestions).await;
    }

    Ok(())
}
