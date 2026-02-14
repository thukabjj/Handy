use crate::managers::history::HistoryManager;
use crate::managers::task_extractor::{ActionItem, TaskExtractor};
use std::sync::Arc;

#[tauri::command]
#[specta::specta]
pub async fn extract_action_items(
    entry_id: i64,
    task_extractor: tauri::State<'_, std::sync::Mutex<TaskExtractor>>,
    history_manager: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<Vec<ActionItem>, String> {
    // Get transcript text from history entry
    let transcript = history_manager
        .get_entry_text(entry_id)
        .map_err(|e| format!("Failed to get transcript: {}", e))?
        .ok_or_else(|| "History entry not found".to_string())?;

    // Clone what we need from the extractor so we don't hold the MutexGuard across await
    let app_handle = {
        let extractor = task_extractor
            .lock()
            .map_err(|e| format!("Failed to lock task extractor: {}", e))?;
        extractor.app_handle().cloned()
    };

    let app = app_handle.ok_or("App handle not set on TaskExtractor")?;

    // Extract action items using LLM (without holding the lock)
    let items = crate::managers::task_extractor::extract_action_items_standalone(
        &app, &transcript, entry_id
    ).await?;

    // Store in database
    let stored_items = history_manager
        .insert_action_items(entry_id, &items)
        .map_err(|e| format!("Failed to store action items: {}", e))?;

    Ok(stored_items)
}

#[tauri::command]
#[specta::specta]
pub async fn get_action_items(
    entry_id: Option<i64>,
    history_manager: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<Vec<ActionItem>, String> {
    history_manager
        .get_action_items(entry_id)
        .map_err(|e| format!("Failed to get action items: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn toggle_action_item(
    id: i64,
    completed: bool,
    history_manager: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<(), String> {
    history_manager
        .toggle_action_item(id, completed)
        .map_err(|e| format!("Failed to toggle action item: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_action_item(
    id: i64,
    history_manager: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<(), String> {
    history_manager
        .delete_action_item(id)
        .map_err(|e| format!("Failed to delete action item: {}", e))
}

#[tauri::command]
#[specta::specta]
pub async fn export_action_items(
    entry_id: Option<i64>,
    format: String,
    history_manager: tauri::State<'_, Arc<HistoryManager>>,
) -> Result<String, String> {
    let items = history_manager
        .get_action_items(entry_id)
        .map_err(|e| format!("Failed to get action items: {}", e))?;

    match format.as_str() {
        "json" => serde_json::to_string_pretty(&items)
            .map_err(|e| format!("Failed to serialize: {}", e)),
        "markdown" | _ => {
            let mut md = String::from("# Action Items\n\n");
            for item in &items {
                let checkbox = if item.completed { "x" } else { " " };
                md.push_str(&format!("- [{}] {}", checkbox, item.task));
                if let Some(ref assignee) = item.assignee {
                    md.push_str(&format!(" (@{})", assignee));
                }
                if let Some(ref deadline) = item.deadline {
                    md.push_str(&format!(" [{}]", deadline));
                }
                md.push_str(&format!(" ({})\n", item.priority));
            }
            Ok(md)
        }
    }
}
