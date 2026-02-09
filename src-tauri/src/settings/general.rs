use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct GeneralSettings {
    pub push_to_talk: bool,
    #[serde(default = "default_start_hidden")]
    pub start_hidden: bool,
    #[serde(default = "default_autostart_enabled")]
    pub autostart_enabled: bool,
    #[serde(default = "default_update_checks_enabled")]
    pub update_checks_enabled: bool,
    #[serde(default)]
    pub mute_while_recording: bool,
    #[serde(default)]
    pub append_trailing_space: bool,
    #[serde(default = "default_app_language")]
    pub app_language: String,
    /// Hide overlay from screen capture/sharing (Zoom, Teams, etc.)
    /// Enabled by default for privacy during screen sharing
    #[serde(default = "default_private_overlay")]
    pub private_overlay: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            push_to_talk: true,
            start_hidden: default_start_hidden(),
            autostart_enabled: default_autostart_enabled(),
            update_checks_enabled: default_update_checks_enabled(),
            mute_while_recording: false,
            append_trailing_space: false,
            app_language: default_app_language(),
            private_overlay: default_private_overlay(),
        }
    }
}

fn default_start_hidden() -> bool {
    false
}

fn default_autostart_enabled() -> bool {
    false
}

fn default_update_checks_enabled() -> bool {
    true
}

fn default_app_language() -> String {
    tauri_plugin_os::locale()
        .and_then(|l| l.split(['-', '_']).next().map(String::from))
        .unwrap_or_else(|| "en".to_string())
}

fn default_private_overlay() -> bool {
    // Enabled by default for privacy during screen sharing
    true
}
