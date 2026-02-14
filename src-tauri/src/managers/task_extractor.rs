use log::{debug, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::AppHandle;

use crate::ollama_client::OllamaClient;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ActionItem {
    pub id: i64,
    pub entry_id: i64,
    pub task: String,
    pub assignee: Option<String>,
    pub deadline: Option<String>,
    pub priority: String,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
struct RawActionItem {
    task: Option<String>,
    assignee: Option<String>,
    deadline: Option<String>,
    priority: Option<String>,
}

pub struct TaskExtractor {
    app_handle: Option<AppHandle>,
}

impl TaskExtractor {
    pub fn new() -> Self {
        Self { app_handle: None }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub fn app_handle(&self) -> Option<&AppHandle> {
        self.app_handle.as_ref()
    }
}

/// Standalone extraction function that doesn't require holding a lock across await points.
pub async fn extract_action_items_standalone(
    app: &AppHandle,
    transcript: &str,
    entry_id: i64,
) -> Result<Vec<ActionItem>, String> {
    let settings = crate::settings::get_settings(app);
    let ollama_url = &settings.active_listening.ollama_base_url;
    let model = &settings.active_listening.ollama_model;

    let client = OllamaClient::new(ollama_url)
        .map_err(|e| format!("Failed to create Ollama client: {}", e))?;

    let health = client
        .health_check()
        .await
        .map_err(|e| format!("Ollama health check failed: {}", e))?;

    if !health {
        return Err(
            "Ollama server is not available. Please ensure Ollama is running.".to_string(),
        );
    }

    let prompt = format!(
        r#"Analyze this transcript and extract action items. For each action item, identify:
- task: The specific action to be taken
- assignee: Who should do it (if mentioned, otherwise null)
- deadline: When it should be done (if mentioned, otherwise null)
- priority: "high", "medium", or "low" based on urgency

Return ONLY a JSON array. Example:
[{{"task": "Send the report", "assignee": "John", "deadline": "Friday", "priority": "high"}}]

If no action items found, return: []

Transcript:
{}"#,
        transcript
    );

    debug!("Extracting action items using model: {}", model);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);
    client
        .generate_stream(model, prompt, tx)
        .await
        .map_err(|e| format!("Failed to generate: {}", e))?;

    let mut full_response = String::new();
    while let Some(chunk) = rx.recv().await {
        full_response.push_str(&chunk);
    }

    debug!(
        "Ollama response: {}",
        &full_response[..full_response.len().min(200)]
    );

    // Try to extract JSON array from the response
    let json_str = extract_json_array(&full_response)
        .ok_or_else(|| "Could not find JSON array in Ollama response".to_string())?;

    let raw_items: Vec<RawActionItem> = serde_json::from_str(&json_str)
        .map_err(|e| format!("Failed to parse action items JSON: {}", e))?;

    let items: Vec<ActionItem> = raw_items
        .into_iter()
        .filter_map(|raw| {
            let task = raw.task?;
            if task.trim().is_empty() {
                return None;
            }
            Some(ActionItem {
                id: 0,
                entry_id,
                task,
                assignee: raw.assignee.filter(|s| !s.trim().is_empty()),
                deadline: raw.deadline.filter(|s| !s.trim().is_empty()),
                priority: raw
                    .priority
                    .unwrap_or_else(|| "medium".to_string())
                    .to_lowercase(),
                completed: false,
                created_at: chrono::Utc::now().to_rfc3339(),
            })
        })
        .collect();

    info!("Extracted {} action items from transcript", items.len());
    Ok(items)
}

fn extract_json_array(text: &str) -> Option<String> {
    // Find the first '[' and matching ']'
    let start = text.find('[')?;
    let mut depth = 0;
    for (i, c) in text[start..].char_indices() {
        match c {
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[start..start + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}
