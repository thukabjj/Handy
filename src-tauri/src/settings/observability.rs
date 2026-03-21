use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum ObservabilityDataMode {
    #[default]
    MetadataOnly,
    SensitiveText,
    Developer,
}

impl ObservabilityDataMode {
    pub fn includes_sensitive_content(self) -> bool {
        matches!(self, Self::SensitiveText | Self::Developer)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ObservabilitySettings {
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub data_mode: ObservabilityDataMode,
    #[serde(default = "default_max_events")]
    pub max_events: u32,
    #[serde(default = "default_emit_live_events")]
    pub emit_live_events: bool,
    #[serde(default = "default_capture_frontend_events")]
    pub capture_frontend_events: bool,
}

fn default_enabled() -> bool {
    true
}

fn default_max_events() -> u32 {
    400
}

fn default_emit_live_events() -> bool {
    true
}

fn default_capture_frontend_events() -> bool {
    true
}

impl Default for ObservabilitySettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            data_mode: ObservabilityDataMode::default(),
            max_events: default_max_events(),
            emit_live_events: default_emit_live_events(),
            capture_frontend_events: default_capture_frontend_events(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_safe_and_enabled() {
        let settings = ObservabilitySettings::default();
        assert!(settings.enabled);
        assert_eq!(settings.data_mode, ObservabilityDataMode::MetadataOnly);
        assert_eq!(settings.max_events, 400);
        assert!(settings.emit_live_events);
        assert!(settings.capture_frontend_events);
    }

    #[test]
    fn sensitive_modes_are_detected() {
        assert!(!ObservabilityDataMode::MetadataOnly.includes_sensitive_content());
        assert!(ObservabilityDataMode::SensitiveText.includes_sensitive_content());
        assert!(ObservabilityDataMode::Developer.includes_sensitive_content());
    }
}
