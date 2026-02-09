//! Settings for the Real-Time Suggestions feature

use serde::{Deserialize, Serialize};
use specta::Type;

/// Severity level for warning suggestions
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Type, Default)]
#[serde(rename_all = "snake_case")]
pub enum WarningSeverity {
    #[default]
    Low,
    Medium,
    High,
}

/// A quick response template that can be triggered by keywords
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct QuickResponse {
    /// Unique identifier for the quick response
    pub id: String,
    /// Display name for the quick response
    pub name: String,
    /// Keywords that trigger this response (comma-separated phrases)
    pub trigger_phrases: Vec<String>,
    /// Category for grouping (e.g., "pricing", "objection", "closing")
    pub category: String,
    /// The response template to suggest
    pub response_template: String,
    /// Whether this quick response is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// When this was created (Unix timestamp in milliseconds)
    pub created_at: i64,
}

fn default_true() -> bool {
    true
}

/// Settings for the Suggestions feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SuggestionsSettings {
    /// Whether suggestions are enabled
    #[serde(default)]
    pub enabled: bool,

    /// Quick response templates
    #[serde(default = "default_quick_responses")]
    pub quick_responses: Vec<QuickResponse>,

    /// Whether to use RAG for context-aware suggestions
    #[serde(default = "default_true")]
    pub rag_suggestions_enabled: bool,

    /// Whether to use LLM for generating dynamic suggestions
    #[serde(default = "default_true")]
    pub llm_suggestions_enabled: bool,

    /// Maximum number of suggestions to show at once
    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: usize,

    /// Minimum confidence score for showing suggestions (0.0 - 1.0)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,

    /// Whether to auto-dismiss suggestions after copying
    #[serde(default = "default_true")]
    pub auto_dismiss_on_copy: bool,

    /// Suggestion display duration in seconds (0 = until dismissed)
    #[serde(default = "default_display_duration")]
    pub display_duration_seconds: u32,
}

fn default_max_suggestions() -> usize {
    3
}

fn default_min_confidence() -> f32 {
    0.5
}

fn default_display_duration() -> u32 {
    0 // Until dismissed
}

fn default_quick_responses() -> Vec<QuickResponse> {
    vec![
        // Pricing objections
        QuickResponse {
            id: "qr_too_expensive".to_string(),
            name: "Too Expensive".to_string(),
            trigger_phrases: vec![
                "too expensive".to_string(),
                "costs too much".to_string(),
                "over budget".to_string(),
                "can't afford".to_string(),
                "price is high".to_string(),
            ],
            category: "pricing".to_string(),
            response_template: "I understand budget is a concern. Let me share how our solution delivers ROI that typically exceeds the investment within [X] months. Would it help if I walked you through a cost-benefit analysis?".to_string(),
            enabled: true,
            created_at: 0,
        },
        QuickResponse {
            id: "qr_competitor_cheaper".to_string(),
            name: "Competitor is Cheaper".to_string(),
            trigger_phrases: vec![
                "competitor is cheaper".to_string(),
                "found it cheaper".to_string(),
                "lower price elsewhere".to_string(),
                "better deal".to_string(),
            ],
            category: "pricing".to_string(),
            response_template: "That's a fair point. While price is important, let me highlight what's included in our offering that provides additional value: [key differentiators]. Would you like to compare feature-by-feature?".to_string(),
            enabled: true,
            created_at: 0,
        },
        // Timing objections
        QuickResponse {
            id: "qr_not_right_time".to_string(),
            name: "Not the Right Time".to_string(),
            trigger_phrases: vec![
                "not the right time".to_string(),
                "bad timing".to_string(),
                "maybe later".to_string(),
                "need to wait".to_string(),
                "not ready yet".to_string(),
            ],
            category: "timing".to_string(),
            response_template: "I appreciate your honesty. What would need to change for this to become a priority? Understanding your timeline helps me provide relevant information when you're ready.".to_string(),
            enabled: true,
            created_at: 0,
        },
        // Trust objections
        QuickResponse {
            id: "qr_need_to_think".to_string(),
            name: "Need to Think About It".to_string(),
            trigger_phrases: vec![
                "need to think about it".to_string(),
                "let me think".to_string(),
                "have to consider".to_string(),
                "sleep on it".to_string(),
            ],
            category: "trust".to_string(),
            response_template: "Absolutely, this is an important decision. What specific aspects would you like to think through? I'm happy to provide additional information that might help with your evaluation.".to_string(),
            enabled: true,
            created_at: 0,
        },
        // Authority objections
        QuickResponse {
            id: "qr_check_with_team".to_string(),
            name: "Need to Check with Team".to_string(),
            trigger_phrases: vec![
                "check with my team".to_string(),
                "need approval".to_string(),
                "talk to my boss".to_string(),
                "discuss internally".to_string(),
                "get buy-in".to_string(),
            ],
            category: "authority".to_string(),
            response_template: "That makes sense - getting team alignment is crucial. Would it help if I prepared a summary document you could share? I could also join a follow-up call with the key stakeholders.".to_string(),
            enabled: true,
            created_at: 0,
        },
        // Interview responses
        QuickResponse {
            id: "qr_weakness".to_string(),
            name: "Weakness Question".to_string(),
            trigger_phrases: vec![
                "what is your weakness".to_string(),
                "biggest weakness".to_string(),
                "area to improve".to_string(),
                "development area".to_string(),
            ],
            category: "interview".to_string(),
            response_template: "A genuine area I've been working on is [specific skill]. I've addressed this by [concrete steps taken], and I've seen improvement in [measurable outcome].".to_string(),
            enabled: true,
            created_at: 0,
        },
        // Negotiation
        QuickResponse {
            id: "qr_final_offer".to_string(),
            name: "Is This Your Final Offer".to_string(),
            trigger_phrases: vec![
                "final offer".to_string(),
                "best price".to_string(),
                "lowest you can go".to_string(),
                "any flexibility".to_string(),
            ],
            category: "negotiation".to_string(),
            response_template: "I appreciate you asking directly. Before discussing pricing flexibility, I want to make sure I understand all your requirements. Are there any additional needs we haven't covered that could affect the scope?".to_string(),
            enabled: true,
            created_at: 0,
        },
    ]
}

impl Default for SuggestionsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            quick_responses: default_quick_responses(),
            rag_suggestions_enabled: true,
            llm_suggestions_enabled: true,
            max_suggestions: default_max_suggestions(),
            min_confidence: default_min_confidence(),
            auto_dismiss_on_copy: true,
            display_duration_seconds: default_display_duration(),
        }
    }
}

impl SuggestionsSettings {
    /// Get a quick response by ID
    pub fn get_quick_response(&self, id: &str) -> Option<&QuickResponse> {
        self.quick_responses.iter().find(|qr| qr.id == id)
    }

    /// Get a mutable quick response by ID
    pub fn get_quick_response_mut(&mut self, id: &str) -> Option<&mut QuickResponse> {
        self.quick_responses.iter_mut().find(|qr| qr.id == id)
    }

    /// Get all enabled quick responses
    pub fn get_enabled_quick_responses(&self) -> Vec<&QuickResponse> {
        self.quick_responses.iter().filter(|qr| qr.enabled).collect()
    }

    /// Get quick responses by category
    pub fn get_quick_responses_by_category(&self, category: &str) -> Vec<&QuickResponse> {
        self.quick_responses
            .iter()
            .filter(|qr| qr.category == category && qr.enabled)
            .collect()
    }
}

/// Ensure default quick responses exist in settings (for migrations)
pub fn ensure_suggestions_defaults(settings: &mut SuggestionsSettings) -> bool {
    let mut changed = false;
    let defaults = default_quick_responses();

    for default_qr in defaults {
        if !settings.quick_responses.iter().any(|qr| qr.id == default_qr.id) {
            settings.quick_responses.push(default_qr);
            changed = true;
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = SuggestionsSettings::default();

        assert!(!settings.enabled);
        assert!(settings.rag_suggestions_enabled);
        assert!(settings.llm_suggestions_enabled);
        assert_eq!(settings.max_suggestions, 3);
        assert_eq!(settings.min_confidence, 0.5);
        assert!(settings.auto_dismiss_on_copy);
        assert_eq!(settings.display_duration_seconds, 0);
        assert!(!settings.quick_responses.is_empty());
    }

    #[test]
    fn test_default_quick_responses() {
        let responses = default_quick_responses();

        assert!(!responses.is_empty());

        // Check pricing objection exists
        let pricing = responses.iter().find(|qr| qr.id == "qr_too_expensive");
        assert!(pricing.is_some());
        assert!(pricing.unwrap().enabled);
        assert!(!pricing.unwrap().trigger_phrases.is_empty());

        // Check interview response exists
        let interview = responses.iter().find(|qr| qr.id == "qr_weakness");
        assert!(interview.is_some());
        assert_eq!(interview.unwrap().category, "interview");
    }

    #[test]
    fn test_get_quick_response() {
        let settings = SuggestionsSettings::default();

        let qr = settings.get_quick_response("qr_too_expensive");
        assert!(qr.is_some());
        assert_eq!(qr.unwrap().name, "Too Expensive");

        let nonexistent = settings.get_quick_response("nonexistent");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_get_enabled_quick_responses() {
        let mut settings = SuggestionsSettings::default();

        // Disable one
        if let Some(qr) = settings.get_quick_response_mut("qr_too_expensive") {
            qr.enabled = false;
        }

        let enabled = settings.get_enabled_quick_responses();
        assert!(!enabled.iter().any(|qr| qr.id == "qr_too_expensive"));
    }

    #[test]
    fn test_get_quick_responses_by_category() {
        let settings = SuggestionsSettings::default();

        let pricing = settings.get_quick_responses_by_category("pricing");
        assert!(!pricing.is_empty());
        assert!(pricing.iter().all(|qr| qr.category == "pricing"));

        let interview = settings.get_quick_responses_by_category("interview");
        assert!(!interview.is_empty());
    }

    #[test]
    fn test_ensure_defaults() {
        let mut settings = SuggestionsSettings::default();
        settings.quick_responses.clear();

        let changed = ensure_suggestions_defaults(&mut settings);

        assert!(changed);
        assert!(!settings.quick_responses.is_empty());
    }

    #[test]
    fn test_ensure_defaults_no_change() {
        let mut settings = SuggestionsSettings::default();

        let changed = ensure_suggestions_defaults(&mut settings);

        assert!(!changed);
    }

    #[test]
    fn test_quick_response_clone() {
        let qr = QuickResponse {
            id: "test".to_string(),
            name: "Test".to_string(),
            trigger_phrases: vec!["trigger".to_string()],
            category: "test".to_string(),
            response_template: "Response".to_string(),
            enabled: true,
            created_at: 12345,
        };

        let cloned = qr.clone();
        assert_eq!(qr.id, cloned.id);
        assert_eq!(qr.trigger_phrases, cloned.trigger_phrases);
    }

    #[test]
    fn test_warning_severity_default() {
        let severity = WarningSeverity::default();
        assert_eq!(severity, WarningSeverity::Low);
    }
}
