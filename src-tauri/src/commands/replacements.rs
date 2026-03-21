//! Tauri commands for managing text replacements.

use crate::managers::replacements::TextReplacement;
use crate::settings::{get_settings, write_settings};
use tauri::AppHandle;

/// Get all text replacements.
#[tauri::command]
#[specta::specta]
pub fn get_text_replacements(app: AppHandle) -> Vec<TextReplacement> {
    get_settings(&app).text_replacements
}

/// Add a new text replacement.
#[tauri::command]
#[specta::specta]
pub fn add_text_replacement(
    app: AppHandle,
    pattern: String,
    replacement: String,
    is_regex: bool,
    case_insensitive: bool,
    description: Option<String>,
) -> Result<TextReplacement, String> {
    let mut settings = get_settings(&app);

    let mut text_replacement = TextReplacement::new(pattern, replacement);
    text_replacement.is_regex = is_regex;
    text_replacement.case_insensitive = case_insensitive;
    text_replacement.description = description;

    // Validate regex if it's a regex pattern
    if is_regex {
        if let Err(e) = regex::Regex::new(&text_replacement.pattern) {
            return Err(format!("Invalid regex pattern: {}", e));
        }
    }

    settings.text_replacements.push(text_replacement.clone());
    write_settings(&app, settings);

    Ok(text_replacement)
}

/// Update an existing text replacement.
#[tauri::command]
#[specta::specta]
#[allow(clippy::too_many_arguments)]
pub fn update_text_replacement(
    app: AppHandle,
    id: String,
    pattern: Option<String>,
    replacement: Option<String>,
    is_regex: Option<bool>,
    case_insensitive: Option<bool>,
    enabled: Option<bool>,
    description: Option<String>,
) -> Result<TextReplacement, String> {
    let mut settings = get_settings(&app);

    let text_replacement = settings
        .text_replacements
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Text replacement with id {} not found", id))?;

    if let Some(p) = pattern {
        text_replacement.pattern = p;
    }
    if let Some(r) = replacement {
        text_replacement.replacement = r;
    }
    if let Some(ir) = is_regex {
        text_replacement.is_regex = ir;
    }
    if let Some(ci) = case_insensitive {
        text_replacement.case_insensitive = ci;
    }
    if let Some(e) = enabled {
        text_replacement.enabled = e;
    }
    if let Some(d) = description {
        text_replacement.description = Some(d);
    }

    // Validate regex if it's a regex pattern
    if text_replacement.is_regex {
        if let Err(e) = regex::Regex::new(&text_replacement.pattern) {
            return Err(format!("Invalid regex pattern: {}", e));
        }
    }

    let updated = text_replacement.clone();
    write_settings(&app, settings);

    Ok(updated)
}

/// Delete a text replacement.
#[tauri::command]
#[specta::specta]
pub fn delete_text_replacement(app: AppHandle, id: String) -> Result<(), String> {
    let mut settings = get_settings(&app);

    let original_len = settings.text_replacements.len();
    settings.text_replacements.retain(|r| r.id != id);

    if settings.text_replacements.len() == original_len {
        return Err(format!("Text replacement with id {} not found", id));
    }

    write_settings(&app, settings);
    Ok(())
}

/// Toggle a text replacement's enabled state.
#[tauri::command]
#[specta::specta]
pub fn toggle_text_replacement(app: AppHandle, id: String) -> Result<bool, String> {
    let mut settings = get_settings(&app);

    let text_replacement = settings
        .text_replacements
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("Text replacement with id {} not found", id))?;

    text_replacement.enabled = !text_replacement.enabled;
    let new_state = text_replacement.enabled;

    write_settings(&app, settings);
    Ok(new_state)
}

/// Change the text replacements enabled setting.
#[tauri::command]
#[specta::specta]
pub fn change_text_replacements_enabled_setting(app: AppHandle, enabled: bool) {
    let mut settings = get_settings(&app);
    settings.text_replacements_enabled = enabled;
    write_settings(&app, settings);
}

/// Test a replacement pattern against sample text.
#[tauri::command]
#[specta::specta]
pub fn test_text_replacement(
    pattern: String,
    replacement: String,
    is_regex: bool,
    case_insensitive: bool,
    sample_text: String,
) -> Result<String, String> {
    let mut text_replacement = TextReplacement::new(pattern, replacement);
    text_replacement.is_regex = is_regex;
    text_replacement.case_insensitive = case_insensitive;

    // Validate regex
    if is_regex {
        if let Err(e) = regex::Regex::new(&text_replacement.pattern) {
            return Err(format!("Invalid regex pattern: {}", e));
        }
    }

    Ok(text_replacement.apply(&sample_text))
}

/// Reorder text replacements (move a replacement to a new position).
#[tauri::command]
#[specta::specta]
pub fn reorder_text_replacement(
    app: AppHandle,
    id: String,
    new_index: usize,
) -> Result<(), String> {
    let mut settings = get_settings(&app);

    let current_index = settings
        .text_replacements
        .iter()
        .position(|r| r.id == id)
        .ok_or_else(|| format!("Text replacement with id {} not found", id))?;

    if new_index >= settings.text_replacements.len() {
        return Err("New index out of bounds".to_string());
    }

    let replacement = settings.text_replacements.remove(current_index);
    settings.text_replacements.insert(new_index, replacement);

    write_settings(&app, settings);
    Ok(())
}

/// Import text replacements from JSON.
#[tauri::command]
#[specta::specta]
pub fn import_text_replacements(
    app: AppHandle,
    json: String,
    merge: bool,
) -> Result<usize, String> {
    let imported: Vec<TextReplacement> =
        serde_json::from_str(&json).map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut settings = get_settings(&app);

    if !merge {
        settings.text_replacements.clear();
    }

    let count = imported.len();
    for mut replacement in imported {
        // Generate new IDs to avoid conflicts when merging
        if merge {
            replacement.id = uuid::Uuid::new_v4().to_string();
        }
        settings.text_replacements.push(replacement);
    }

    write_settings(&app, settings);
    Ok(count)
}

/// Export text replacements to JSON.
#[tauri::command]
#[specta::specta]
pub fn export_text_replacements(app: AppHandle) -> Result<String, String> {
    let settings = get_settings(&app);
    serde_json::to_string_pretty(&settings.text_replacements)
        .map_err(|e| format!("Failed to serialize: {}", e))
}
