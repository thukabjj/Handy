use serde::{Deserialize, Serialize};
use specta::Type;

/// Audio source type for Active Listening
/// Determines where audio is captured from for transcription
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioSourceType {
    /// Microphone input only (current default behavior)
    #[default]
    Microphone,
    /// System audio output only (loopback capture)
    SystemAudio,
    /// Both microphone and system audio mixed together
    Mixed,
}

/// Settings for audio source mixing when using Mixed mode
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct AudioMixSettings {
    /// Mix ratio: 0.0 = microphone only, 1.0 = system audio only, 0.5 = equal mix
    #[serde(default = "default_mix_ratio")]
    pub mix_ratio: f32,
}

fn default_mix_ratio() -> f32 {
    0.5 // Equal mix by default
}

impl Default for AudioMixSettings {
    fn default() -> Self {
        Self {
            mix_ratio: default_mix_ratio(),
        }
    }
}

/// Settings for the Active Listening feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ActiveListeningSettings {
    /// Whether active listening is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Duration of each audio segment in seconds before transcription
    #[serde(default = "default_segment_duration_seconds")]
    pub segment_duration_seconds: u32,

    /// Ollama server base URL
    #[serde(default = "default_ollama_base_url")]
    pub ollama_base_url: String,

    /// Ollama model to use for generating insights
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,

    /// Custom prompts for active listening
    #[serde(default = "default_prompts")]
    pub prompts: Vec<ActiveListeningPrompt>,

    /// Currently selected prompt ID
    #[serde(default)]
    pub selected_prompt_id: Option<String>,

    /// Number of previous summaries to keep for context
    #[serde(default = "default_context_window_size")]
    pub context_window_size: usize,

    /// Audio source type for capturing audio
    #[serde(default)]
    pub audio_source_type: AudioSourceType,

    /// Settings for audio mixing when using Mixed mode
    #[serde(default)]
    pub audio_mix_settings: AudioMixSettings,
}

/// Category for grouping prompts
#[derive(Serialize, Deserialize, Debug, Clone, Type, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    /// Note-taking and summarization prompts
    #[default]
    NoteTaking,
    /// Real-time meeting coach prompts (Perssua-like)
    MeetingCoach,
    /// User-created custom prompts
    Custom,
}

/// A prompt template for active listening
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct ActiveListeningPrompt {
    /// Unique identifier for the prompt
    pub id: String,

    /// Display name for the prompt
    pub name: String,

    /// Prompt template supporting {{transcription}}, {{previous_context}}, {{session_topic}}
    pub prompt_template: String,

    /// When this prompt was created (Unix timestamp in milliseconds)
    pub created_at: i64,

    /// Whether this is a built-in default prompt
    #[serde(default)]
    pub is_default: bool,

    /// Category for grouping prompts in the UI
    #[serde(default)]
    pub category: PromptCategory,
}

// Default value functions
fn default_enabled() -> bool {
    false
}

fn default_segment_duration_seconds() -> u32 {
    15
}

fn default_ollama_base_url() -> String {
    "http://localhost:11434".to_string()
}

fn default_ollama_model() -> String {
    String::new()
}

fn default_context_window_size() -> usize {
    3
}

fn default_prompts() -> Vec<ActiveListeningPrompt> {
    vec![
        // === Note-Taking Prompts ===
        ActiveListeningPrompt {
            id: "default_meeting_notes".to_string(),
            name: "Meeting Notes".to_string(),
            prompt_template: r#"Analyze this conversation segment and extract key information:

Transcription: {{transcription}}

Previous context: {{previous_context}}

Session topic: {{session_topic}}

Provide a brief summary including:
1. Key topics discussed
2. Any action items or decisions mentioned
3. Important points to remember

Be concise and focus on the most relevant information."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::NoteTaking,
        },
        ActiveListeningPrompt {
            id: "default_action_items".to_string(),
            name: "Action Items".to_string(),
            prompt_template:
                r#"Review this conversation segment and identify any action items or tasks:

Transcription: {{transcription}}

Previous context: {{previous_context}}

Session topic: {{session_topic}}

List any action items mentioned, including:
- Who is responsible (if mentioned)
- What needs to be done
- Any deadlines mentioned

If no action items are found, briefly note the main topic discussed."#
                    .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::NoteTaking,
        },
        // === Meeting Coach Prompts ===
        ActiveListeningPrompt {
            id: "meeting_coach_objection_handler".to_string(),
            name: "Objection Handler".to_string(),
            prompt_template: r#"You are a real-time meeting coach. Analyze this conversation segment:

Transcription: {{transcription}}
Previous context: {{previous_context}}
Topic: {{session_topic}}

If you detect an objection, concern, or difficult question:
1. Identify what the objection is
2. Provide 2-3 persuasive counter-arguments
3. Suggest a diplomatic response

If no objection detected, provide a brief insight about the conversation flow.

Be concise - this is real-time assistance. Keep response under 100 words."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        },
        ActiveListeningPrompt {
            id: "meeting_coach_sales".to_string(),
            name: "Sales Coach".to_string(),
            prompt_template: r#"You are a sales coach providing real-time guidance.

Current segment: {{transcription}}
Context: {{previous_context}}
Deal/Product: {{session_topic}}

Analyze and provide:
- Key buying signals detected
- Suggested next talking point
- Warning if conversation going off-track
- Confidence level assessment

Keep response under 100 words for quick reading."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        },
        ActiveListeningPrompt {
            id: "meeting_coach_interview".to_string(),
            name: "Interview Assistant".to_string(),
            prompt_template: r#"You are an interview assistant helping evaluate candidates.

Segment: {{transcription}}
Previous: {{previous_context}}
Role/Position: {{session_topic}}

Provide:
- Follow-up question suggestions based on their response
- Points to probe deeper
- Red/green flags detected in answers
- Brief assessment notes

Be concise for real-time use. Keep response under 100 words."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        },
        ActiveListeningPrompt {
            id: "meeting_coach_negotiation".to_string(),
            name: "Negotiation Coach".to_string(),
            prompt_template: r#"You are a negotiation coach providing real-time tactical advice.

Current segment: {{transcription}}
Context: {{previous_context}}
Negotiation subject: {{session_topic}}

Analyze and provide:
- Their position and potential interests
- Leverage points you can use
- Suggested response or counter-offer approach
- Warning signs or manipulation tactics detected

Keep response under 100 words for quick reading."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        },
        ActiveListeningPrompt {
            id: "meeting_coach_presentation".to_string(),
            name: "Presentation Coach".to_string(),
            prompt_template: r#"You are a presentation coach providing real-time feedback.

Current segment: {{transcription}}
Context: {{previous_context}}
Presentation topic: {{session_topic}}

Provide:
- Audience engagement assessment
- Suggested transitions or emphasis points
- Questions to anticipate
- Pacing and clarity feedback

Keep response under 100 words for quick reading."#
                .to_string(),
            created_at: 0,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        },
    ]
}

impl Default for ActiveListeningSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            segment_duration_seconds: default_segment_duration_seconds(),
            ollama_base_url: default_ollama_base_url(),
            ollama_model: default_ollama_model(),
            prompts: default_prompts(),
            selected_prompt_id: Some("default_meeting_notes".to_string()),
            context_window_size: default_context_window_size(),
            audio_source_type: AudioSourceType::default(),
            audio_mix_settings: AudioMixSettings::default(),
        }
    }
}

impl ActiveListeningSettings {
    /// Get the currently selected prompt
    pub fn get_selected_prompt(&self) -> Option<&ActiveListeningPrompt> {
        self.selected_prompt_id
            .as_ref()
            .and_then(|id| self.prompts.iter().find(|p| &p.id == id))
    }

    /// Get a prompt by ID
    pub fn get_prompt(&self, id: &str) -> Option<&ActiveListeningPrompt> {
        self.prompts.iter().find(|p| p.id == id)
    }

    /// Get a mutable prompt by ID
    pub fn get_prompt_mut(&mut self, id: &str) -> Option<&mut ActiveListeningPrompt> {
        self.prompts.iter_mut().find(|p| p.id == id)
    }
}

/// Ensure default prompts exist in settings (for migrations)
pub fn ensure_active_listening_defaults(settings: &mut ActiveListeningSettings) -> bool {
    let mut changed = false;
    let defaults = default_prompts();

    for default_prompt in defaults {
        if !settings.prompts.iter().any(|p| p.id == default_prompt.id) {
            settings.prompts.push(default_prompt);
            changed = true;
        }
    }

    // Ensure a prompt is selected if none is
    if settings.selected_prompt_id.is_none() && !settings.prompts.is_empty() {
        settings.selected_prompt_id = Some(settings.prompts[0].id.clone());
        changed = true;
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = ActiveListeningSettings::default();

        assert!(!settings.enabled);
        assert_eq!(settings.segment_duration_seconds, 15);
        assert_eq!(settings.ollama_base_url, "http://localhost:11434");
        assert!(settings.ollama_model.is_empty());
        assert_eq!(settings.context_window_size, 3);
        assert_eq!(settings.prompts.len(), 7); // 2 note-taking + 5 meeting coach
        assert_eq!(
            settings.selected_prompt_id,
            Some("default_meeting_notes".to_string())
        );
    }

    #[test]
    fn test_default_prompts() {
        let prompts = default_prompts();

        assert_eq!(prompts.len(), 7);

        // Check note-taking prompts
        let meeting_notes = &prompts[0];
        assert_eq!(meeting_notes.id, "default_meeting_notes");
        assert_eq!(meeting_notes.name, "Meeting Notes");
        assert!(meeting_notes.prompt_template.contains("{{transcription}}"));
        assert!(meeting_notes.prompt_template.contains("{{previous_context}}"));
        assert!(meeting_notes.prompt_template.contains("{{session_topic}}"));
        assert!(meeting_notes.is_default);
        assert_eq!(meeting_notes.category, PromptCategory::NoteTaking);

        let action_items = &prompts[1];
        assert_eq!(action_items.id, "default_action_items");
        assert_eq!(action_items.name, "Action Items");
        assert!(action_items.is_default);
        assert_eq!(action_items.category, PromptCategory::NoteTaking);

        // Check meeting coach prompts
        let objection_handler = &prompts[2];
        assert_eq!(objection_handler.id, "meeting_coach_objection_handler");
        assert_eq!(objection_handler.name, "Objection Handler");
        assert!(objection_handler.is_default);
        assert_eq!(objection_handler.category, PromptCategory::MeetingCoach);

        let sales_coach = &prompts[3];
        assert_eq!(sales_coach.id, "meeting_coach_sales");
        assert_eq!(sales_coach.name, "Sales Coach");
        assert!(sales_coach.is_default);
        assert_eq!(sales_coach.category, PromptCategory::MeetingCoach);

        let interview_assistant = &prompts[4];
        assert_eq!(interview_assistant.id, "meeting_coach_interview");
        assert_eq!(interview_assistant.name, "Interview Assistant");
        assert!(interview_assistant.is_default);
        assert_eq!(interview_assistant.category, PromptCategory::MeetingCoach);

        let negotiation_coach = &prompts[5];
        assert_eq!(negotiation_coach.id, "meeting_coach_negotiation");
        assert_eq!(negotiation_coach.name, "Negotiation Coach");
        assert!(negotiation_coach.is_default);
        assert_eq!(negotiation_coach.category, PromptCategory::MeetingCoach);

        let presentation_coach = &prompts[6];
        assert_eq!(presentation_coach.id, "meeting_coach_presentation");
        assert_eq!(presentation_coach.name, "Presentation Coach");
        assert!(presentation_coach.is_default);
        assert_eq!(presentation_coach.category, PromptCategory::MeetingCoach);
    }

    #[test]
    fn test_prompt_category_default() {
        let category = PromptCategory::default();
        assert_eq!(category, PromptCategory::NoteTaking);
    }

    #[test]
    fn test_get_selected_prompt() {
        let settings = ActiveListeningSettings::default();
        let selected = settings.get_selected_prompt();

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "default_meeting_notes");
    }

    #[test]
    fn test_get_selected_prompt_none() {
        let mut settings = ActiveListeningSettings::default();
        settings.selected_prompt_id = None;

        assert!(settings.get_selected_prompt().is_none());
    }

    #[test]
    fn test_get_selected_prompt_invalid_id() {
        let mut settings = ActiveListeningSettings::default();
        settings.selected_prompt_id = Some("nonexistent".to_string());

        assert!(settings.get_selected_prompt().is_none());
    }

    #[test]
    fn test_get_prompt() {
        let settings = ActiveListeningSettings::default();

        let meeting = settings.get_prompt("default_meeting_notes");
        assert!(meeting.is_some());
        assert_eq!(meeting.unwrap().name, "Meeting Notes");

        let action = settings.get_prompt("default_action_items");
        assert!(action.is_some());
        assert_eq!(action.unwrap().name, "Action Items");

        let nonexistent = settings.get_prompt("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_get_prompt_mut() {
        let mut settings = ActiveListeningSettings::default();

        {
            let prompt = settings.get_prompt_mut("default_meeting_notes");
            assert!(prompt.is_some());
            prompt.unwrap().name = "Modified Name".to_string();
        }

        assert_eq!(
            settings.get_prompt("default_meeting_notes").unwrap().name,
            "Modified Name"
        );
    }

    #[test]
    fn test_ensure_defaults_no_change_when_complete() {
        let mut settings = ActiveListeningSettings::default();
        let changed = ensure_active_listening_defaults(&mut settings);

        assert!(!changed);
        assert_eq!(settings.prompts.len(), 7); // 2 note-taking + 5 meeting coach
    }

    #[test]
    fn test_ensure_defaults_adds_missing_prompts() {
        let mut settings = ActiveListeningSettings::default();
        settings.prompts.clear();
        settings.selected_prompt_id = Some("custom".to_string());

        let changed = ensure_active_listening_defaults(&mut settings);

        assert!(changed);
        assert_eq!(settings.prompts.len(), 7); // All default prompts added
    }

    #[test]
    fn test_ensure_defaults_sets_selected_when_none() {
        let mut settings = ActiveListeningSettings::default();
        settings.selected_prompt_id = None;

        let changed = ensure_active_listening_defaults(&mut settings);

        assert!(changed);
        assert!(settings.selected_prompt_id.is_some());
    }

    #[test]
    fn test_ensure_defaults_with_custom_prompts() {
        let mut settings = ActiveListeningSettings::default();
        settings.prompts = vec![ActiveListeningPrompt {
            id: "custom_prompt".to_string(),
            name: "Custom".to_string(),
            prompt_template: "Test template".to_string(),
            created_at: 12345,
            is_default: false,
            category: PromptCategory::Custom,
        }];
        settings.selected_prompt_id = Some("custom_prompt".to_string());

        let changed = ensure_active_listening_defaults(&mut settings);

        assert!(changed);
        assert_eq!(settings.prompts.len(), 8); // 1 custom + 7 defaults
        assert_eq!(settings.selected_prompt_id, Some("custom_prompt".to_string()));
    }

    #[test]
    fn test_active_listening_prompt_fields() {
        let prompt = ActiveListeningPrompt {
            id: "test_id".to_string(),
            name: "Test Prompt".to_string(),
            prompt_template: "Hello {{transcription}}".to_string(),
            created_at: 1234567890,
            is_default: false,
            category: PromptCategory::Custom,
        };

        assert_eq!(prompt.id, "test_id");
        assert_eq!(prompt.name, "Test Prompt");
        assert_eq!(prompt.prompt_template, "Hello {{transcription}}");
        assert_eq!(prompt.created_at, 1234567890);
        assert!(!prompt.is_default);
        assert_eq!(prompt.category, PromptCategory::Custom);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = ActiveListeningSettings::default();
        let json = serde_json::to_string(&settings).unwrap();

        assert!(json.contains("\"enabled\":false"));
        assert!(json.contains("\"segment_duration_seconds\":15"));
        assert!(json.contains("\"ollama_base_url\":\"http://localhost:11434\""));
    }

    #[test]
    fn test_settings_deserialization_with_defaults() {
        let json = r#"{"enabled": true}"#;
        let settings: ActiveListeningSettings = serde_json::from_str(json).unwrap();

        assert!(settings.enabled);
        assert_eq!(settings.segment_duration_seconds, 15); // default
        assert_eq!(settings.ollama_base_url, "http://localhost:11434"); // default
        assert_eq!(settings.prompts.len(), 7); // defaults
    }

    #[test]
    fn test_settings_clone() {
        let settings = ActiveListeningSettings::default();
        let cloned = settings.clone();

        assert_eq!(settings.enabled, cloned.enabled);
        assert_eq!(settings.segment_duration_seconds, cloned.segment_duration_seconds);
        assert_eq!(settings.prompts.len(), cloned.prompts.len());
    }

    #[test]
    fn test_prompt_clone() {
        let prompt = ActiveListeningPrompt {
            id: "test".to_string(),
            name: "Test".to_string(),
            prompt_template: "Template".to_string(),
            created_at: 100,
            is_default: true,
            category: PromptCategory::MeetingCoach,
        };
        let cloned = prompt.clone();

        assert_eq!(prompt.id, cloned.id);
        assert_eq!(prompt.name, cloned.name);
        assert_eq!(prompt.is_default, cloned.is_default);
        assert_eq!(prompt.category, cloned.category);
    }

    // Tests for AudioSourceType
    #[test]
    fn test_audio_source_type_default() {
        let source = AudioSourceType::default();
        assert_eq!(source, AudioSourceType::Microphone);
    }

    #[test]
    fn test_audio_source_type_variants() {
        let mic = AudioSourceType::Microphone;
        let system = AudioSourceType::SystemAudio;
        let mixed = AudioSourceType::Mixed;

        assert_ne!(mic, system);
        assert_ne!(mic, mixed);
        assert_ne!(system, mixed);
    }

    #[test]
    fn test_audio_source_type_serialization() {
        let mic = AudioSourceType::Microphone;
        let json = serde_json::to_string(&mic).unwrap();
        assert_eq!(json, "\"microphone\"");

        let system = AudioSourceType::SystemAudio;
        let json = serde_json::to_string(&system).unwrap();
        assert_eq!(json, "\"system_audio\"");

        let mixed = AudioSourceType::Mixed;
        let json = serde_json::to_string(&mixed).unwrap();
        assert_eq!(json, "\"mixed\"");
    }

    #[test]
    fn test_audio_source_type_deserialization() {
        let mic: AudioSourceType = serde_json::from_str("\"microphone\"").unwrap();
        assert_eq!(mic, AudioSourceType::Microphone);

        let system: AudioSourceType = serde_json::from_str("\"system_audio\"").unwrap();
        assert_eq!(system, AudioSourceType::SystemAudio);

        let mixed: AudioSourceType = serde_json::from_str("\"mixed\"").unwrap();
        assert_eq!(mixed, AudioSourceType::Mixed);
    }

    #[test]
    fn test_audio_source_type_clone_and_copy() {
        let source = AudioSourceType::SystemAudio;
        let cloned = source.clone();
        let copied = source; // Copy trait

        assert_eq!(source, cloned);
        assert_eq!(source, copied);
    }

    // Tests for AudioMixSettings
    #[test]
    fn test_audio_mix_settings_default() {
        let settings = AudioMixSettings::default();
        assert_eq!(settings.mix_ratio, 0.5);
    }

    #[test]
    fn test_audio_mix_settings_serialization() {
        let settings = AudioMixSettings { mix_ratio: 0.7 };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"mix_ratio\":0.7"));
    }

    #[test]
    fn test_audio_mix_settings_deserialization() {
        let json = r#"{"mix_ratio": 0.3}"#;
        let settings: AudioMixSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.mix_ratio, 0.3);
    }

    #[test]
    fn test_audio_mix_settings_deserialization_with_default() {
        let json = r#"{}"#;
        let settings: AudioMixSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.mix_ratio, 0.5); // default
    }

    // Tests for ActiveListeningSettings with audio source
    #[test]
    fn test_default_settings_includes_audio_source() {
        let settings = ActiveListeningSettings::default();
        assert_eq!(settings.audio_source_type, AudioSourceType::Microphone);
        assert_eq!(settings.audio_mix_settings.mix_ratio, 0.5);
    }

    #[test]
    fn test_settings_deserialization_with_audio_source() {
        let json = r#"{"enabled": true, "audio_source_type": "system_audio"}"#;
        let settings: ActiveListeningSettings = serde_json::from_str(json).unwrap();

        assert!(settings.enabled);
        assert_eq!(settings.audio_source_type, AudioSourceType::SystemAudio);
        assert_eq!(settings.audio_mix_settings.mix_ratio, 0.5); // default
    }

    #[test]
    fn test_settings_serialization_with_audio_source() {
        let mut settings = ActiveListeningSettings::default();
        settings.audio_source_type = AudioSourceType::Mixed;
        settings.audio_mix_settings.mix_ratio = 0.3;

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"audio_source_type\":\"mixed\""));
        assert!(json.contains("\"mix_ratio\":0.3"));
    }
}
