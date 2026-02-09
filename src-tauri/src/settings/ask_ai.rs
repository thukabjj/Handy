use serde::{Deserialize, Serialize};
use specta::Type;

/// Settings for the Ask AI feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AskAiSettings {
    /// Whether Ask AI feature is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Ollama server base URL
    #[serde(default = "default_ollama_base_url")]
    pub ollama_base_url: String,

    /// Ollama model to use for generating responses
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,

    /// System prompt for the AI assistant
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    /// Saved window width for the Ask AI overlay
    #[serde(default)]
    pub window_width: Option<f64>,

    /// Saved window height for the Ask AI overlay
    #[serde(default)]
    pub window_height: Option<f64>,

    /// Saved window X position for the Ask AI overlay
    #[serde(default)]
    pub window_x: Option<f64>,

    /// Saved window Y position for the Ask AI overlay
    #[serde(default)]
    pub window_y: Option<f64>,
}

fn default_enabled() -> bool {
    true
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    String::new()
}

fn default_system_prompt() -> String {
    "You are a helpful AI assistant. Provide clear, concise, and accurate responses.".to_string()
}

impl Default for AskAiSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            ollama_base_url: default_ollama_base_url(),
            ollama_model: default_ollama_model(),
            system_prompt: default_system_prompt(),
            window_width: None,
            window_height: None,
            window_x: None,
            window_y: None,
        }
    }
}
