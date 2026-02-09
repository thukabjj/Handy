//! Knowledge Base (RAG) Settings
//!
//! Settings for the local RAG-based knowledge base feature.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Settings for the Knowledge Base feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct KnowledgeBaseSettings {
    /// Whether knowledge base is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Automatically index transcriptions from Active Listening
    #[serde(default = "default_auto_index")]
    pub auto_index_transcriptions: bool,

    /// Embedding model to use (Ollama model name)
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,

    /// Number of context chunks to retrieve per query
    #[serde(default = "default_top_k")]
    pub top_k: usize,

    /// Minimum similarity threshold for including results (0.0-1.0)
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,

    /// Use RAG context in Active Listening prompts
    #[serde(default = "default_use_in_active_listening")]
    pub use_in_active_listening: bool,
}

fn default_enabled() -> bool {
    false // Disabled by default, user needs to opt-in
}

fn default_auto_index() -> bool {
    true // Auto-index when KB is enabled
}

fn default_embedding_model() -> String {
    "nomic-embed-text".to_string()
}

fn default_top_k() -> usize {
    3
}

fn default_similarity_threshold() -> f32 {
    0.5
}

fn default_use_in_active_listening() -> bool {
    true
}

impl Default for KnowledgeBaseSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            auto_index_transcriptions: default_auto_index(),
            embedding_model: default_embedding_model(),
            top_k: default_top_k(),
            similarity_threshold: default_similarity_threshold(),
            use_in_active_listening: default_use_in_active_listening(),
        }
    }
}
