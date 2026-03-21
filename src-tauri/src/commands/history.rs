use crate::actions::post_process_transcription;
use crate::managers::history::{HistoryEntry, HistoryManager};
use crate::settings::get_settings;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Txt,
    Srt,
    Vtt,
    Json,
    Markdown,
}

#[tauri::command]
#[specta::specta]
pub async fn get_history_entries(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
) -> Result<Vec<HistoryEntry>, String> {
    history_manager
        .get_history_entries()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_history_entry_saved(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .toggle_saved_status(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn get_audio_file_path(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    file_name: String,
) -> Result<String, String> {
    let path = history_manager.get_audio_file_path(&file_name);
    path.to_str()
        .ok_or_else(|| "Invalid file path".to_string())
        .map(|s| s.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<(), String> {
    history_manager
        .delete_entry(id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn update_history_limit(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    limit: usize,
) -> Result<(), String> {
    let mut settings = crate::settings::get_settings(&app);
    settings.history_limit = limit;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn update_recording_retention_period(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    period: String,
) -> Result<(), String> {
    use crate::settings::RecordingRetentionPeriod;

    let retention_period = match period.as_str() {
        "never" => RecordingRetentionPeriod::Never,
        "preserve_limit" => RecordingRetentionPeriod::PreserveLimit,
        "days3" => RecordingRetentionPeriod::Days3,
        "weeks2" => RecordingRetentionPeriod::Weeks2,
        "months3" => RecordingRetentionPeriod::Months3,
        _ => return Err(format!("Invalid retention period: {}", period)),
    };

    let mut settings = crate::settings::get_settings(&app);
    settings.recording_retention_period = retention_period;
    crate::settings::write_settings(&app, settings);

    history_manager
        .cleanup_old_entries()
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Post-process a history entry using the current LLM settings.
/// Returns the processed text if successful.
#[allow(dead_code)]
#[tauri::command]
#[specta::specta]
pub async fn post_process_history_entry(
    app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<String, String> {
    // Get the entry
    let entry = history_manager
        .get_entry_by_id(id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("History entry with id {} not found", id))?;

    // Get current settings for post-processing config
    let settings = get_settings(&app);

    // Use the original transcription text for post-processing
    let text_to_process = &entry.transcription_text;

    // Post-process using the configured LLM
    let processed_text = post_process_transcription(&settings, text_to_process)
        .await
        .ok_or_else(|| {
            "Post-processing failed. Please check your LLM provider settings.".to_string()
        })?;

    // Get the prompt that was used
    let post_process_prompt =
        settings
            .post_process_selected_prompt_id
            .as_ref()
            .and_then(|prompt_id| {
                settings
                    .post_process_prompts
                    .iter()
                    .find(|p| &p.id == prompt_id)
                    .map(|p| p.prompt.clone())
            });

    // Update the entry with the processed text
    history_manager
        .update_post_processed_text(id, processed_text.clone(), post_process_prompt)
        .await
        .map_err(|e| e.to_string())?;

    Ok(processed_text)
}

/// Get a specific history entry by ID.
#[allow(dead_code)]
#[tauri::command]
#[specta::specta]
pub async fn get_history_entry_by_id(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    id: i64,
) -> Result<Option<HistoryEntry>, String> {
    history_manager
        .get_entry_by_id(id)
        .await
        .map_err(|e| e.to_string())
}

/// Get a suggested filename for exporting a single history entry.
#[tauri::command]
#[specta::specta]
pub async fn get_export_filename(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_id: i64,
    format: ExportFormat,
) -> Result<String, String> {
    let entry = history_manager
        .get_entry_by_id(entry_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Entry {} not found", entry_id))?;

    let ext = match format {
        ExportFormat::Txt => "txt",
        ExportFormat::Srt => "srt",
        ExportFormat::Vtt => "vtt",
        ExportFormat::Json => "json",
        ExportFormat::Markdown => "md",
    };
    Ok(format!("transcription-{}.{}", entry.timestamp, ext))
}

/// Export a single transcription entry in the given format.
#[tauri::command]
#[specta::specta]
pub async fn export_transcription(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_id: i64,
    format: ExportFormat,
) -> Result<String, String> {
    let entry = history_manager
        .get_entry_by_id(entry_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Entry {} not found", entry_id))?;

    format_entry(&entry, format)
}

/// Get a suggested filename for a batch export.
#[tauri::command]
#[specta::specta]
pub fn get_batch_export_filename(format: ExportFormat) -> String {
    let ext = match format {
        ExportFormat::Txt => "txt",
        ExportFormat::Srt => "srt",
        ExportFormat::Vtt => "vtt",
        ExportFormat::Json => "json",
        ExportFormat::Markdown => "md",
    };
    let now = chrono::Local::now().format("%Y-%m-%d");
    format!("transcriptions-{}.{}", now, ext)
}

/// Export multiple transcription entries in the given format.
#[tauri::command]
#[specta::specta]
pub async fn export_transcriptions(
    _app: AppHandle,
    history_manager: State<'_, Arc<HistoryManager>>,
    entry_ids: Vec<i64>,
    format: ExportFormat,
) -> Result<String, String> {
    let mut parts = Vec::new();
    for id in entry_ids {
        if let Some(entry) = history_manager
            .get_entry_by_id(id)
            .await
            .map_err(|e| e.to_string())?
        {
            parts.push(format_entry(&entry, format)?);
        }
    }

    match format {
        ExportFormat::Json => Ok(format!("[{}]", parts.join(","))),
        _ => Ok(parts.join("\n\n---\n\n")),
    }
}

fn format_entry(entry: &HistoryEntry, format: ExportFormat) -> Result<String, String> {
    let text = entry
        .post_processed_text
        .as_deref()
        .unwrap_or(&entry.transcription_text);

    match format {
        ExportFormat::Txt => Ok(text.to_string()),
        ExportFormat::Markdown => Ok(format!(
            "# Transcription - {}\n\n{}\n",
            entry.timestamp, text
        )),
        ExportFormat::Json => serde_json::to_string_pretty(&serde_json::json!({
            "id": entry.id,
            "text": text,
            "timestamp": entry.timestamp,
            "title": entry.title,
        }))
        .map_err(|e| e.to_string()),
        ExportFormat::Srt => Ok(format!("1\n00:00:00,000 --> 00:00:00,000\n{}\n", text)),
        ExportFormat::Vtt => Ok(format!(
            "WEBVTT\n\n00:00:00.000 --> 00:00:00.000\n{}\n",
            text
        )),
    }
}
