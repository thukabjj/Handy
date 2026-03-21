use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::audio_toolkit::decoder;
use crate::managers::history::HistoryManager;
use crate::managers::transcription::TranscriptionManager;
use crate::telemetry::{TelemetryEventBuilder, TelemetryLevel};

#[derive(Debug, Clone, Serialize, Deserialize, Type, PartialEq)]
pub enum JobStatus {
    Queued,
    Decoding,
    Transcribing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchItem {
    pub id: String,
    pub file_name: String,
    pub file_path: String,
    pub status: JobStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchQueueStatus {
    pub items: Vec<BatchItem>,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub is_processing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BatchProgressEvent {
    pub id: String,
    pub status: JobStatus,
    pub progress: f32,
    pub error: Option<String>,
    pub duration_seconds: Option<f64>,
    pub total_items: usize,
    pub completed_items: usize,
}

pub struct BatchProcessor {
    queue: Arc<Mutex<VecDeque<BatchItem>>>,
    cancel_signal: Arc<AtomicBool>,
    is_processing: Arc<AtomicBool>,
    app_handle: Option<AppHandle>,
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            cancel_signal: Arc::new(AtomicBool::new(false)),
            is_processing: Arc::new(AtomicBool::new(false)),
            app_handle: None,
        }
    }

    #[allow(dead_code)]
    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    pub async fn add_files(&self, paths: Vec<PathBuf>) -> Result<BatchQueueStatus, String> {
        let mut queue = self.queue.lock().await;
        let requested = paths.len();
        let mut added = 0usize;
        let mut skipped = 0usize;

        for path in paths {
            if !decoder::is_supported_format(&path) {
                warn!("Skipping unsupported file: {}", path.display());
                skipped += 1;
                continue;
            }

            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();

            let item = BatchItem {
                id: Uuid::new_v4().to_string(),
                file_name,
                file_path: path.to_string_lossy().to_string(),
                status: JobStatus::Queued,
                progress: 0.0,
                error: None,
                duration_seconds: None,
            };

            queue.push_back(item);
            added += 1;
        }

        TelemetryEventBuilder::new("batch_processing", "files_added")
            .message("Files added to batch queue")
            .attr("requested", requested)
            .attr("added", added)
            .attr("skipped", skipped)
            .attr("queue_size", queue.len())
            .emit();

        Ok(self.build_status(&queue))
    }

    pub async fn process_queue(&self) -> Result<(), String> {
        if self.is_processing.load(Ordering::SeqCst) {
            TelemetryEventBuilder::new("batch_processing", "process_queue_rejected")
                .level(TelemetryLevel::Warn)
                .message("Batch processing start rejected because a run is already active")
                .emit();
            return Err("Batch processing already in progress".to_string());
        }

        self.is_processing.store(true, Ordering::SeqCst);
        self.cancel_signal.store(false, Ordering::SeqCst);

        let queue = self.queue.clone();
        let cancel = self.cancel_signal.clone();
        let is_processing = self.is_processing.clone();
        let app = self.app_handle.clone();
        let initial_total = {
            let q = self.queue.lock().await;
            q.len()
        };

        TelemetryEventBuilder::new("batch_processing", "process_queue_started")
            .message("Batch processing started")
            .attr("queue_size", initial_total)
            .emit();

        tokio::spawn(async move {
            loop {
                if cancel.load(Ordering::SeqCst) {
                    info!("Batch processing cancelled");
                    TelemetryEventBuilder::new("batch_processing", "process_queue_cancelled")
                        .message("Batch processing cancelled")
                        .emit();
                    break;
                }

                // Find next queued item
                let next_id = {
                    let q = queue.lock().await;
                    q.iter()
                        .find(|item| item.status == JobStatus::Queued)
                        .map(|item| item.id.clone())
                };

                let Some(item_id) = next_id else {
                    info!("Batch processing complete - no more queued items");
                    break;
                };

                // Update status to Decoding
                {
                    let mut q = queue.lock().await;
                    if let Some(item) = q.iter_mut().find(|i| i.id == item_id) {
                        let file_name = item.file_name.clone();
                        item.status = JobStatus::Decoding;
                        item.progress = 0.1;
                        TelemetryEventBuilder::new("batch_processing", "item_decoding_started")
                            .message("Batch item decoding started")
                            .session_id(item_id.clone())
                            .attr("file_name", file_name)
                            .emit();
                    }
                    if let Some(ref app) = app {
                        let status = Self::build_status_static(&q);
                        let _ = app.emit(
                            "batch-item-status",
                            &BatchProgressEvent {
                                id: item_id.clone(),
                                status: JobStatus::Decoding,
                                progress: 0.1,
                                error: None,
                                duration_seconds: None,
                                total_items: status.total,
                                completed_items: status.completed,
                            },
                        );
                    }
                }

                // Get file path
                let file_path = {
                    let q = queue.lock().await;
                    q.iter()
                        .find(|i| i.id == item_id)
                        .map(|i| i.file_path.clone())
                        .unwrap_or_default()
                };

                // Decode audio file
                let path = PathBuf::from(&file_path);
                match decoder::decode_audio_file(&path) {
                    Ok(decoded) => {
                        debug!(
                            "Decoded {}: {:.1}s, {} samples",
                            file_path,
                            decoded.duration_seconds,
                            decoded.samples.len()
                        );

                        // Update with duration and mark as Transcribing
                        {
                            let mut q = queue.lock().await;
                            if let Some(item) = q.iter_mut().find(|i| i.id == item_id) {
                                let file_name = item.file_name.clone();
                                item.duration_seconds = Some(decoded.duration_seconds);
                                item.status = JobStatus::Transcribing;
                                item.progress = 0.5;
                                TelemetryEventBuilder::new("batch_processing", "item_transcribing_started")
                                    .message("Batch item transcription started")
                                    .session_id(item_id.clone())
                                    .attr("file_name", file_name)
                                    .attr("duration_seconds", format!("{:.2}", decoded.duration_seconds))
                                    .emit();
                            }
                            if let Some(ref app) = app {
                                let status = Self::build_status_static(&q);
                                let _ = app.emit(
                                    "batch-item-status",
                                    &BatchProgressEvent {
                                        id: item_id.clone(),
                                        status: JobStatus::Transcribing,
                                        progress: 0.5,
                                        error: None,
                                        duration_seconds: Some(decoded.duration_seconds),
                                        total_items: status.total,
                                        completed_items: status.completed,
                                    },
                                );
                            }
                        }

                        // Transcribe with the same manager used by live transcription.
                        let transcription_text = if let Some(ref app) = app {
                            let transcription_manager = app.state::<Arc<TranscriptionManager>>();
                            match transcription_manager.transcribe(decoded.samples.clone()) {
                                Ok(text) => text,
                                Err(e) => {
                                    error!("Failed to transcribe {}: {}", file_path, e);
                                    let mut q = queue.lock().await;
                                    if let Some(item) = q.iter_mut().find(|i| i.id == item_id) {
                                        let file_name = item.file_name.clone();
                                        item.status = JobStatus::Failed;
                                        item.error = Some(format!("Transcription failed: {}", e));
                                        item.progress = 0.0;
                                        TelemetryEventBuilder::new("batch_processing", "item_failed")
                                            .level(TelemetryLevel::Error)
                                            .message("Batch item transcription failed")
                                            .session_id(item_id.clone())
                                            .attr("file_name", file_name)
                                            .status("transcription_failed")
                                            .attr("error", &e)
                                            .emit();
                                    }
                                    let status = Self::build_status_static(&q);
                                    let _ = app.emit(
                                        "batch-item-status",
                                        &BatchProgressEvent {
                                            id: item_id.clone(),
                                            status: JobStatus::Failed,
                                            progress: 0.0,
                                            error: Some(format!("Transcription failed: {}", e)),
                                            duration_seconds: Some(decoded.duration_seconds),
                                            total_items: status.total,
                                            completed_items: status.completed,
                                        },
                                    );
                                    continue;
                                }
                            }
                        } else {
                            return;
                        };

                        // Persist transcription output in history.
                        if let Some(ref app) = app {
                            let history_manager = app.state::<Arc<HistoryManager>>();
                            if let Err(e) = history_manager
                                .save_transcription(
                                    decoded.samples.clone(),
                                    transcription_text,
                                    None,
                                    None,
                                )
                                .await
                            {
                                warn!("Failed to save batch transcription history: {}", e);
                            }
                        }

                        // Mark as completed after transcription and persistence.
                        {
                            let mut q = queue.lock().await;
                            if let Some(item) = q.iter_mut().find(|i| i.id == item_id) {
                                let file_name = item.file_name.clone();
                                item.status = JobStatus::Completed;
                                item.progress = 1.0;
                                TelemetryEventBuilder::new("batch_processing", "item_completed")
                                    .message("Batch item completed")
                                    .session_id(item_id.clone())
                                    .attr("file_name", file_name)
                                    .attr("duration_seconds", format!("{:.2}", decoded.duration_seconds))
                                    .emit();
                            }
                            if let Some(ref app) = app {
                                let status = Self::build_status_static(&q);
                                let _ = app.emit(
                                    "batch-item-status",
                                    &BatchProgressEvent {
                                        id: item_id.clone(),
                                        status: JobStatus::Completed,
                                        progress: 1.0,
                                        error: None,
                                        duration_seconds: Some(decoded.duration_seconds),
                                        total_items: status.total,
                                        completed_items: status.completed,
                                    },
                                );
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to decode {}: {}", file_path, e);
                        let mut q = queue.lock().await;
                        if let Some(item) = q.iter_mut().find(|i| i.id == item_id) {
                            let file_name = item.file_name.clone();
                            item.status = JobStatus::Failed;
                            item.error = Some(e.clone());
                            item.progress = 0.0;
                            TelemetryEventBuilder::new("batch_processing", "item_failed")
                                .level(TelemetryLevel::Error)
                                .message("Batch item decoding failed")
                                .session_id(item_id.clone())
                                .attr("file_name", file_name)
                                .status("decode_failed")
                                .attr("error", &e)
                                .emit();
                        }
                        if let Some(ref app) = app {
                            let status = Self::build_status_static(&q);
                            let _ = app.emit(
                                "batch-item-status",
                                &BatchProgressEvent {
                                    id: item_id.clone(),
                                    status: JobStatus::Failed,
                                    progress: 0.0,
                                    error: Some(e),
                                    duration_seconds: None,
                                    total_items: status.total,
                                    completed_items: status.completed,
                                },
                            );
                        }
                    }
                }
            }

            is_processing.store(false, Ordering::SeqCst);

            // Emit completion event
            if let Some(ref app) = app {
                let q = queue.lock().await;
                let status = Self::build_status_static(&q);
                let _ = app.emit("batch-complete", &status);
                TelemetryEventBuilder::new("batch_processing", "process_queue_completed")
                    .message("Batch processing completed")
                    .attr("total", status.total)
                    .attr("completed", status.completed)
                    .attr("failed", status.failed)
                    .emit();
            }
        });

        Ok(())
    }

    pub fn cancel(&self) {
        self.cancel_signal.store(true, Ordering::SeqCst);
        TelemetryEventBuilder::new("batch_processing", "cancel_requested")
            .message("Batch processing cancellation requested")
            .emit();
    }

    pub async fn get_status(&self) -> BatchQueueStatus {
        let queue = self.queue.lock().await;
        self.build_status(&queue)
    }

    pub async fn remove_item(&self, id: &str) -> Result<(), String> {
        let mut queue = self.queue.lock().await;
        if let Some(pos) = queue.iter().position(|item| item.id == id) {
            let file_name = queue.get(pos).map(|item| item.file_name.clone());
            queue.remove(pos);
            TelemetryEventBuilder::new("batch_processing", "item_removed")
                .message("Batch item removed from queue")
                .session_id(id.to_string())
                .attr("file_name", file_name.unwrap_or_default())
                .emit();
            Ok(())
        } else {
            Err("Item not found".to_string())
        }
    }

    pub async fn clear_completed(&self) {
        let mut queue = self.queue.lock().await;
        let before = queue.len();
        queue
            .retain(|item| item.status != JobStatus::Completed && item.status != JobStatus::Failed);
        let removed = before.saturating_sub(queue.len());
        TelemetryEventBuilder::new("batch_processing", "completed_items_cleared")
            .message("Cleared completed batch items")
            .attr("removed", removed)
            .attr("remaining", queue.len())
            .emit();
    }

    fn build_status(&self, queue: &VecDeque<BatchItem>) -> BatchQueueStatus {
        Self::build_status_static(queue)
    }

    fn build_status_static(queue: &VecDeque<BatchItem>) -> BatchQueueStatus {
        let items: Vec<BatchItem> = queue.iter().cloned().collect();
        let total = items.len();
        let completed = items
            .iter()
            .filter(|i| i.status == JobStatus::Completed)
            .count();
        let failed = items
            .iter()
            .filter(|i| i.status == JobStatus::Failed)
            .count();
        let is_processing = items
            .iter()
            .any(|i| i.status == JobStatus::Decoding || i.status == JobStatus::Transcribing);

        BatchQueueStatus {
            items,
            total,
            completed,
            failed,
            is_processing,
        }
    }
}
