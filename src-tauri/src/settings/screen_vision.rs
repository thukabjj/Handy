use crate::settings::active_listening::LlmProvider;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ScreenVisionSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_interval_seconds")]
    pub interval_seconds: u64,
    #[serde(default = "default_duration_seconds")]
    pub duration_seconds: u64,
    #[serde(default = "default_max_snapshots")]
    pub max_snapshots: u32,
    #[serde(default = "default_llm_provider")]
    pub llm_provider: LlmProvider,
    #[serde(default)]
    pub llm_api_key: Option<String>,
    #[serde(default)]
    pub llm_base_url: Option<String>,
    #[serde(default = "default_llm_model")]
    pub llm_model: String,
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

fn default_interval_seconds() -> u64 {
    5
}

fn default_duration_seconds() -> u64 {
    30
}

fn default_max_snapshots() -> u32 {
    6
}

fn default_llm_provider() -> LlmProvider {
    LlmProvider::OpenRouter
}

fn default_llm_model() -> String {
    "qwen/qwen2.5-vl-72b-instruct:free".to_string()
}

fn default_prompt() -> String {
    "Analyze this screen for actionable context. Return concise bullet points for what is visible, what requires attention, and recommended next action.".to_string()
}

impl Default for ScreenVisionSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            interval_seconds: default_interval_seconds(),
            duration_seconds: default_duration_seconds(),
            max_snapshots: default_max_snapshots(),
            llm_provider: default_llm_provider(),
            llm_api_key: None,
            llm_base_url: None,
            llm_model: default_llm_model(),
            prompt: default_prompt(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_are_stable() {
        let settings = ScreenVisionSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.interval_seconds, 5);
        assert_eq!(settings.duration_seconds, 30);
        assert_eq!(settings.max_snapshots, 6);
        assert_eq!(settings.llm_provider, LlmProvider::OpenRouter);
        assert_eq!(settings.llm_model, "qwen/qwen2.5-vl-72b-instruct:free");
        assert!(!settings.prompt.is_empty());
    }

    #[test]
    fn deserialize_missing_fields_uses_defaults() {
        let parsed: ScreenVisionSettings =
            serde_json::from_str(r#"{"enabled":true,"llm_model":"test/model"}"#).unwrap();
        assert!(parsed.enabled);
        assert_eq!(parsed.llm_model, "test/model");
        assert_eq!(parsed.interval_seconds, 5);
        assert_eq!(parsed.duration_seconds, 30);
        assert_eq!(parsed.max_snapshots, 6);
        assert_eq!(parsed.llm_provider, LlmProvider::OpenRouter);
    }
}
