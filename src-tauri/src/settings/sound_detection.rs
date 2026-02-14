//! Sound Detection Settings
//!
//! Settings for the environmental sound detection feature.

use serde::{Deserialize, Serialize};
use specta::Type;

/// Categories of environmental sounds that can be detected
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Type)]
#[serde(rename_all = "snake_case")]
pub enum SoundCategory {
    Doorbell,
    Alarm,
    PhoneRing,
    DogBark,
    BabyCry,
    Knocking,
    Siren,
    Applause,
}

/// Settings for the Environmental Sound Detection feature
#[derive(Serialize, Deserialize, Debug, Clone, Type)]
pub struct SoundDetectionSettings {
    /// Whether sound detection is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Which sound categories to detect
    #[serde(default = "default_categories")]
    pub categories: Vec<SoundCategory>,

    /// Minimum confidence threshold for triggering a detection (0.0-1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f32,

    /// Whether to show system notifications on detection
    #[serde(default = "default_notification_enabled")]
    pub notification_enabled: bool,
}

fn default_enabled() -> bool {
    false
}

fn default_categories() -> Vec<SoundCategory> {
    vec![
        SoundCategory::Doorbell,
        SoundCategory::Alarm,
        SoundCategory::PhoneRing,
        SoundCategory::DogBark,
        SoundCategory::BabyCry,
        SoundCategory::Knocking,
        SoundCategory::Siren,
        SoundCategory::Applause,
    ]
}

fn default_threshold() -> f32 {
    0.5
}

fn default_notification_enabled() -> bool {
    true
}

impl Default for SoundDetectionSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            categories: default_categories(),
            threshold: default_threshold(),
            notification_enabled: default_notification_enabled(),
        }
    }
}
