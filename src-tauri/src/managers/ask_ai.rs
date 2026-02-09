//! Ask AI Manager
//!
//! Manages the Ask AI feature which allows users to record voice questions,
//! transcribe them, and get AI responses via Ollama.
//!
//! Supports multi-turn conversations where users can ask follow-up questions
//! by triggering the shortcut again while the modal is open.

use crate::managers::transcription::TranscriptionManager;
use crate::ollama_client::OllamaClient;
use crate::overlay::{hide_recording_overlay, reset_overlay_size, show_ask_ai_response_overlay};
use crate::settings::get_settings;
use crate::tray::{change_tray_icon, TrayIconState};
use chrono::Utc;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use uuid::Uuid;

/// Maximum number of conversation turns to include in context
const MAX_CONTEXT_TURNS: usize = 10;

/// State of the Ask AI session
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum AskAiState {
    /// No active session
    Idle,
    /// Recording the user's question
    Recording,
    /// Transcribing the recorded audio
    Transcribing,
    /// Generating AI response
    Generating,
    /// Response complete, waiting for follow-up
    Complete,
    /// Conversation active, waiting for follow-up question
    ConversationActive,
    /// Error occurred
    Error,
}

impl Default for AskAiState {
    fn default() -> Self {
        Self::Idle
    }
}

/// A single turn in a conversation (question + response pair)
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct ConversationTurn {
    /// Unique identifier for the turn
    pub id: String,
    /// The transcribed question
    pub question: String,
    /// The AI response
    pub response: String,
    /// Unix timestamp when this turn was created
    pub timestamp: i64,
    /// Optional reference to the audio file for this turn
    pub audio_file_name: Option<String>,
}

/// An Ask AI conversation consisting of multiple turns
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct AskAiConversation {
    /// Unique identifier for the conversation
    pub id: String,
    /// All Q&A turns in this conversation
    pub turns: Vec<ConversationTurn>,
    /// Unix timestamp when conversation was created
    pub created_at: i64,
    /// Unix timestamp when conversation was last updated
    pub updated_at: i64,
    /// Auto-generated title from first question
    pub title: Option<String>,
}

impl AskAiConversation {
    /// Create a new conversation
    pub fn new() -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            turns: Vec::new(),
            created_at: now,
            updated_at: now,
            title: None,
        }
    }

    /// Add a turn to the conversation
    pub fn add_turn(&mut self, question: String, response: String, audio_file_name: Option<String>) {
        let turn = ConversationTurn {
            id: Uuid::new_v4().to_string(),
            question: question.clone(),
            response,
            timestamp: Utc::now().timestamp(),
            audio_file_name,
        };

        // Set title from first question if not set
        if self.title.is_none() {
            self.title = Some(Self::generate_title(&question));
        }

        self.turns.push(turn);
        self.updated_at = Utc::now().timestamp();
    }

    /// Generate a title from the first question (truncated)
    fn generate_title(question: &str) -> String {
        let trimmed = question.trim();
        if trimmed.len() <= 50 {
            trimmed.to_string()
        } else {
            format!("{}...", &trimmed[..47])
        }
    }

    /// Build context string for Ollama from conversation history
    pub fn build_context(&self) -> String {
        let mut context = String::new();

        // Take the last MAX_CONTEXT_TURNS turns for context
        let start_idx = if self.turns.len() > MAX_CONTEXT_TURNS {
            self.turns.len() - MAX_CONTEXT_TURNS
        } else {
            0
        };

        for turn in &self.turns[start_idx..] {
            context.push_str(&format!("User: {}\n", turn.question));
            context.push_str(&format!("Assistant: {}\n\n", turn.response));
        }

        context
    }
}

impl Default for AskAiConversation {
    fn default() -> Self {
        Self::new()
    }
}

/// Event payload for Ask AI state changes
#[derive(Clone, Debug, Serialize, Type)]
pub struct AskAiStateEvent {
    pub state: AskAiState,
    pub question: Option<String>,
    pub error: Option<String>,
    /// Full conversation for UI display
    pub conversation: Option<AskAiConversation>,
}

/// Event payload for Ask AI response streaming
#[derive(Clone, Debug, Serialize, Type)]
pub struct AskAiResponseEvent {
    pub chunk: String,
    pub done: bool,
}

/// Ask AI Manager
///
/// Coordinates the Ask AI feature:
/// - Audio recording via the shortcut action
/// - Transcription of recorded audio
/// - Sending question to Ollama
/// - Streaming response back to the overlay window
/// - Multi-turn conversation support
pub struct AskAiManager {
    app_handle: AppHandle,
    transcription_manager: Arc<TranscriptionManager>,

    /// Current state
    state: Arc<Mutex<AskAiState>>,

    /// The transcribed question for current turn
    current_question: Arc<Mutex<Option<String>>>,

    /// The complete AI response for current turn
    current_response: Arc<Mutex<String>>,

    /// Audio samples for current turn
    current_audio_samples: Arc<Mutex<Vec<f32>>>,

    /// Active conversation (multi-turn)
    active_conversation: Arc<Mutex<Option<AskAiConversation>>>,

    /// Cancellation signal for current operation
    cancel_signal: Arc<AtomicBool>,
}

impl AskAiManager {
    /// Create a new AskAiManager
    pub fn new(
        app_handle: &AppHandle,
        transcription_manager: Arc<TranscriptionManager>,
    ) -> Result<Self, anyhow::Error> {
        Ok(Self {
            app_handle: app_handle.clone(),
            transcription_manager,
            state: Arc::new(Mutex::new(AskAiState::Idle)),
            current_question: Arc::new(Mutex::new(None)),
            current_response: Arc::new(Mutex::new(String::new())),
            current_audio_samples: Arc::new(Mutex::new(Vec::new())),
            active_conversation: Arc::new(Mutex::new(None)),
            cancel_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Get the current state
    pub fn get_state(&self) -> AskAiState {
        self.state.lock().unwrap().clone()
    }

    /// Get the current question
    pub fn get_question(&self) -> Option<String> {
        self.current_question.lock().unwrap().clone()
    }

    /// Get the current response
    pub fn get_response(&self) -> String {
        self.current_response.lock().unwrap().clone()
    }

    /// Get the active conversation
    pub fn get_conversation(&self) -> Option<AskAiConversation> {
        self.active_conversation.lock().unwrap().clone()
    }

    /// Check if a session is active
    pub fn is_active(&self) -> bool {
        let state = self.state.lock().unwrap();
        !matches!(*state, AskAiState::Idle)
    }

    /// Check if we can start a new recording (either from idle or from conversation active)
    pub fn can_start_recording(&self) -> bool {
        let state = self.state.lock().unwrap();
        matches!(
            *state,
            AskAiState::Idle | AskAiState::Complete | AskAiState::ConversationActive
        )
    }

    /// Start recording - called when shortcut is pressed
    /// Can be called from Idle (new conversation) or ConversationActive/Complete (follow-up)
    pub fn start_recording(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();

        // Allow starting from idle, complete, or conversation active states
        let is_follow_up = matches!(
            *state,
            AskAiState::Complete | AskAiState::ConversationActive
        );

        if !matches!(
            *state,
            AskAiState::Idle | AskAiState::Complete | AskAiState::ConversationActive
        ) {
            return Err("Ask AI session busy".to_string());
        }

        // Update state to recording
        *state = AskAiState::Recording;
        drop(state);

        // Clear current turn data (but NOT the conversation for follow-ups)
        {
            let mut question = self.current_question.lock().unwrap();
            *question = None;
        }
        {
            let mut response = self.current_response.lock().unwrap();
            response.clear();
        }
        {
            let mut audio_samples = self.current_audio_samples.lock().unwrap();
            audio_samples.clear();
        }

        // If this is a new conversation (not a follow-up), create a new conversation
        if !is_follow_up {
            let mut conversation = self.active_conversation.lock().unwrap();
            *conversation = Some(AskAiConversation::new());
        }

        // Reset cancel signal
        self.cancel_signal.store(false, Ordering::SeqCst);

        // Emit state change with conversation
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(AskAiState::Recording, None, None, conversation);

        info!(
            "Ask AI: Started recording (follow_up: {})",
            is_follow_up
        );
        Ok(())
    }

    /// Start a new conversation (clears any existing conversation)
    pub fn start_new_conversation(&self) -> Result<(), String> {
        // Reset everything
        self.reset();

        // Create new conversation
        {
            let mut conversation = self.active_conversation.lock().unwrap();
            *conversation = Some(AskAiConversation::new());
        }

        info!("Ask AI: Started new conversation");
        Ok(())
    }

    /// Process the recorded audio - called when shortcut is released
    pub fn process_question(&self, samples: Vec<f32>) {
        if samples.is_empty() {
            warn!("Ask AI: No audio samples to process");
            let conversation = self.active_conversation.lock().unwrap().clone();
            self.emit_state_change_with_conversation(
                AskAiState::Error,
                None,
                Some("No audio recorded".to_string()),
                conversation,
            );
            // Don't fully reset - just go back to conversation active if we have turns
            let has_turns = self
                .active_conversation
                .lock()
                .unwrap()
                .as_ref()
                .map(|c| !c.turns.is_empty())
                .unwrap_or(false);
            if has_turns {
                let mut state = self.state.lock().unwrap();
                *state = AskAiState::ConversationActive;
            } else {
                self.reset();
            }
            return;
        }

        // Store audio samples for later
        {
            let mut audio_samples = self.current_audio_samples.lock().unwrap();
            *audio_samples = samples.clone();
        }

        // Update state to transcribing
        {
            let mut state = self.state.lock().unwrap();
            *state = AskAiState::Transcribing;
        }
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(AskAiState::Transcribing, None, None, conversation);

        // Process in background
        let handle = AskAiManagerHandle {
            app_handle: self.app_handle.clone(),
            transcription_manager: self.transcription_manager.clone(),
            state: self.state.clone(),
            current_question: self.current_question.clone(),
            current_response: self.current_response.clone(),
            current_audio_samples: self.current_audio_samples.clone(),
            active_conversation: self.active_conversation.clone(),
            cancel_signal: self.cancel_signal.clone(),
        };

        tauri::async_runtime::spawn(async move {
            handle.process(samples).await;
        });
    }

    /// Cancel the current session
    pub fn cancel(&self) {
        info!("Ask AI: Cancelling session");
        self.cancel_signal.store(true, Ordering::SeqCst);
        self.reset();
    }

    /// Reset to idle state and clear conversation
    pub fn reset(&self) {
        {
            let mut state = self.state.lock().unwrap();
            *state = AskAiState::Idle;
        }
        {
            let mut conversation = self.active_conversation.lock().unwrap();
            *conversation = None;
        }
        {
            let mut question = self.current_question.lock().unwrap();
            *question = None;
        }
        {
            let mut response = self.current_response.lock().unwrap();
            response.clear();
        }
        self.emit_state_change_with_conversation(AskAiState::Idle, None, None, None);
    }

    /// Dismiss the overlay but keep conversation for potential resume
    pub fn dismiss(&self) {
        let has_turns = self
            .active_conversation
            .lock()
            .unwrap()
            .as_ref()
            .map(|c| !c.turns.is_empty())
            .unwrap_or(false);

        if has_turns {
            // Keep conversation, just mark as conversation active (hidden)
            let mut state = self.state.lock().unwrap();
            *state = AskAiState::Idle;
        } else {
            self.reset();
        }
    }

    #[allow(dead_code)]
    fn emit_state_change(&self, state: AskAiState, question: Option<String>, error: Option<String>) {
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(state, question, error, conversation);
    }

    fn emit_state_change_with_conversation(
        &self,
        state: AskAiState,
        question: Option<String>,
        error: Option<String>,
        conversation: Option<AskAiConversation>,
    ) {
        let _ = self.app_handle.emit(
            "ask-ai-state-changed",
            AskAiStateEvent {
                state,
                question,
                error,
                conversation,
            },
        );
    }
}

/// Handle for async operations
struct AskAiManagerHandle {
    app_handle: AppHandle,
    transcription_manager: Arc<TranscriptionManager>,
    state: Arc<Mutex<AskAiState>>,
    current_question: Arc<Mutex<Option<String>>>,
    current_response: Arc<Mutex<String>>,
    #[allow(dead_code)]
    current_audio_samples: Arc<Mutex<Vec<f32>>>,
    active_conversation: Arc<Mutex<Option<AskAiConversation>>>,
    cancel_signal: Arc<AtomicBool>,
}

impl AskAiManagerHandle {
    async fn process(&self, samples: Vec<f32>) {
        // Check for cancellation
        if self.cancel_signal.load(Ordering::SeqCst) {
            debug!("Ask AI: Cancelled before transcription");
            return;
        }

        // Step 1: Transcribe the audio
        debug!("Ask AI: Transcribing {} samples", samples.len());
        let transcription = match self.transcription_manager.transcribe(samples) {
            Ok(text) => text,
            Err(e) => {
                error!("Ask AI: Transcription failed: {}", e);
                self.emit_error(format!("Transcription failed: {}", e));
                return;
            }
        };

        if transcription.trim().is_empty() {
            warn!("Ask AI: Empty transcription");
            self.emit_error("No speech detected".to_string());
            return;
        }

        info!("Ask AI: Transcribed question: {}", transcription);

        // Store the question
        {
            let mut question = self.current_question.lock().unwrap();
            *question = Some(transcription.clone());
        }

        // Check for cancellation
        if self.cancel_signal.load(Ordering::SeqCst) {
            debug!("Ask AI: Cancelled after transcription");
            return;
        }

        // Update state to generating
        {
            let mut state = self.state.lock().unwrap();
            *state = AskAiState::Generating;
        }
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(
            AskAiState::Generating,
            Some(transcription.clone()),
            None,
            conversation,
        );

        // Show the Ask AI response overlay (replaces the recording overlay with expanded view)
        show_ask_ai_response_overlay(&self.app_handle);
        change_tray_icon(&self.app_handle, TrayIconState::Idle);

        // Step 2: Get AI response from Ollama
        let settings = get_settings(&self.app_handle);
        let ask_ai_settings = &settings.ask_ai;

        if ask_ai_settings.ollama_model.is_empty() {
            warn!("Ask AI: No Ollama model configured");
            self.emit_error(
                "No Ollama model configured. Please configure an Ollama model in Ask AI settings."
                    .to_string(),
            );
            return;
        }

        // Build the prompt with conversation context and system prompt
        let prompt = self.build_prompt(&transcription, &ask_ai_settings.system_prompt);

        let client = match OllamaClient::new(&ask_ai_settings.ollama_base_url) {
            Ok(c) => c,
            Err(e) => {
                error!("Ask AI: Failed to create Ollama client: {}", e);
                self.emit_error(format!("Failed to create Ollama client: {}", e));
                return;
            }
        };
        let (tx, mut rx) = mpsc::channel::<String>(100);

        let app_handle_clone = self.app_handle.clone();
        let current_response = self.current_response.clone();
        let cancel_signal = self.cancel_signal.clone();

        // Spawn task to forward stream chunks to frontend
        let stream_forward_handle = tauri::async_runtime::spawn(async move {
            let mut full_response = String::new();
            while let Some(chunk) = rx.recv().await {
                // Check for cancellation
                if cancel_signal.load(Ordering::SeqCst) {
                    debug!("Ask AI: Stream forwarding cancelled");
                    break;
                }

                full_response.push_str(&chunk);

                // Update stored response
                {
                    let mut response = current_response.lock().unwrap();
                    response.push_str(&chunk);
                }

                // Emit chunk to frontend
                let _ = app_handle_clone.emit(
                    "ask-ai-response",
                    AskAiResponseEvent {
                        chunk,
                        done: false,
                    },
                );
            }
            full_response
        });

        // Call Ollama
        let ollama_result = client
            .generate_stream(&ask_ai_settings.ollama_model, prompt, tx)
            .await;

        // Wait for stream forwarding to complete
        let full_response = stream_forward_handle.await.unwrap_or_default();

        // Check for cancellation
        if self.cancel_signal.load(Ordering::SeqCst) {
            debug!("Ask AI: Cancelled during generation");
            reset_overlay_size(&self.app_handle);
            hide_recording_overlay(&self.app_handle);
            return;
        }

        // Handle result
        match ollama_result {
            Ok(_) => {
                // Add turn to conversation
                {
                    let mut conversation = self.active_conversation.lock().unwrap();
                    if let Some(ref mut conv) = *conversation {
                        conv.add_turn(transcription.clone(), full_response.clone(), None);
                    }
                }

                // Emit done signal
                let _ = self.app_handle.emit(
                    "ask-ai-response",
                    AskAiResponseEvent {
                        chunk: String::new(),
                        done: true,
                    },
                );

                // Update state to complete (conversation active)
                {
                    let mut state = self.state.lock().unwrap();
                    *state = AskAiState::Complete;
                }
                let conversation = self.active_conversation.lock().unwrap().clone();
                self.emit_state_change_with_conversation(
                    AskAiState::Complete,
                    Some(transcription.clone()),
                    None,
                    conversation,
                );

                info!("Ask AI: Response complete");
            }
            Err(e) => {
                error!("Ask AI: Ollama generation failed: {}", e);
                self.emit_error(format!("AI generation failed: {}", e));
            }
        }
    }

    /// Build the prompt with conversation context and system prompt
    fn build_prompt(&self, new_question: &str, system_prompt: &str) -> String {
        let conversation = self.active_conversation.lock().unwrap();

        // Start with system prompt if provided
        let system_section = if system_prompt.is_empty() {
            String::new()
        } else {
            format!("System: {}\n\n", system_prompt)
        };

        if let Some(ref conv) = *conversation {
            if conv.turns.is_empty() {
                // First question
                format!("{}User: {}", system_section, new_question)
            } else {
                // Multi-turn - include context
                let context = conv.build_context();
                format!("{}{}User: {}", system_section, context, new_question)
            }
        } else {
            // No conversation
            format!("{}User: {}", system_section, new_question)
        }
    }

    #[allow(dead_code)]
    fn emit_state_change(&self, state: AskAiState, question: Option<String>, error: Option<String>) {
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(state, question, error, conversation);
    }

    fn emit_state_change_with_conversation(
        &self,
        state: AskAiState,
        question: Option<String>,
        error: Option<String>,
        conversation: Option<AskAiConversation>,
    ) {
        let _ = self.app_handle.emit(
            "ask-ai-state-changed",
            AskAiStateEvent {
                state,
                question,
                error,
                conversation,
            },
        );
    }

    fn emit_error(&self, error: String) {
        {
            let mut state = self.state.lock().unwrap();
            *state = AskAiState::Error;
        }
        // Show error in the expanded overlay
        show_ask_ai_response_overlay(&self.app_handle);
        change_tray_icon(&self.app_handle, TrayIconState::Idle);
        let conversation = self.active_conversation.lock().unwrap().clone();
        self.emit_state_change_with_conversation(AskAiState::Error, None, Some(error), conversation);
    }
}

impl Drop for AskAiManager {
    fn drop(&mut self) {
        debug!("Shutting down AskAiManager");
        self.cancel_signal.store(true, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new_creates_unique_id() {
        let conv1 = AskAiConversation::new();
        let conv2 = AskAiConversation::new();

        assert!(!conv1.id.is_empty());
        assert!(!conv2.id.is_empty());
        assert_ne!(conv1.id, conv2.id);
    }

    #[test]
    fn test_conversation_new_has_empty_turns() {
        let conv = AskAiConversation::new();

        assert!(conv.turns.is_empty());
        assert!(conv.title.is_none());
    }

    #[test]
    fn test_conversation_new_sets_timestamps() {
        let before = Utc::now().timestamp();
        let conv = AskAiConversation::new();
        let after = Utc::now().timestamp();

        assert!(conv.created_at >= before && conv.created_at <= after);
        assert!(conv.updated_at >= before && conv.updated_at <= after);
    }

    #[test]
    fn test_add_turn_sets_title_from_first_question() {
        let mut conv = AskAiConversation::new();

        conv.add_turn("What is Rust?".to_string(), "A programming language.".to_string(), None);

        assert_eq!(conv.title, Some("What is Rust?".to_string()));
        assert_eq!(conv.turns.len(), 1);
    }

    #[test]
    fn test_add_turn_truncates_long_title() {
        let mut conv = AskAiConversation::new();

        let long_question = "This is a very long question that should be truncated to 47 characters plus ellipsis to make exactly 50";
        conv.add_turn(long_question.to_string(), "Response".to_string(), None);

        let title = conv.title.as_ref().unwrap();
        assert!(title.len() == 50);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_add_turn_does_not_overwrite_title() {
        let mut conv = AskAiConversation::new();

        conv.add_turn("First question".to_string(), "First response".to_string(), None);
        conv.add_turn("Second question".to_string(), "Second response".to_string(), None);

        assert_eq!(conv.title, Some("First question".to_string()));
        assert_eq!(conv.turns.len(), 2);
    }

    #[test]
    fn test_add_turn_updates_timestamp() {
        let mut conv = AskAiConversation::new();
        let initial_updated_at = conv.updated_at;

        // Wait a tiny bit to ensure time difference (though in practice timestamps should differ)
        std::thread::sleep(std::time::Duration::from_millis(10));

        conv.add_turn("Question".to_string(), "Response".to_string(), None);

        assert!(conv.updated_at >= initial_updated_at);
    }

    #[test]
    fn test_add_turn_stores_audio_file_name() {
        let mut conv = AskAiConversation::new();

        conv.add_turn(
            "Question".to_string(),
            "Response".to_string(),
            Some("audio_123.wav".to_string()),
        );

        assert_eq!(
            conv.turns[0].audio_file_name,
            Some("audio_123.wav".to_string())
        );
    }

    #[test]
    fn test_build_context_empty_conversation() {
        let conv = AskAiConversation::new();

        let context = conv.build_context();

        assert!(context.is_empty());
    }

    #[test]
    fn test_build_context_single_turn() {
        let mut conv = AskAiConversation::new();
        conv.add_turn("Hello".to_string(), "Hi there!".to_string(), None);

        let context = conv.build_context();

        assert!(context.contains("User: Hello"));
        assert!(context.contains("Assistant: Hi there!"));
    }

    #[test]
    fn test_build_context_multiple_turns() {
        let mut conv = AskAiConversation::new();
        conv.add_turn("Question 1".to_string(), "Answer 1".to_string(), None);
        conv.add_turn("Question 2".to_string(), "Answer 2".to_string(), None);

        let context = conv.build_context();

        assert!(context.contains("User: Question 1"));
        assert!(context.contains("Assistant: Answer 1"));
        assert!(context.contains("User: Question 2"));
        assert!(context.contains("Assistant: Answer 2"));
    }

    #[test]
    fn test_build_context_respects_max_context_turns() {
        let mut conv = AskAiConversation::new();

        // Add more than MAX_CONTEXT_TURNS turns
        for i in 0..(MAX_CONTEXT_TURNS + 5) {
            conv.add_turn(format!("Question {}", i), format!("Answer {}", i), None);
        }

        let context = conv.build_context();

        // Should NOT contain early turns
        assert!(!context.contains("Question 0"));
        assert!(!context.contains("Question 4"));

        // Should contain the last MAX_CONTEXT_TURNS turns
        let last_turn_index = MAX_CONTEXT_TURNS + 5 - 1;
        assert!(context.contains(&format!("Question {}", last_turn_index)));
    }

    #[test]
    fn test_generate_title_short_question() {
        let title = AskAiConversation::generate_title("Short question");
        assert_eq!(title, "Short question");
    }

    #[test]
    fn test_generate_title_exactly_50_chars() {
        let question = "x".repeat(50);
        let title = AskAiConversation::generate_title(&question);
        assert_eq!(title.len(), 50);
        assert!(!title.ends_with("..."));
    }

    #[test]
    fn test_generate_title_over_50_chars() {
        let question = "x".repeat(60);
        let title = AskAiConversation::generate_title(&question);
        assert_eq!(title.len(), 50);
        assert!(title.ends_with("..."));
    }

    #[test]
    fn test_generate_title_trims_whitespace() {
        let title = AskAiConversation::generate_title("  Hello world  ");
        assert_eq!(title, "Hello world");
    }

    #[test]
    fn test_default_state_is_idle() {
        let state = AskAiState::default();
        assert_eq!(state, AskAiState::Idle);
    }

    #[test]
    fn test_conversation_turn_id_is_unique() {
        let mut conv = AskAiConversation::new();
        conv.add_turn("Q1".to_string(), "A1".to_string(), None);
        conv.add_turn("Q2".to_string(), "A2".to_string(), None);

        assert_ne!(conv.turns[0].id, conv.turns[1].id);
    }
}
