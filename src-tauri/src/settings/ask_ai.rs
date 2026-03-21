use crate::settings::active_listening::LlmProvider;
use serde::{Deserialize, Serialize};
use specta::Type;

/// Settings for the Ask AI feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AskAiSettings {
    /// Whether Ask AI feature is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// LLM provider to use for generating responses
    #[serde(default)]
    pub llm_provider: LlmProvider,

    /// API key for cloud LLM providers (OpenRouter, etc.)
    #[serde(default)]
    pub llm_api_key: Option<String>,

    /// Model to use with the cloud LLM provider
    #[serde(default)]
    pub llm_model: Option<String>,

    /// Custom base URL for the LLM provider (used with Custom provider)
    #[serde(default)]
    pub llm_base_url: Option<String>,

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
            llm_provider: LlmProvider::default(),
            llm_api_key: None,
            llm_model: None,
            llm_base_url: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_ask_ai_settings() {
        let settings = AskAiSettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.llm_provider, LlmProvider::Ollama);
        assert_eq!(settings.ollama_base_url, "http://localhost:11434");
        assert!(settings.ollama_model.is_empty());
        assert!(settings.llm_model.is_none());
        assert!(settings.llm_base_url.is_none());
        assert!(settings.system_prompt.contains("helpful AI assistant"));
    }

    #[test]
    fn test_deserialize_with_new_cloud_providers() {
        let providers = [
            ("local_open_ai", LlmProvider::LocalOpenAi),
            ("open_router", LlmProvider::OpenRouter),
            ("open_ai", LlmProvider::OpenAi),
            ("groq", LlmProvider::Groq),
            ("together", LlmProvider::Together),
            ("fireworks", LlmProvider::Fireworks),
            ("custom", LlmProvider::Custom),
        ];

        for (provider_str, expected_provider) in providers {
            let json = format!(r#"{{"enabled":true,"llm_provider":"{}"}}"#, provider_str);
            let settings: AskAiSettings =
                serde_json::from_str(&json).expect("deserialize Ask AI settings");
            assert_eq!(settings.llm_provider, expected_provider);
        }
    }

    #[test]
    fn test_deserialize_missing_fields_uses_defaults() {
        let settings: AskAiSettings = serde_json::from_str(r#"{"enabled":false}"#).unwrap();
        assert!(!settings.enabled);
        assert_eq!(settings.llm_provider, LlmProvider::Ollama);
        assert_eq!(settings.ollama_base_url, "http://localhost:11434");
    }
}
