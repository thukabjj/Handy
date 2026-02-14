use crate::managers::batch_processor::{BatchProcessor, BatchQueueStatus};
use std::path::PathBuf;
use tauri::AppHandle;
use tokio::sync::Mutex;

#[tauri::command]
#[specta::specta]
pub async fn add_to_batch_queue(
    paths: Vec<String>,
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<BatchQueueStatus, String> {
    let file_paths: Vec<PathBuf> = paths.into_iter().map(PathBuf::from).collect();
    let processor = batch_processor.lock().await;
    processor.add_files(file_paths).await
}

#[tauri::command]
#[specta::specta]
pub async fn start_batch_processing(
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<(), String> {
    let processor = batch_processor.lock().await;
    processor.process_queue().await
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_batch_processing(
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<(), String> {
    let processor = batch_processor.lock().await;
    processor.cancel();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_batch_status(
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<BatchQueueStatus, String> {
    let processor = batch_processor.lock().await;
    Ok(processor.get_status().await)
}

#[tauri::command]
#[specta::specta]
pub async fn remove_batch_item(
    id: String,
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<(), String> {
    let processor = batch_processor.lock().await;
    processor.remove_item(&id).await
}

#[tauri::command]
#[specta::specta]
pub async fn clear_completed_batch_items(
    batch_processor: tauri::State<'_, Mutex<BatchProcessor>>,
) -> Result<(), String> {
    let processor = batch_processor.lock().await;
    processor.clear_completed().await;
    Ok(())
}
