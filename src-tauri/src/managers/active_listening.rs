//! Active Listening Manager
//!
//! Manages continuous background transcription with Ollama-powered AI insights.
//! Handles the state machine for active listening sessions and coordinates
//! between audio input, transcription, and insight generation.

use crate::audio_toolkit::diarization::{create_shared_diarizer, SharedDiarizer};
use crate::managers::history::HistoryManager;
use crate::managers::rag::{DocMetadata, RagManager};
use crate::managers::suggestion_engine::{Suggestion, SuggestionContext, SuggestionEngine};
use crate::managers::transcription::TranscriptionManager;
use crate::ollama_client::{apply_prompt_template, OllamaClient};
use crate::settings::get_settings;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

/// State of the active listening session
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ActiveListeningState {
    /// No active session
    Idle,
    /// Listening and accumulating audio
    Listening,
    /// Processing a segment (transcribing + Ollama)
    Processing,
    /// Error state
    Error,
}

impl Default for ActiveListeningState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Information about an active listening session
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct ActiveListeningSession {
    /// Unique session identifier
    pub id: String,
    /// Unix timestamp when session started (milliseconds)
    pub started_at: i64,
    /// Unix timestamp when session ended (milliseconds)
    pub ended_at: Option<i64>,
    /// User-defined topic for this session
    pub topic: Option<String>,
    /// All insights generated during this session
    pub insights: Vec<SessionInsight>,
}

/// A single insight generated from a segment
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct SessionInsight {
    /// Unix timestamp when this insight was generated
    pub timestamp: i64,
    /// The transcribed text for this segment
    pub transcription: String,
    /// The AI-generated insight
    pub insight: String,
    /// Duration of the audio segment in milliseconds
    pub duration_ms: u64,
    /// Speaker ID (0 = primary/you, 1+ = others)
    pub speaker_id: Option<u32>,
    /// Human-readable speaker label (e.g., "You", "Speaker 2", or custom name)
    pub speaker_label: Option<String>,
}

/// An action item extracted from a meeting
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct ActionItem {
    /// Description of the action
    pub description: String,
    /// Person responsible (if mentioned)
    pub assignee: Option<String>,
    /// Deadline (if mentioned)
    pub deadline: Option<String>,
}

/// Comprehensive meeting summary generated from a session
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct MeetingSummary {
    /// Session ID this summary is for
    pub session_id: String,
    /// Brief executive summary (2-3 sentences)
    pub executive_summary: String,
    /// Key decisions made during the meeting
    pub decisions: Vec<String>,
    /// Action items with optional assignees and deadlines
    pub action_items: Vec<ActionItem>,
    /// Main topics discussed
    pub topics: Vec<String>,
    /// Suggested follow-up questions
    pub follow_ups: Vec<String>,
    /// Total duration in minutes
    pub duration_minutes: u32,
    /// When this summary was generated
    pub generated_at: i64,
}

/// Event payload for active listening segment
#[derive(Clone, Debug, Serialize, Type)]
pub struct ActiveListeningSegmentEvent {
    pub session_id: String,
    pub transcription: String,
    pub timestamp: i64,
    /// Speaker ID (0 = primary/you, 1+ = others)
    pub speaker_id: Option<u32>,
    /// Human-readable speaker label
    pub speaker_label: Option<String>,
}

/// Event payload for active listening insight (streaming)
#[derive(Clone, Debug, Serialize, Type)]
pub struct ActiveListeningInsightEvent {
    pub session_id: String,
    pub chunk: String,
    pub done: bool,
}

/// Event payload for session state changes
#[derive(Clone, Debug, Serialize, Type)]
pub struct ActiveListeningStateEvent {
    pub state: ActiveListeningState,
    pub session_id: Option<String>,
    pub error: Option<String>,
}

/// Active Listening Manager
///
/// Coordinates the active listening feature, managing:
/// - Session lifecycle (start/stop)
/// - Audio segment buffering
/// - Transcription via TranscriptionManager
/// - Insight generation via Ollama
/// - Context management across segments
/// - Speaker diarization (who is speaking)
pub struct ActiveListeningManager {
    app_handle: AppHandle,
    transcription_manager: Arc<TranscriptionManager>,

    /// Current state
    state: Arc<Mutex<ActiveListeningState>>,

    /// Current session
    current_session: Arc<Mutex<Option<ActiveListeningSession>>>,

    /// Audio sample buffer for current segment
    segment_buffer: Arc<Mutex<Vec<f32>>>,

    /// When the current segment started accumulating
    segment_start_time: Arc<Mutex<Option<Instant>>>,

    /// Rolling context of previous insights for continuity
    context_buffer: Arc<Mutex<VecDeque<String>>>,

    /// Shutdown signal for background tasks
    shutdown_signal: Arc<AtomicBool>,

    /// Speaker diarizer for tracking who is speaking
    diarizer: SharedDiarizer,

    /// Current detected speaker ID for the segment being accumulated
    current_segment_speaker: Arc<Mutex<Option<u32>>>,
}

impl ActiveListeningManager {
    /// Create a new ActiveListeningManager
    pub fn new(
        app_handle: &AppHandle,
        transcription_manager: Arc<TranscriptionManager>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            app_handle: app_handle.clone(),
            transcription_manager,
            state: Arc::new(Mutex::new(ActiveListeningState::Idle)),
            current_session: Arc::new(Mutex::new(None)),
            segment_buffer: Arc::new(Mutex::new(Vec::new())),
            segment_start_time: Arc::new(Mutex::new(None)),
            context_buffer: Arc::new(Mutex::new(VecDeque::new())),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
            diarizer: create_shared_diarizer(),
            current_segment_speaker: Arc::new(Mutex::new(None)),
        })
    }

    /// Get the current state
    pub fn get_state(&self) -> ActiveListeningState {
        self.state.lock().unwrap().clone()
    }

    /// Check if a session is currently active (not idle)
    pub fn is_session_active(&self) -> bool {
        *self.state.lock().unwrap() != ActiveListeningState::Idle
    }

    /// Get the current session info
    pub fn get_current_session(&self) -> Option<ActiveListeningSession> {
        self.current_session.lock().unwrap().clone()
    }

    /// Start a new active listening session
    pub fn start_session(&self, topic: Option<String>) -> Result<String, String> {
        let mut state = self.state.lock().unwrap();

        if *state != ActiveListeningState::Idle {
            return Err("A session is already active".to_string());
        }

        // Generate session ID
        let session_id = format!("al_{}", chrono::Utc::now().timestamp_millis());
        let started_at = chrono::Utc::now().timestamp_millis();

        // Create new session
        let session = ActiveListeningSession {
            id: session_id.clone(),
            started_at,
            ended_at: None,
            topic: topic.clone(),
            insights: Vec::new(),
        };

        // Update state
        *state = ActiveListeningState::Listening;
        drop(state);

        // Store session
        {
            let mut current = self.current_session.lock().unwrap();
            *current = Some(session);
        }

        // Clear buffers
        {
            let mut buffer = self.segment_buffer.lock().unwrap();
            buffer.clear();
        }
        {
            let mut start_time = self.segment_start_time.lock().unwrap();
            *start_time = None;
        }
        {
            let mut context = self.context_buffer.lock().unwrap();
            context.clear();
        }

        // Reset diarizer for new session
        {
            let mut diarizer = self.diarizer.lock().unwrap();
            diarizer.reset();
        }
        {
            let mut speaker = self.current_segment_speaker.lock().unwrap();
            *speaker = None;
        }

        // Emit session started event
        let _ = self.app_handle.emit(
            "active-listening-state-changed",
            ActiveListeningStateEvent {
                state: ActiveListeningState::Listening,
                session_id: Some(session_id.clone()),
                error: None,
            },
        );

        info!(
            "Started active listening session: {} with topic: {:?}",
            session_id, topic
        );

        Ok(session_id)
    }

    /// Stop the current active listening session
    pub fn stop_session(&self) -> Result<Option<ActiveListeningSession>, String> {
        let mut state = self.state.lock().unwrap();

        if *state == ActiveListeningState::Idle {
            return Ok(None);
        }

        // Update state
        *state = ActiveListeningState::Idle;
        drop(state);

        // Get and finalize session
        let session = {
            let mut current = self.current_session.lock().unwrap();
            if let Some(mut session) = current.take() {
                session.ended_at = Some(chrono::Utc::now().timestamp_millis());
                Some(session)
            } else {
                None
            }
        };

        // Clear buffers
        {
            let mut buffer = self.segment_buffer.lock().unwrap();
            buffer.clear();
        }
        {
            let mut start_time = self.segment_start_time.lock().unwrap();
            *start_time = None;
        }

        // Emit session ended event
        let _ = self.app_handle.emit(
            "active-listening-state-changed",
            ActiveListeningStateEvent {
                state: ActiveListeningState::Idle,
                session_id: session.as_ref().map(|s| s.id.clone()),
                error: None,
            },
        );

        if let Some(ref s) = session {
            info!(
                "Stopped active listening session: {} with {} insights",
                s.id,
                s.insights.len()
            );
        }

        Ok(session)
    }

    /// Push audio samples to the segment buffer
    ///
    /// This is called by the audio pipeline when in active listening mode.
    /// Samples are accumulated until the segment duration is reached.
    /// Also runs diarization to track speaker changes.
    pub fn push_audio_samples(&self, samples: &[f32]) {
        let state = self.get_state();
        if state != ActiveListeningState::Listening {
            debug!(
                "Skipping audio samples push - state is {:?}, not Listening",
                state
            );
            return;
        }
        debug!("Pushing {} audio samples to segment buffer", samples.len());

        let settings = get_settings(&self.app_handle);
        let segment_duration_ms =
            (settings.active_listening.segment_duration_seconds as u64) * 1000;

        // Update segment start time if this is the first push
        {
            let mut start_time = self.segment_start_time.lock().unwrap();
            if start_time.is_none() {
                *start_time = Some(Instant::now());
            }
        }

        // Run diarization on the audio samples
        {
            let mut diarizer = self.diarizer.lock().unwrap();
            // Process samples in 30ms frames (480 samples at 16kHz)
            const FRAME_SIZE: usize = 480;
            for chunk in samples.chunks(FRAME_SIZE) {
                if let Some(change) = diarizer.process_frame(chunk) {
                    debug!(
                        "Speaker change detected: {} -> {}",
                        change.previous_speaker, change.new_speaker
                    );
                }
            }
            // Update current segment speaker
            let current_speaker = diarizer.get_current_speaker();
            let mut segment_speaker = self.current_segment_speaker.lock().unwrap();
            // Use the first detected speaker for the segment, or update if none set
            if segment_speaker.is_none() {
                *segment_speaker = Some(current_speaker);
            }
        }

        // Add samples to buffer
        {
            let mut buffer = self.segment_buffer.lock().unwrap();
            buffer.extend_from_slice(samples);
        }

        // Check if we should process the segment
        let should_process = {
            let start_time = self.segment_start_time.lock().unwrap();
            if let Some(start) = *start_time {
                start.elapsed() >= Duration::from_millis(segment_duration_ms)
            } else {
                false
            }
        };

        if should_process {
            self.trigger_segment_processing();
        }
    }

    /// Trigger processing of the current segment
    fn trigger_segment_processing(&self) {
        // Get samples and clear buffer
        let samples = {
            let mut buffer = self.segment_buffer.lock().unwrap();
            let samples = buffer.clone();
            buffer.clear();
            samples
        };

        // Reset segment start time
        {
            let mut start_time = self.segment_start_time.lock().unwrap();
            *start_time = Some(Instant::now());
        }

        if samples.is_empty() {
            debug!("Empty segment buffer, skipping processing");
            return;
        }

        // Get speaker info for this segment
        let speaker_id = {
            let mut segment_speaker = self.current_segment_speaker.lock().unwrap();
            let id = *segment_speaker;
            *segment_speaker = None; // Reset for next segment
            id
        };

        // Capture session info BEFORE spawning async task (so stop_session doesn't clear it first)
        let (session_id, topic) = {
            let session = self.current_session.lock().unwrap();
            match &*session {
                Some(s) => (s.id.clone(), s.topic.clone()),
                None => {
                    warn!("No active session when triggering segment processing");
                    return;
                }
            }
        };

        // Update state
        {
            let mut state = self.state.lock().unwrap();
            *state = ActiveListeningState::Processing;
        }

        // Emit state change
        let _ = self.app_handle.emit(
            "active-listening-state-changed",
            ActiveListeningStateEvent {
                state: ActiveListeningState::Processing,
                session_id: Some(session_id.clone()),
                error: None,
            },
        );

        // Process in background with captured session info
        let self_clone = ActiveListeningManagerHandle {
            app_handle: self.app_handle.clone(),
            transcription_manager: self.transcription_manager.clone(),
            state: self.state.clone(),
            current_session: self.current_session.clone(),
            context_buffer: self.context_buffer.clone(),
            shutdown_signal: self.shutdown_signal.clone(),
        };

        let segment_start_instant = Instant::now();
        info!(
            "Spawning segment processing for session {} with {} samples, speaker: {:?}",
            session_id,
            samples.len(),
            speaker_id
        );
        tauri::async_runtime::spawn(async move {
            self_clone
                .process_segment_with_session(
                    samples,
                    segment_start_instant,
                    session_id,
                    topic,
                    speaker_id,
                )
                .await;
        });
    }

    /// Force process any remaining audio in the buffer
    pub fn flush_segment(&self) {
        let state = self.get_state();
        info!(
            "flush_segment called - current state: {:?}",
            state
        );
        if state != ActiveListeningState::Listening {
            info!("flush_segment: skipping - not in Listening state");
            return;
        }

        let buffer_len = {
            let buffer = self.segment_buffer.lock().unwrap();
            buffer.len()
        };

        info!(
            "flush_segment: buffer has {} samples (need >= 8000)",
            buffer_len
        );

        // Only process if we have meaningful audio (at least 0.5 seconds at 16kHz)
        if buffer_len >= 8000 {
            info!("flush_segment: triggering segment processing");
            self.trigger_segment_processing();
        } else {
            info!("flush_segment: not enough audio samples, skipping");
        }
    }

    /// Generate a comprehensive meeting summary from the session
    ///
    /// This method creates a structured summary including:
    /// - Executive summary (2-3 sentences)
    /// - Key decisions made
    /// - Action items with assignees and deadlines
    /// - Topics discussed
    /// - Suggested follow-up questions
    pub async fn generate_session_summary(
        &self,
        session: &ActiveListeningSession,
    ) -> Result<MeetingSummary, String> {
        let settings = get_settings(&self.app_handle);
        let ollama_settings = &settings.active_listening;

        if ollama_settings.ollama_model.is_empty() {
            return Err("No Ollama model configured".to_string());
        }

        if session.insights.is_empty() {
            return Err("No insights to summarize".to_string());
        }

        // Combine all transcriptions
        let full_transcript = session
            .insights
            .iter()
            .map(|i| i.transcription.as_str())
            .collect::<Vec<_>>()
            .join("\n\n");

        // Calculate duration
        let duration_minutes = if let Some(ended) = session.ended_at {
            ((ended - session.started_at) / 60000) as u32
        } else {
            let now = chrono::Utc::now().timestamp_millis();
            ((now - session.started_at) / 60000) as u32
        };

        let topic = session.topic.clone().unwrap_or_else(|| "Meeting".to_string());

        let prompt = format!(
            r#"Analyze this meeting transcript and provide a structured summary.

Meeting Topic: {topic}
Duration: {duration_minutes} minutes
Transcript:
{full_transcript}

Provide a comprehensive summary in the following JSON format:
{{
  "executive_summary": "2-3 sentence overview of the meeting",
  "decisions": ["decision 1", "decision 2"],
  "action_items": [
    {{"description": "task description", "assignee": "person name or null", "deadline": "deadline or null"}}
  ],
  "topics": ["topic 1", "topic 2"],
  "follow_ups": ["suggested follow-up question 1", "question 2"]
}}

Important:
- Be concise and factual
- Only include items that were actually discussed
- Use null for unknown assignees/deadlines
- Return valid JSON only"#,
        );

        info!("Generating meeting summary for session {}", session.id);

        let client = OllamaClient::new(&ollama_settings.ollama_base_url)
            .map_err(|e| format!("Failed to create Ollama client: {}", e))?;

        let response = client
            .generate(&ollama_settings.ollama_model, prompt)
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        // Parse the JSON response
        let summary = Self::parse_summary_response(&response, session, duration_minutes)?;

        info!(
            "Generated summary with {} decisions, {} action items, {} topics",
            summary.decisions.len(),
            summary.action_items.len(),
            summary.topics.len()
        );

        Ok(summary)
    }

    /// Parse the JSON response from Ollama into a MeetingSummary struct
    fn parse_summary_response(
        response: &str,
        session: &ActiveListeningSession,
        duration_minutes: u32,
    ) -> Result<MeetingSummary, String> {
        // Try to extract JSON from the response (it may be wrapped in markdown)
        let json_str = if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                response
            }
        } else {
            response
        };

        // Parse the JSON
        let parsed: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse summary JSON: {}. Response: {}", e, response))?;

        let executive_summary = parsed
            .get("executive_summary")
            .and_then(|v| v.as_str())
            .unwrap_or("Summary not available")
            .to_string();

        let decisions = parsed
            .get("decisions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let action_items = parsed
            .get("action_items")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| {
                        let desc = item.get("description")?.as_str()?.to_string();
                        Some(ActionItem {
                            description: desc,
                            assignee: item
                                .get("assignee")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            deadline: item
                                .get("deadline")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let topics = parsed
            .get("topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let follow_ups = parsed
            .get("follow_ups")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(MeetingSummary {
            session_id: session.id.clone(),
            executive_summary,
            decisions,
            action_items,
            topics,
            follow_ups,
            duration_minutes,
            generated_at: chrono::Utc::now().timestamp_millis(),
        })
    }
}

/// Handle for async operations
struct ActiveListeningManagerHandle {
    app_handle: AppHandle,
    transcription_manager: Arc<TranscriptionManager>,
    state: Arc<Mutex<ActiveListeningState>>,
    current_session: Arc<Mutex<Option<ActiveListeningSession>>>,
    context_buffer: Arc<Mutex<VecDeque<String>>>,
    /// Shutdown signal for graceful cancellation of long-running Ollama requests.
    /// Currently set on Drop but can be extended to support request cancellation.
    #[allow(dead_code)]
    shutdown_signal: Arc<AtomicBool>,
}

impl ActiveListeningManagerHandle {
    /// Process a segment with pre-captured session info.
    /// This version is used by trigger_segment_processing to ensure session info
    /// is captured before the async task starts, preventing race conditions with stop_session.
    async fn process_segment_with_session(
        &self,
        samples: Vec<f32>,
        segment_start: Instant,
        session_id: String,
        topic: Option<String>,
        speaker_id: Option<u32>,
    ) {
        let segment_duration_ms = segment_start.elapsed().as_millis() as u64;
        let speaker_label = speaker_id.map(|id| {
            if id == 0 {
                "You".to_string()
            } else {
                format!("Speaker {}", id + 1)
            }
        });
        info!(
            "process_segment_with_session: session={}, {} samples, duration {}ms, speaker={:?}",
            session_id,
            samples.len(),
            segment_duration_ms,
            speaker_label
        );

        // Keep a copy of samples for saving to history
        let samples_for_history = samples.clone();

        // Step 1: Transcribe the segment
        info!("Transcribing segment with {} samples", samples.len());
        let transcription = match self.transcription_manager.transcribe(samples) {
            Ok(text) => text,
            Err(e) => {
                error!("Transcription failed: {}", e);
                self.emit_error(&session_id, format!("Transcription failed: {}", e));
                self.transition_to_listening();
                return;
            }
        };

        info!("Transcription result: '{}'", transcription.trim());

        if transcription.trim().is_empty() {
            info!("Empty transcription, skipping Ollama");
            self.transition_to_listening();
            return;
        }

        let timestamp = chrono::Utc::now().timestamp_millis();

        // Emit segment transcription event with speaker info
        let _ = self.app_handle.emit(
            "active-listening-segment",
            ActiveListeningSegmentEvent {
                session_id: session_id.clone(),
                transcription: transcription.clone(),
                timestamp,
                speaker_id,
                speaker_label: speaker_label.clone(),
            },
        );

        // Step 2: Generate real-time suggestions (runs in parallel with insights)
        let settings = get_settings(&self.app_handle);
        if settings.suggestions.enabled {
            self.generate_suggestions(
                session_id.clone(),
                transcription.clone(),
                topic.clone(),
            )
            .await;
        }

        // Step 3: Generate insight with Ollama
        let settings = get_settings(&self.app_handle);
        let ollama_settings = &settings.active_listening;

        if ollama_settings.ollama_model.is_empty() {
            warn!("No Ollama model configured, skipping insight generation");
            self.add_insight_to_session(
                &session_id,
                transcription.clone(),
                String::new(),
                segment_duration_ms,
                speaker_id,
                speaker_label.clone(),
            );
            // Save to history without LLM insight
            self.save_to_history(samples_for_history, transcription, None, None)
                .await;
            self.transition_to_listening();
            return;
        }

        info!(
            "Calling Ollama with model: {} at base URL: {}",
            ollama_settings.ollama_model, ollama_settings.ollama_base_url
        );

        // Get the selected prompt
        let prompt_template = ollama_settings
            .get_selected_prompt()
            .map(|p| p.prompt_template.clone())
            .unwrap_or_else(|| "Summarize: {{transcription}}".to_string());

        // Get context from previous insights
        let previous_context = {
            let context = self.context_buffer.lock().unwrap();
            if context.is_empty() {
                "No previous context.".to_string()
            } else {
                context
                    .iter()
                    .enumerate()
                    .map(|(i, c)| format!("{}. {}", i + 1, c))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        // Apply template
        let prompt = apply_prompt_template(
            &prompt_template,
            &transcription,
            &previous_context,
            topic.as_deref(),
        );

        info!("Ollama prompt: {}", prompt);

        // Call Ollama with streaming
        let client = match OllamaClient::new(&ollama_settings.ollama_base_url) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create Ollama client: {}", e);
                self.transition_to_listening();
                return;
            }
        };
        let (tx, mut rx) = mpsc::channel::<String>(100);

        let session_id_clone = session_id.clone();
        let app_handle_clone = self.app_handle.clone();

        // Spawn task to forward stream chunks to frontend
        let stream_forward_handle = tauri::async_runtime::spawn(async move {
            let mut full_response = String::new();
            while let Some(chunk) = rx.recv().await {
                full_response.push_str(&chunk);
                let _ = app_handle_clone.emit(
                    "active-listening-insight",
                    ActiveListeningInsightEvent {
                        session_id: session_id_clone.clone(),
                        chunk,
                        done: false,
                    },
                );
            }
            full_response
        });

        // Call Ollama
        let ollama_result = client
            .generate_stream(&ollama_settings.ollama_model, prompt, tx)
            .await;

        // Wait for stream forwarding to complete
        let insight = match stream_forward_handle.await {
            Ok(text) => text,
            Err(e) => {
                error!("Stream forward task failed: {}", e);
                String::new()
            }
        };

        // Handle Ollama result
        info!(
            "Ollama stream completed, insight length: {} chars",
            insight.len()
        );
        match ollama_result {
            Ok(_) => {
                // Emit done signal
                let _ = self.app_handle.emit(
                    "active-listening-insight",
                    ActiveListeningInsightEvent {
                        session_id: session_id.clone(),
                        chunk: String::new(),
                        done: true,
                    },
                );

                // Add to context buffer
                if !insight.is_empty() {
                    let mut context = self.context_buffer.lock().unwrap();
                    context.push_back(insight.clone());

                    // Keep only the configured number of context entries
                    let max_context = ollama_settings.context_window_size;
                    while context.len() > max_context {
                        context.pop_front();
                    }
                }

                // Add insight to session (session might be stopped, but that's okay)
                self.add_insight_to_session(
                    &session_id,
                    transcription.clone(),
                    insight.clone(),
                    segment_duration_ms,
                    speaker_id,
                    speaker_label.clone(),
                );

                // Save to history with LLM insight as post-processed text
                let post_processed = if insight.is_empty() {
                    None
                } else {
                    Some(insight)
                };
                info!("Saving to history with insight: {:?}", post_processed);
                self.save_to_history(
                    samples_for_history,
                    transcription,
                    post_processed,
                    Some(prompt_template),
                )
                .await;
            }
            Err(e) => {
                error!("Ollama generation failed: {}", e);
                // Still save the transcription without insight
                self.add_insight_to_session(
                    &session_id,
                    transcription.clone(),
                    String::new(),
                    segment_duration_ms,
                    speaker_id,
                    speaker_label,
                );
                // Save to history without LLM insight
                self.save_to_history(samples_for_history, transcription, None, None)
                    .await;
            }
        }

        self.transition_to_listening();
    }

    /// Save transcription and audio to history
    async fn save_to_history(
        &self,
        audio_samples: Vec<f32>,
        transcription: String,
        post_processed_text: Option<String>,
        post_process_prompt: Option<String>,
    ) {
        let history_manager = self.app_handle.state::<Arc<HistoryManager>>();
        if let Err(e) = history_manager
            .save_transcription(
                audio_samples,
                transcription,
                post_processed_text,
                post_process_prompt,
            )
            .await
        {
            error!("Failed to save Active Listening segment to history: {}", e);
        } else {
            debug!("Saved Active Listening segment to history");
        }
    }

    fn transition_to_listening(&self) {
        let session_id = {
            let session = self.current_session.lock().unwrap();
            session.as_ref().map(|s| s.id.clone())
        };

        // Only transition if we're still processing (not stopped)
        let mut state = self.state.lock().unwrap();
        if *state == ActiveListeningState::Processing {
            *state = ActiveListeningState::Listening;
            drop(state);

            let _ = self.app_handle.emit(
                "active-listening-state-changed",
                ActiveListeningStateEvent {
                    state: ActiveListeningState::Listening,
                    session_id,
                    error: None,
                },
            );
        }
    }

    fn emit_error(&self, session_id: &str, error: String) {
        let _ = self.app_handle.emit(
            "active-listening-state-changed",
            ActiveListeningStateEvent {
                state: ActiveListeningState::Error,
                session_id: Some(session_id.to_string()),
                error: Some(error),
            },
        );
    }

    /// Generate and emit suggestions based on the transcribed segment
    async fn generate_suggestions(
        &self,
        session_id: String,
        transcription: String,
        topic: Option<String>,
    ) {
        // Try to get the SuggestionEngine from app state
        if let Some(engine) = self.app_handle.try_state::<SuggestionEngine>() {
            // Build context from previous insights
            let previous_context = {
                let context = self.context_buffer.lock().unwrap();
                if context.is_empty() {
                    String::new()
                } else {
                    context
                        .iter()
                        .take(3) // Last 3 context entries for suggestions
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            };

            let context = SuggestionContext {
                transcription,
                previous_context,
                session_topic: topic,
                session_id: session_id.clone(),
            };

            // Generate suggestions
            let suggestions = engine.get_suggestions(&context).await;

            // Emit suggestions to frontend if any were generated
            if !suggestions.is_empty() {
                info!("Generated {} suggestions for session {}", suggestions.len(), session_id);
                engine.emit_suggestions(&session_id, suggestions).await;
            }
        } else {
            debug!("SuggestionEngine not available in app state");
        }
    }

    fn add_insight_to_session(
        &self,
        session_id: &str,
        transcription: String,
        insight: String,
        duration_ms: u64,
        speaker_id: Option<u32>,
        speaker_label: Option<String>,
    ) {
        // Store transcription for later indexing
        let transcription_for_rag = transcription.clone();
        let session_id_for_rag = session_id.to_string();
        let app_handle = self.app_handle.clone();

        let mut session_guard = self.current_session.lock().unwrap();
        if let Some(ref mut session) = *session_guard {
            if session.id == session_id {
                session.insights.push(SessionInsight {
                    timestamp: chrono::Utc::now().timestamp_millis(),
                    transcription,
                    insight,
                    duration_ms,
                    speaker_id,
                    speaker_label,
                });
            }
        }
        drop(session_guard);

        // Index transcription in knowledge base if enabled
        // Do this asynchronously to not block the main flow
        tokio::spawn(async move {
            Self::maybe_index_transcription(&app_handle, &transcription_for_rag, &session_id_for_rag)
                .await;
        });
    }

    /// Index a transcription in the knowledge base if auto-indexing is enabled
    async fn maybe_index_transcription(
        app_handle: &AppHandle,
        transcription: &str,
        session_id: &str,
    ) {
        // Skip empty transcriptions
        if transcription.trim().is_empty() {
            return;
        }

        // Check if knowledge base and auto-indexing are enabled
        let settings = get_settings(app_handle);
        if !settings.knowledge_base.enabled || !settings.knowledge_base.auto_index_transcriptions {
            return;
        }

        // Get the RAG manager from app state
        let rag_manager = match app_handle.try_state::<Arc<RagManager>>() {
            Some(manager) => manager,
            None => {
                debug!("RAG manager not available, skipping transcription indexing");
                return;
            }
        };

        // Index the transcription
        let metadata = DocMetadata {
            source_type: "transcription".to_string(),
            source_id: Some(session_id.to_string()),
            title: Some(format!(
                "Active Listening - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M")
            )),
            extra: None,
        };

        match rag_manager.add_document(transcription, metadata).await {
            Ok(doc_id) => {
                debug!(
                    "Indexed transcription as document {} for session {}",
                    doc_id, session_id
                );
            }
            Err(e) => {
                warn!("Failed to index transcription in knowledge base: {}", e);
            }
        }
    }
}

impl Drop for ActiveListeningManager {
    fn drop(&mut self) {
        debug!("Shutting down ActiveListeningManager");
        self.shutdown_signal.store(true, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_idle() {
        let state = ActiveListeningState::default();
        assert_eq!(state, ActiveListeningState::Idle);
    }

    #[test]
    fn test_session_insight_fields() {
        let insight = SessionInsight {
            timestamp: 1234567890,
            transcription: "Hello world".to_string(),
            insight: "A greeting".to_string(),
            duration_ms: 5000,
            speaker_id: Some(0),
            speaker_label: Some("You".to_string()),
        };

        assert_eq!(insight.timestamp, 1234567890);
        assert_eq!(insight.transcription, "Hello world");
        assert_eq!(insight.insight, "A greeting");
        assert_eq!(insight.duration_ms, 5000);
        assert_eq!(insight.speaker_id, Some(0));
        assert_eq!(insight.speaker_label, Some("You".to_string()));
    }

    #[test]
    fn test_active_listening_session_fields() {
        let session = ActiveListeningSession {
            id: "test_session_123".to_string(),
            started_at: 1000000,
            ended_at: Some(2000000),
            topic: Some("Test Topic".to_string()),
            insights: vec![],
        };

        assert_eq!(session.id, "test_session_123");
        assert_eq!(session.started_at, 1000000);
        assert_eq!(session.ended_at, Some(2000000));
        assert_eq!(session.topic, Some("Test Topic".to_string()));
        assert!(session.insights.is_empty());
    }

    #[test]
    fn test_active_listening_segment_event_fields() {
        let event = ActiveListeningSegmentEvent {
            session_id: "session_1".to_string(),
            transcription: "Test transcription".to_string(),
            timestamp: 123456789,
            speaker_id: Some(1),
            speaker_label: Some("Speaker 2".to_string()),
        };

        assert_eq!(event.session_id, "session_1");
        assert_eq!(event.transcription, "Test transcription");
        assert_eq!(event.timestamp, 123456789);
        assert_eq!(event.speaker_id, Some(1));
        assert_eq!(event.speaker_label, Some("Speaker 2".to_string()));
    }

    #[test]
    fn test_active_listening_insight_event_fields() {
        let event = ActiveListeningInsightEvent {
            session_id: "session_1".to_string(),
            chunk: "insight chunk".to_string(),
            done: false,
        };

        assert_eq!(event.session_id, "session_1");
        assert_eq!(event.chunk, "insight chunk");
        assert!(!event.done);
    }

    #[test]
    fn test_active_listening_insight_event_done() {
        let event = ActiveListeningInsightEvent {
            session_id: "session_1".to_string(),
            chunk: String::new(),
            done: true,
        };

        assert!(event.done);
        assert!(event.chunk.is_empty());
    }

    #[test]
    fn test_active_listening_state_event_idle() {
        let event = ActiveListeningStateEvent {
            state: ActiveListeningState::Idle,
            session_id: None,
            error: None,
        };

        assert_eq!(event.state, ActiveListeningState::Idle);
        assert!(event.session_id.is_none());
        assert!(event.error.is_none());
    }

    #[test]
    fn test_active_listening_state_event_listening() {
        let event = ActiveListeningStateEvent {
            state: ActiveListeningState::Listening,
            session_id: Some("session_123".to_string()),
            error: None,
        };

        assert_eq!(event.state, ActiveListeningState::Listening);
        assert_eq!(event.session_id, Some("session_123".to_string()));
    }

    #[test]
    fn test_active_listening_state_event_error() {
        let event = ActiveListeningStateEvent {
            state: ActiveListeningState::Error,
            session_id: Some("session_123".to_string()),
            error: Some("Something went wrong".to_string()),
        };

        assert_eq!(event.state, ActiveListeningState::Error);
        assert_eq!(event.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_state_equality() {
        assert_eq!(ActiveListeningState::Idle, ActiveListeningState::Idle);
        assert_eq!(ActiveListeningState::Listening, ActiveListeningState::Listening);
        assert_eq!(ActiveListeningState::Processing, ActiveListeningState::Processing);
        assert_eq!(ActiveListeningState::Error, ActiveListeningState::Error);

        assert_ne!(ActiveListeningState::Idle, ActiveListeningState::Listening);
        assert_ne!(ActiveListeningState::Processing, ActiveListeningState::Error);
    }

    #[test]
    fn test_session_with_insights() {
        let insights = vec![
            SessionInsight {
                timestamp: 1000,
                transcription: "First segment".to_string(),
                insight: "First insight".to_string(),
                duration_ms: 5000,
                speaker_id: Some(0),
                speaker_label: Some("You".to_string()),
            },
            SessionInsight {
                timestamp: 2000,
                transcription: "Second segment".to_string(),
                insight: "Second insight".to_string(),
                duration_ms: 3000,
                speaker_id: Some(1),
                speaker_label: Some("Speaker 2".to_string()),
            },
        ];

        let session = ActiveListeningSession {
            id: "test".to_string(),
            started_at: 0,
            ended_at: None,
            topic: Some("Test Topic".to_string()),
            insights,
        };

        assert_eq!(session.insights.len(), 2);
        assert_eq!(session.insights[0].transcription, "First segment");
        assert_eq!(session.insights[0].speaker_id, Some(0));
        assert_eq!(session.insights[1].transcription, "Second segment");
        assert_eq!(session.insights[1].speaker_id, Some(1));
    }

    #[test]
    fn test_session_clone() {
        let session = ActiveListeningSession {
            id: "clone_test".to_string(),
            started_at: 100,
            ended_at: Some(200),
            topic: Some("Topic".to_string()),
            insights: vec![SessionInsight {
                timestamp: 150,
                transcription: "test".to_string(),
                insight: "insight".to_string(),
                duration_ms: 1000,
                speaker_id: None,
                speaker_label: None,
            }],
        };

        let cloned = session.clone();

        assert_eq!(session.id, cloned.id);
        assert_eq!(session.started_at, cloned.started_at);
        assert_eq!(session.ended_at, cloned.ended_at);
        assert_eq!(session.topic, cloned.topic);
        assert_eq!(session.insights.len(), cloned.insights.len());
    }
}
