use crate::managers::vocabulary::{VocabularyEntry, VocabularyManager};

#[tauri::command]
#[specta::specta]
pub async fn get_vocabulary(
    vocabulary_manager: tauri::State<'_, std::sync::Mutex<VocabularyManager>>,
) -> Result<Vec<VocabularyEntry>, String> {
    let manager = vocabulary_manager
        .lock()
        .map_err(|e| format!("Failed to lock vocabulary manager: {}", e))?;
    manager.get_vocabulary()
}

#[tauri::command]
#[specta::specta]
pub async fn add_vocabulary_term(
    term: String,
    category: Option<String>,
    vocabulary_manager: tauri::State<'_, std::sync::Mutex<VocabularyManager>>,
) -> Result<VocabularyEntry, String> {
    let manager = vocabulary_manager
        .lock()
        .map_err(|e| format!("Failed to lock vocabulary manager: {}", e))?;
    manager.add_term(&term, category.as_deref())
}

#[tauri::command]
#[specta::specta]
pub async fn remove_vocabulary_term(
    id: i64,
    vocabulary_manager: tauri::State<'_, std::sync::Mutex<VocabularyManager>>,
) -> Result<(), String> {
    let manager = vocabulary_manager
        .lock()
        .map_err(|e| format!("Failed to lock vocabulary manager: {}", e))?;
    manager.remove_term(id)
}

#[tauri::command]
#[specta::specta]
pub async fn import_vocabulary(
    json: String,
    vocabulary_manager: tauri::State<'_, std::sync::Mutex<VocabularyManager>>,
) -> Result<usize, String> {
    let manager = vocabulary_manager
        .lock()
        .map_err(|e| format!("Failed to lock vocabulary manager: {}", e))?;
    manager.import_vocabulary(&json)
}

#[tauri::command]
#[specta::specta]
pub async fn export_vocabulary(
    vocabulary_manager: tauri::State<'_, std::sync::Mutex<VocabularyManager>>,
) -> Result<String, String> {
    let manager = vocabulary_manager
        .lock()
        .map_err(|e| format!("Failed to lock vocabulary manager: {}", e))?;
    manager.export_vocabulary()
}
