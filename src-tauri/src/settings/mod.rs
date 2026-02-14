use log::{debug, warn};
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use specta::Type;
use std::collections::HashMap;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

pub mod active_listening;
pub mod ask_ai;
pub mod general;
pub mod knowledge_base;
pub mod sound_detection;
pub mod suggestions;

pub use active_listening::{
    ActiveListeningPrompt, ActiveListeningSettings, AudioSourceType, PromptCategory,
};
pub use ask_ai::AskAiSettings;
pub use knowledge_base::KnowledgeBaseSettings;
pub use sound_detection::{SoundCategory, SoundDetectionSettings};
pub use suggestions::{QuickResponse, SuggestionsSettings, WarningSeverity};

pub const APPLE_INTELLIGENCE_PROVIDER_ID: &str = "apple_intelligence";
pub const APPLE_INTELLIGENCE_DEFAULT_MODEL_ID: &str = "Apple Intelligence";

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

// Custom deserializer to handle both old numeric format (1-5) and new string format ("trace", "debug", etc.)
impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LogLevelVisitor;

        impl<'de> Visitor<'de> for LogLevelVisitor {
            type Value = LogLevel;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string or integer representing log level")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<LogLevel, E> {
                match value.to_lowercase().as_str() {
                    "trace" => Ok(LogLevel::Trace),
                    "debug" => Ok(LogLevel::Debug),
                    "info" => Ok(LogLevel::Info),
                    "warn" => Ok(LogLevel::Warn),
                    "error" => Ok(LogLevel::Error),
                    _ => Err(E::unknown_variant(
                        value,
                        &["trace", "debug", "info", "warn", "error"],
                    )),
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<LogLevel, E> {
                match value {
                    1 => Ok(LogLevel::Trace),
                    2 => Ok(LogLevel::Debug),
                    3 => Ok(LogLevel::Info),
                    4 => Ok(LogLevel::Warn),
                    5 => Ok(LogLevel::Error),
                    _ => Err(E::invalid_value(de::Unexpected::Unsigned(value), &"1-5")),
                }
            }
        }

        deserializer.deserialize_any(LogLevelVisitor)
    }
}

impl From<LogLevel> for tauri_plugin_log::LogLevel {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => tauri_plugin_log::LogLevel::Trace,
            LogLevel::Debug => tauri_plugin_log::LogLevel::Debug,
            LogLevel::Info => tauri_plugin_log::LogLevel::Info,
            LogLevel::Warn => tauri_plugin_log::LogLevel::Warn,
            LogLevel::Error => tauri_plugin_log::LogLevel::Error,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ShortcutBinding {
    pub id: String,
    pub name: String,
    pub description: String,
    pub default_binding: String,
    pub current_binding: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct LLMPrompt {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct PostProcessProvider {
    pub id: String,
    pub label: String,
    pub base_url: String,
    #[serde(default)]
    pub allow_base_url_edit: bool,
    #[serde(default)]
    pub models_endpoint: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "lowercase")]
pub enum OverlayPosition {
    None,
    Top,
    Bottom,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ModelUnloadTimeout {
    Never,
    Immediately,
    Min2,
    Min5,
    Min10,
    Min15,
    Hour1,
    Sec5, // Debug mode only
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum PasteMethod {
    CtrlV,
    Direct,
    None,
    ShiftInsert,
    CtrlShiftV,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum ClipboardHandling {
    DontModify,
    CopyToClipboard,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum RecordingRetentionPeriod {
    Never,
    PreserveLimit,
    Days3,
    Weeks2,
    Months3,
}

impl Default for ModelUnloadTimeout {
    fn default() -> Self {
        ModelUnloadTimeout::Never
    }
}

impl Default for PasteMethod {
    fn default() -> Self {
        // Default to CtrlV for macOS and Windows, Direct for Linux
        #[cfg(target_os = "linux")]
        return PasteMethod::Direct;
        #[cfg(not(target_os = "linux"))]
        return PasteMethod::CtrlV;
    }
}

impl Default for ClipboardHandling {
    fn default() -> Self {
        ClipboardHandling::DontModify
    }
}

impl ModelUnloadTimeout {
    pub fn to_minutes(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Min2 => Some(2),
            ModelUnloadTimeout::Min5 => Some(5),
            ModelUnloadTimeout::Min10 => Some(10),
            ModelUnloadTimeout::Min15 => Some(15),
            ModelUnloadTimeout::Hour1 => Some(60),
            ModelUnloadTimeout::Sec5 => Some(0), // Special case for debug - handled separately
        }
    }

    pub fn to_seconds(self) -> Option<u64> {
        match self {
            ModelUnloadTimeout::Never => None,
            ModelUnloadTimeout::Immediately => Some(0), // Special case for immediate unloading
            ModelUnloadTimeout::Sec5 => Some(5),
            _ => self.to_minutes().map(|m| m * 60),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum SoundTheme {
    Marimba,
    Pop,
    Custom,
}

impl SoundTheme {
    fn as_str(&self) -> &'static str {
        match self {
            SoundTheme::Marimba => "marimba",
            SoundTheme::Pop => "pop",
            SoundTheme::Custom => "custom",
        }
    }

    pub fn to_start_path(&self) -> String {
        format!("resources/{}_start.wav", self.as_str())
    }

    pub fn to_stop_path(&self) -> String {
        format!("resources/{}_stop.wav", self.as_str())
    }
}

/* still handy for composing the initial JSON in the store ------------- */
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AppSettings {
    #[serde(flatten)]
    pub general: general::GeneralSettings,
    pub bindings: HashMap<String, ShortcutBinding>,
    pub audio_feedback: bool,
    #[serde(default = "default_audio_feedback_volume")]
    pub audio_feedback_volume: f32,
    #[serde(default = "default_sound_theme")]
    pub sound_theme: SoundTheme,
    #[serde(default = "default_model")]
    pub selected_model: String,
    #[serde(default = "default_always_on_microphone")]
    pub always_on_microphone: bool,
    #[serde(default)]
    pub selected_microphone: Option<String>,
    #[serde(default)]
    pub clamshell_microphone: Option<String>,
    #[serde(default)]
    pub selected_output_device: Option<String>,
    #[serde(default = "default_translate_to_english")]
    pub translate_to_english: bool,
    #[serde(default = "default_selected_language")]
    pub selected_language: String,
    #[serde(default = "default_overlay_position")]
    pub overlay_position: OverlayPosition,
    #[serde(default = "default_debug_mode")]
    pub debug_mode: bool,
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    #[serde(default)]
    pub custom_words: Vec<String>,
    #[serde(default)]
    pub model_unload_timeout: ModelUnloadTimeout,
    #[serde(default = "default_word_correction_threshold")]
    pub word_correction_threshold: f64,
    #[serde(default = "default_history_limit")]
    pub history_limit: usize,
    #[serde(default = "default_recording_retention_period")]
    pub recording_retention_period: RecordingRetentionPeriod,
    #[serde(default)]
    pub paste_method: PasteMethod,
    #[serde(default = "default_paste_delay_ms")]
    pub paste_delay_ms: u64,
    #[serde(default)]
    pub clipboard_handling: ClipboardHandling,
    #[serde(default = "default_post_process_enabled")]
    pub post_process_enabled: bool,
    #[serde(default = "default_post_process_provider_id")]
    pub post_process_provider_id: String,
    #[serde(default = "default_post_process_providers")]
    pub post_process_providers: Vec<PostProcessProvider>,
    #[serde(default = "default_post_process_api_keys")]
    pub post_process_api_keys: HashMap<String, String>,
    #[serde(default = "default_post_process_models")]
    pub post_process_models: HashMap<String, String>,
    #[serde(default = "default_post_process_prompts")]
    pub post_process_prompts: Vec<LLMPrompt>,
    #[serde(default)]
    pub post_process_selected_prompt_id: Option<String>,
    #[serde(default)]
    pub active_listening: ActiveListeningSettings,
    #[serde(default)]
    pub ask_ai: AskAiSettings,
    #[serde(default)]
    pub knowledge_base: KnowledgeBaseSettings,
    #[serde(default)]
    pub suggestions: SuggestionsSettings,
    #[serde(default)]
    pub sound_detection: SoundDetectionSettings,
}

fn default_model() -> String {
    "".to_string()
}

fn default_always_on_microphone() -> bool {
    false
}

fn default_translate_to_english() -> bool {
    false
}

fn default_selected_language() -> String {
    "auto".to_string()
}

fn default_overlay_position() -> OverlayPosition {
    #[cfg(target_os = "linux")]
    return OverlayPosition::None;
    #[cfg(not(target_os = "linux"))]
    return OverlayPosition::Bottom;
}

fn default_debug_mode() -> bool {
    false
}

fn default_log_level() -> LogLevel {
    LogLevel::Debug
}

fn default_word_correction_threshold() -> f64 {
    0.18
}

fn default_history_limit() -> usize {
    5
}

fn default_recording_retention_period() -> RecordingRetentionPeriod {
    RecordingRetentionPeriod::PreserveLimit
}

fn default_audio_feedback_volume() -> f32 {
    1.0
}

fn default_sound_theme() -> SoundTheme {
    SoundTheme::Marimba
}

fn default_paste_delay_ms() -> u64 {
    50
}

fn default_post_process_enabled() -> bool {
    false
}

fn default_post_process_provider_id() -> String {
    "openai".to_string()
}

fn default_post_process_providers() -> Vec<PostProcessProvider> {
    let mut providers = vec![
        PostProcessProvider {
            id: "openai".to_string(),
            label: "OpenAI".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
        },
        PostProcessProvider {
            id: "openrouter".to_string(),
            label: "OpenRouter".to_string(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
        },
        PostProcessProvider {
            id: "anthropic".to_string(),
            label: "Anthropic".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
        },
        PostProcessProvider {
            id: "groq".to_string(),
            label: "Groq".to_string(),
            base_url: "https://api.groq.com/openai/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
        },
        PostProcessProvider {
            id: "cerebras".to_string(),
            label: "Cerebras".to_string(),
            base_url: "https://api.cerebras.ai/v1".to_string(),
            allow_base_url_edit: false,
            models_endpoint: Some("/models".to_string()),
        },
    ];

    // Note: We always include Apple Intelligence on macOS ARM64 without checking availability
    // at startup. The availability check is deferred to when the user actually tries to use it
    // (in actions.rs). This prevents crashes on macOS 26.x beta where accessing
    // SystemLanguageModel.default during early app initialization causes SIGABRT.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        providers.push(PostProcessProvider {
            id: APPLE_INTELLIGENCE_PROVIDER_ID.to_string(),
            label: "Apple Intelligence".to_string(),
            base_url: "apple-intelligence://local".to_string(),
            allow_base_url_edit: false,
            models_endpoint: None,
        });
    }

    // Custom provider always comes last
    providers.push(PostProcessProvider {
        id: "custom".to_string(),
        label: "Custom".to_string(),
        base_url: "http://localhost:11434/v1".to_string(),
        allow_base_url_edit: true,
        models_endpoint: Some("/models".to_string()),
    });

    providers
}

fn default_post_process_api_keys() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for provider in default_post_process_providers() {
        map.insert(provider.id, String::new());
    }
    map
}

fn default_model_for_provider(provider_id: &str) -> String {
    if provider_id == APPLE_INTELLIGENCE_PROVIDER_ID {
        return APPLE_INTELLIGENCE_DEFAULT_MODEL_ID.to_string();
    }
    String::new()
}

fn default_post_process_models() -> HashMap<String, String> {
    let mut map = HashMap::new();
    for provider in default_post_process_providers() {
        map.insert(
            provider.id.clone(),
            default_model_for_provider(&provider.id),
        );
    }
    map
}

fn default_post_process_prompts() -> Vec<LLMPrompt> {
    vec![LLMPrompt {
        id: "default_improve_transcriptions".to_string(),
        name: "Improve Transcriptions".to_string(),
        prompt: "Clean this transcript:\n1. Fix spelling, capitalization, and punctuation errors\n2. Convert number words to digits (twenty-five → 25, ten percent → 10%, five dollars → $5)\n3. Replace spoken punctuation with symbols (period → ., comma → ,, question mark → ?)\n4. Remove filler words (um, uh, like as filler)\n5. Keep the language in the original version (if it was french, keep it in french for example)\n\nPreserve exact meaning and word order. Do not paraphrase or reorder content.\n\nReturn only the cleaned transcript.\n\nTranscript:\n${output}".to_string(),
    }]
}

fn ensure_post_process_defaults(settings: &mut AppSettings) -> bool {
    let mut changed = false;
    for provider in default_post_process_providers() {
        if settings
            .post_process_providers
            .iter()
            .all(|existing| existing.id != provider.id)
        {
            settings.post_process_providers.push(provider.clone());
            changed = true;
        }

        if !settings.post_process_api_keys.contains_key(&provider.id) {
            settings
                .post_process_api_keys
                .insert(provider.id.clone(), String::new());
            changed = true;
        }

        let default_model = default_model_for_provider(&provider.id);
        match settings.post_process_models.get_mut(&provider.id) {
            Some(existing) => {
                if existing.is_empty() && !default_model.is_empty() {
                    *existing = default_model.clone();
                    changed = true;
                }
            }
            None => {
                settings
                    .post_process_models
                    .insert(provider.id.clone(), default_model);
                changed = true;
            }
        }
    }

    changed
}

pub const SETTINGS_STORE_PATH: &str = "settings_store.json";

pub fn get_default_settings() -> AppSettings {
    #[cfg(target_os = "windows")]
    let default_shortcut = "ctrl+space";
    #[cfg(target_os = "macos")]
    let default_shortcut = "option+space";
    #[cfg(target_os = "linux")]
    let default_shortcut = "ctrl+space";
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    let default_shortcut = "alt+space";

    // Active listening shortcut
    #[cfg(target_os = "macos")]
    let active_listening_shortcut = "cmd+shift+l";
    #[cfg(not(target_os = "macos"))]
    let active_listening_shortcut = "ctrl+shift+l";

    // Ask AI shortcut
    #[cfg(target_os = "macos")]
    let ask_ai_shortcut = "cmd+shift+a";
    #[cfg(not(target_os = "macos"))]
    let ask_ai_shortcut = "ctrl+shift+a";

    // Toggle overlay visibility shortcut
    #[cfg(target_os = "macos")]
    let toggle_overlay_shortcut = "cmd+shift+h";
    #[cfg(not(target_os = "macos"))]
    let toggle_overlay_shortcut = "ctrl+shift+h";

    let mut bindings = HashMap::new();
    bindings.insert(
        "transcribe".to_string(),
        ShortcutBinding {
            id: "transcribe".to_string(),
            name: "Transcribe".to_string(),
            description: "Converts your speech into text.".to_string(),
            default_binding: default_shortcut.to_string(),
            current_binding: default_shortcut.to_string(),
        },
    );
    bindings.insert(
        "cancel".to_string(),
        ShortcutBinding {
            id: "cancel".to_string(),
            name: "Cancel".to_string(),
            description: "Cancels the current recording.".to_string(),
            default_binding: "escape".to_string(),
            current_binding: "escape".to_string(),
        },
    );
    bindings.insert(
        "active_listening".to_string(),
        ShortcutBinding {
            id: "active_listening".to_string(),
            name: "Active Listening".to_string(),
            description:
                "Toggle active listening mode for continuous transcription with AI insights."
                    .to_string(),
            default_binding: active_listening_shortcut.to_string(),
            current_binding: active_listening_shortcut.to_string(),
        },
    );
    bindings.insert(
        "ask_ai".to_string(),
        ShortcutBinding {
            id: "ask_ai".to_string(),
            name: "Ask AI".to_string(),
            description: "Record a voice question and get an AI response using Ollama.".to_string(),
            default_binding: ask_ai_shortcut.to_string(),
            current_binding: ask_ai_shortcut.to_string(),
        },
    );
    bindings.insert(
        "toggle_overlay".to_string(),
        ShortcutBinding {
            id: "toggle_overlay".to_string(),
            name: "Toggle Overlay".to_string(),
            description: "Temporarily hide or show the recording overlay.".to_string(),
            default_binding: toggle_overlay_shortcut.to_string(),
            current_binding: toggle_overlay_shortcut.to_string(),
        },
    );

    AppSettings {
        general: general::GeneralSettings::default(),
        bindings,
        audio_feedback: false,
        audio_feedback_volume: default_audio_feedback_volume(),
        sound_theme: default_sound_theme(),
        selected_model: "".to_string(),
        always_on_microphone: false,
        selected_microphone: None,
        clamshell_microphone: None,
        selected_output_device: None,
        translate_to_english: false,
        selected_language: "auto".to_string(),
        overlay_position: default_overlay_position(),
        debug_mode: false,
        log_level: default_log_level(),
        custom_words: Vec::new(),
        model_unload_timeout: ModelUnloadTimeout::Never,
        word_correction_threshold: default_word_correction_threshold(),
        history_limit: default_history_limit(),
        recording_retention_period: default_recording_retention_period(),
        paste_method: PasteMethod::default(),
        paste_delay_ms: default_paste_delay_ms(),
        clipboard_handling: ClipboardHandling::default(),
        post_process_enabled: default_post_process_enabled(),
        post_process_provider_id: default_post_process_provider_id(),
        post_process_providers: default_post_process_providers(),
        post_process_api_keys: default_post_process_api_keys(),
        post_process_models: default_post_process_models(),
        post_process_prompts: default_post_process_prompts(),
        post_process_selected_prompt_id: None,
        active_listening: ActiveListeningSettings::default(),
        ask_ai: AskAiSettings::default(),
        knowledge_base: KnowledgeBaseSettings::default(),
        suggestions: SuggestionsSettings::default(),
        sound_detection: SoundDetectionSettings::default(),
    }
}

impl AppSettings {
    pub fn active_post_process_provider(&self) -> Option<&PostProcessProvider> {
        self.post_process_providers
            .iter()
            .find(|provider| provider.id == self.post_process_provider_id)
    }

    pub fn post_process_provider(&self, provider_id: &str) -> Option<&PostProcessProvider> {
        self.post_process_providers
            .iter()
            .find(|provider| provider.id == provider_id)
    }

    pub fn post_process_provider_mut(
        &mut self,
        provider_id: &str,
    ) -> Option<&mut PostProcessProvider> {
        self.post_process_providers
            .iter_mut()
            .find(|provider| provider.id == provider_id)
    }
}

pub fn load_or_create_app_settings(app: &AppHandle) -> AppSettings {
    // Initialize store
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    let mut settings = if let Some(settings_value) = store.get("settings") {
        // Parse the entire settings object
        match serde_json::from_value::<AppSettings>(settings_value) {
            Ok(mut settings) => {
                debug!("Found existing settings: {:?}", settings);
                let default_settings = get_default_settings();
                let mut updated = false;

                // Merge default bindings into existing settings
                for (key, value) in default_settings.bindings {
                    if !settings.bindings.contains_key(&key) {
                        debug!("Adding missing binding: {}", key);
                        settings.bindings.insert(key, value);
                        updated = true;
                    }
                }

                if updated {
                    debug!("Settings updated with new bindings");
                    store.set("settings", serde_json::to_value(&settings).unwrap());
                }

                settings
            }
            Err(e) => {
                warn!("Failed to parse settings: {}", e);
                // Fall back to default settings if parsing fails
                let default_settings = get_default_settings();
                store.set("settings", serde_json::to_value(&default_settings).unwrap());
                default_settings
            }
        }
    } else {
        let default_settings = get_default_settings();
        store.set("settings", serde_json::to_value(&default_settings).unwrap());
        default_settings
    };

    let mut post_changed = ensure_post_process_defaults(&mut settings);
    post_changed |=
        active_listening::ensure_active_listening_defaults(&mut settings.active_listening);
    post_changed |= suggestions::ensure_suggestions_defaults(&mut settings.suggestions);

    if post_changed {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    settings
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    let mut settings = if let Some(settings_value) = store.get("settings") {
        serde_json::from_value::<AppSettings>(settings_value).unwrap_or_else(|_| {
            let default_settings = get_default_settings();
            store.set("settings", serde_json::to_value(&default_settings).unwrap());
            default_settings
        })
    } else {
        let default_settings = get_default_settings();
        store.set("settings", serde_json::to_value(&default_settings).unwrap());
        default_settings
    };

    let mut changed = ensure_post_process_defaults(&mut settings);
    changed |= active_listening::ensure_active_listening_defaults(&mut settings.active_listening);
    changed |= suggestions::ensure_suggestions_defaults(&mut settings.suggestions);

    if changed {
        store.set("settings", serde_json::to_value(&settings).unwrap());
    }

    settings
}

pub fn write_settings(app: &AppHandle, settings: AppSettings) {
    let store = app
        .store(SETTINGS_STORE_PATH)
        .expect("Failed to initialize store");

    store.set("settings", serde_json::to_value(&settings).unwrap());
}

pub fn get_bindings(app: &AppHandle) -> HashMap<String, ShortcutBinding> {
    let settings = get_settings(app);

    settings.bindings
}

pub fn get_stored_binding(app: &AppHandle, id: &str) -> ShortcutBinding {
    let bindings = get_bindings(app);

    let binding = bindings.get(id).unwrap().clone();

    binding
}

pub fn get_history_limit(app: &AppHandle) -> usize {
    let settings = get_settings(app);
    settings.history_limit
}

pub fn get_recording_retention_period(app: &AppHandle) -> RecordingRetentionPeriod {
    let settings = get_settings(app);
    settings.recording_retention_period
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings_has_required_bindings() {
        let settings = get_default_settings();

        assert!(
            settings.bindings.contains_key("transcribe"),
            "transcribe binding should exist"
        );
        assert!(
            settings.bindings.contains_key("cancel"),
            "cancel binding should exist"
        );
        assert!(
            settings.bindings.contains_key("active_listening"),
            "active_listening binding should exist"
        );
        assert!(
            settings.bindings.contains_key("ask_ai"),
            "ask_ai binding should exist"
        );
    }

    #[test]
    fn test_default_settings_binding_structure() {
        let settings = get_default_settings();

        let transcribe_binding = settings
            .bindings
            .get("transcribe")
            .expect("transcribe binding should exist");

        assert_eq!(transcribe_binding.id, "transcribe");
        assert!(!transcribe_binding.name.is_empty());
        assert!(!transcribe_binding.description.is_empty());
        assert!(!transcribe_binding.default_binding.is_empty());
        assert_eq!(
            transcribe_binding.default_binding, transcribe_binding.current_binding,
            "default and current binding should be the same initially"
        );
    }

    #[test]
    fn test_default_settings_audio_feedback() {
        let settings = get_default_settings();

        assert!(!settings.audio_feedback, "audio feedback should be off by default");
        assert_eq!(
            settings.audio_feedback_volume, 1.0,
            "audio feedback volume should default to 1.0"
        );
        assert_eq!(
            settings.sound_theme,
            SoundTheme::Marimba,
            "default sound theme should be Marimba"
        );
    }

    #[test]
    fn test_default_settings_transcription_options() {
        let settings = get_default_settings();

        assert!(
            !settings.translate_to_english,
            "translate_to_english should be off by default"
        );
        assert_eq!(
            settings.selected_language, "auto",
            "selected_language should default to 'auto'"
        );
        assert!(
            settings.custom_words.is_empty(),
            "custom_words should be empty by default"
        );
    }

    #[test]
    fn test_default_settings_model_options() {
        let settings = get_default_settings();

        assert!(
            settings.selected_model.is_empty(),
            "selected_model should be empty by default"
        );
        assert_eq!(
            settings.model_unload_timeout,
            ModelUnloadTimeout::Never,
            "model_unload_timeout should default to Never"
        );
    }

    #[test]
    fn test_default_settings_microphone_options() {
        let settings = get_default_settings();

        assert!(
            !settings.always_on_microphone,
            "always_on_microphone should be off by default"
        );
        assert!(
            settings.selected_microphone.is_none(),
            "selected_microphone should be None by default"
        );
        assert!(
            settings.clamshell_microphone.is_none(),
            "clamshell_microphone should be None by default"
        );
        assert!(
            settings.selected_output_device.is_none(),
            "selected_output_device should be None by default"
        );
    }

    #[test]
    fn test_default_settings_debug_options() {
        let settings = get_default_settings();

        assert!(!settings.debug_mode, "debug_mode should be off by default");
        assert_eq!(
            settings.log_level,
            LogLevel::Debug,
            "log_level should default to Debug"
        );
    }

    #[test]
    fn test_default_settings_history_options() {
        let settings = get_default_settings();

        assert_eq!(settings.history_limit, 5, "history_limit should default to 5");
        assert_eq!(
            settings.recording_retention_period,
            RecordingRetentionPeriod::PreserveLimit,
            "recording_retention_period should default to PreserveLimit"
        );
    }

    #[test]
    fn test_default_settings_post_process_options() {
        let settings = get_default_settings();

        assert!(
            !settings.post_process_enabled,
            "post_process_enabled should be off by default"
        );
        assert_eq!(
            settings.post_process_provider_id, "openai",
            "default provider should be openai"
        );
        assert!(
            !settings.post_process_providers.is_empty(),
            "post_process_providers should not be empty"
        );
        assert!(
            !settings.post_process_prompts.is_empty(),
            "post_process_prompts should have at least one default prompt"
        );
    }

    #[test]
    fn test_default_settings_clipboard_options() {
        let settings = get_default_settings();

        assert_eq!(
            settings.clipboard_handling,
            ClipboardHandling::DontModify,
            "clipboard_handling should default to DontModify"
        );
    }

    #[test]
    fn test_model_unload_timeout_to_minutes() {
        assert_eq!(ModelUnloadTimeout::Never.to_minutes(), None);
        assert_eq!(ModelUnloadTimeout::Immediately.to_minutes(), Some(0));
        assert_eq!(ModelUnloadTimeout::Min2.to_minutes(), Some(2));
        assert_eq!(ModelUnloadTimeout::Min5.to_minutes(), Some(5));
        assert_eq!(ModelUnloadTimeout::Min10.to_minutes(), Some(10));
        assert_eq!(ModelUnloadTimeout::Min15.to_minutes(), Some(15));
        assert_eq!(ModelUnloadTimeout::Hour1.to_minutes(), Some(60));
        assert_eq!(ModelUnloadTimeout::Sec5.to_minutes(), Some(0)); // Special debug case
    }

    #[test]
    fn test_model_unload_timeout_to_seconds() {
        assert_eq!(ModelUnloadTimeout::Never.to_seconds(), None);
        assert_eq!(ModelUnloadTimeout::Immediately.to_seconds(), Some(0));
        assert_eq!(ModelUnloadTimeout::Min2.to_seconds(), Some(120));
        assert_eq!(ModelUnloadTimeout::Min5.to_seconds(), Some(300));
        assert_eq!(ModelUnloadTimeout::Min10.to_seconds(), Some(600));
        assert_eq!(ModelUnloadTimeout::Min15.to_seconds(), Some(900));
        assert_eq!(ModelUnloadTimeout::Hour1.to_seconds(), Some(3600));
        assert_eq!(ModelUnloadTimeout::Sec5.to_seconds(), Some(5)); // Special debug case
    }

    #[test]
    fn test_sound_theme_paths() {
        assert_eq!(SoundTheme::Marimba.to_start_path(), "resources/marimba_start.wav");
        assert_eq!(SoundTheme::Marimba.to_stop_path(), "resources/marimba_stop.wav");
        assert_eq!(SoundTheme::Pop.to_start_path(), "resources/pop_start.wav");
        assert_eq!(SoundTheme::Pop.to_stop_path(), "resources/pop_stop.wav");
        assert_eq!(SoundTheme::Custom.to_start_path(), "resources/custom_start.wav");
        assert_eq!(SoundTheme::Custom.to_stop_path(), "resources/custom_stop.wav");
    }

    #[test]
    fn test_app_settings_active_provider_lookup() {
        let settings = get_default_settings();

        let provider = settings.active_post_process_provider();
        assert!(provider.is_some(), "should find the active provider");

        let provider = provider.unwrap();
        assert_eq!(provider.id, "openai");
        assert_eq!(provider.label, "OpenAI");
    }

    #[test]
    fn test_app_settings_provider_by_id() {
        let settings = get_default_settings();

        let openai = settings.post_process_provider("openai");
        assert!(openai.is_some());
        assert_eq!(openai.unwrap().label, "OpenAI");

        let anthropic = settings.post_process_provider("anthropic");
        assert!(anthropic.is_some());
        assert_eq!(anthropic.unwrap().label, "Anthropic");

        let nonexistent = settings.post_process_provider("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_app_settings_provider_mut() {
        let mut settings = get_default_settings();

        if let Some(custom) = settings.post_process_provider_mut("custom") {
            custom.base_url = "http://new-url.local:8080/v1".to_string();
        }

        let custom = settings.post_process_provider("custom").unwrap();
        assert_eq!(custom.base_url, "http://new-url.local:8080/v1");
    }

    #[test]
    fn test_default_post_process_providers_include_expected() {
        let settings = get_default_settings();

        let provider_ids: Vec<&str> = settings
            .post_process_providers
            .iter()
            .map(|p| p.id.as_str())
            .collect();

        assert!(provider_ids.contains(&"openai"), "should include openai");
        assert!(provider_ids.contains(&"anthropic"), "should include anthropic");
        assert!(provider_ids.contains(&"groq"), "should include groq");
        assert!(provider_ids.contains(&"openrouter"), "should include openrouter");
        assert!(provider_ids.contains(&"custom"), "should include custom");
    }

    #[test]
    fn test_custom_provider_allows_base_url_edit() {
        let settings = get_default_settings();

        let custom = settings.post_process_provider("custom");
        assert!(custom.is_some());
        assert!(
            custom.unwrap().allow_base_url_edit,
            "custom provider should allow base URL editing"
        );

        let openai = settings.post_process_provider("openai");
        assert!(openai.is_some());
        assert!(
            !openai.unwrap().allow_base_url_edit,
            "openai provider should not allow base URL editing"
        );
    }

    #[test]
    fn test_default_prompts_structure() {
        let settings = get_default_settings();

        assert!(!settings.post_process_prompts.is_empty());

        let first_prompt = &settings.post_process_prompts[0];
        assert!(!first_prompt.id.is_empty());
        assert!(!first_prompt.name.is_empty());
        assert!(!first_prompt.prompt.is_empty());
        assert!(
            first_prompt.prompt.contains("${output}"),
            "default prompt should contain ${{output}} placeholder"
        );
    }

    #[test]
    fn test_settings_serialization_roundtrip() {
        let settings = get_default_settings();

        // Serialize to JSON
        let json = serde_json::to_string(&settings).expect("should serialize");

        // Deserialize back
        let deserialized: AppSettings =
            serde_json::from_str(&json).expect("should deserialize");

        // Verify key fields match
        assert_eq!(settings.audio_feedback, deserialized.audio_feedback);
        assert_eq!(settings.debug_mode, deserialized.debug_mode);
        assert_eq!(settings.history_limit, deserialized.history_limit);
        assert_eq!(settings.bindings.len(), deserialized.bindings.len());
    }

    #[test]
    fn test_log_level_serialization() {
        // Test string format
        let json = r#""debug""#;
        let level: LogLevel = serde_json::from_str(json).expect("should parse string format");
        assert_eq!(level, LogLevel::Debug);

        // Test numeric format (legacy)
        let json = "3";
        let level: LogLevel = serde_json::from_str(json).expect("should parse numeric format");
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_general_settings_defaults() {
        let settings = get_default_settings();

        assert!(
            settings.general.push_to_talk,
            "push_to_talk should be true by default"
        );
        assert!(
            !settings.general.start_hidden,
            "start_hidden should be false by default"
        );
        assert!(
            !settings.general.autostart_enabled,
            "autostart_enabled should be false by default"
        );
        assert!(
            settings.general.update_checks_enabled,
            "update_checks_enabled should be true by default"
        );
    }

    #[test]
    fn test_active_listening_defaults() {
        let settings = get_default_settings();

        assert!(
            !settings.active_listening.enabled,
            "active_listening should be disabled by default"
        );
        assert!(
            settings.active_listening.segment_duration_seconds > 0,
            "segment_duration_seconds should be positive"
        );
        assert!(
            !settings.active_listening.ollama_base_url.is_empty(),
            "ollama_base_url should have a default"
        );
    }

    #[test]
    fn test_ask_ai_defaults() {
        let settings = get_default_settings();

        assert!(
            settings.ask_ai.enabled,
            "ask_ai should be enabled by default"
        );
    }

    #[test]
    fn test_suggestions_defaults() {
        let settings = get_default_settings();

        assert!(
            !settings.suggestions.enabled,
            "suggestions should be disabled by default"
        );
        assert!(
            !settings.suggestions.quick_responses.is_empty(),
            "quick_responses should have default entries"
        );
        assert!(
            settings.suggestions.rag_suggestions_enabled,
            "rag_suggestions_enabled should be true by default"
        );
        assert!(
            settings.suggestions.llm_suggestions_enabled,
            "llm_suggestions_enabled should be true by default"
        );
        assert_eq!(
            settings.suggestions.max_suggestions, 3,
            "max_suggestions should default to 3"
        );
    }
}
