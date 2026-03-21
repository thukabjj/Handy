use crate::settings::{
    get_settings,
    observability::{ObservabilityDataMode, ObservabilitySettings},
};
use chrono::Utc;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use uuid::Uuid;

static GLOBAL_APP_HANDLE: OnceCell<AppHandle> = OnceCell::new();

const MIN_EVENT_BUFFER: usize = 50;
const MAX_EVENT_BUFFER: usize = 5_000;
const REDACTED_VALUE: &str = "[redacted]";

#[derive(Clone, Debug, Serialize, Deserialize, Type, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryAttribute {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub redacted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryEvent {
    pub id: String,
    pub timestamp_ms: i64,
    pub component: String,
    pub action: String,
    pub level: TelemetryLevel,
    pub message: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: Option<String>,
    pub source: String,
    pub attributes: Vec<TelemetryAttribute>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryComponentStat {
    pub component: String,
    pub total_events: u32,
    pub error_events: u32,
    pub warn_events: u32,
    pub last_event_at_ms: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryStats {
    pub total_events: u32,
    pub buffered_events: u32,
    pub dropped_events: u64,
    pub error_events: u32,
    pub warn_events: u32,
    pub last_event_at_ms: Option<i64>,
    pub live_streaming_enabled: bool,
    pub component_breakdown: Vec<TelemetryComponentStat>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetrySnapshot {
    pub settings: ObservabilitySettings,
    pub stats: TelemetryStats,
    pub events: Vec<TelemetryEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryAttributeInput {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub sensitive: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub struct TelemetryRecordInput {
    pub component: String,
    pub action: String,
    pub level: TelemetryLevel,
    pub message: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub status: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub attributes: Vec<TelemetryAttributeInput>,
}

pub struct TelemetryEventBuilder {
    component: String,
    action: String,
    level: TelemetryLevel,
    message: String,
    trace_id: Option<String>,
    session_id: Option<String>,
    duration_ms: Option<u64>,
    status: Option<String>,
    source: Option<String>,
    attributes: Vec<TelemetryAttributeInput>,
}

#[allow(dead_code)]
impl TelemetryEventBuilder {
    pub fn new(component: impl Into<String>, action: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            action: action.into(),
            level: TelemetryLevel::Info,
            message: String::new(),
            trace_id: None,
            session_id: None,
            duration_ms: None,
            status: None,
            source: None,
            attributes: Vec::new(),
        }
    }

    pub fn level(mut self, level: TelemetryLevel) -> Self {
        self.level = level;
        self
    }

    pub fn trace(mut self) -> Self {
        self.level = TelemetryLevel::Trace;
        self
    }

    pub fn debug(mut self) -> Self {
        self.level = TelemetryLevel::Debug;
        self
    }

    pub fn warn(mut self) -> Self {
        self.level = TelemetryLevel::Warn;
        self
    }

    pub fn error(mut self) -> Self {
        self.level = TelemetryLevel::Error;
        self
    }

    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn trace_id(mut self, trace_id: impl Into<String>) -> Self {
        self.trace_id = Some(trace_id.into());
        self
    }

    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn attr(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.attributes.push(TelemetryAttributeInput {
            key: key.into(),
            value: value.to_string(),
            sensitive: false,
        });
        self
    }

    pub fn sensitive_attr(mut self, key: impl Into<String>, value: impl ToString) -> Self {
        self.attributes.push(TelemetryAttributeInput {
            key: key.into(),
            value: value.to_string(),
            sensitive: true,
        });
        self
    }

    pub fn emit(self) {
        record_input(self.into());
    }
}

impl From<TelemetryEventBuilder> for TelemetryRecordInput {
    fn from(value: TelemetryEventBuilder) -> Self {
        Self {
            component: value.component,
            action: value.action,
            level: value.level,
            message: value.message,
            trace_id: value.trace_id,
            session_id: value.session_id,
            duration_ms: value.duration_ms,
            status: value.status,
            source: value.source,
            attributes: value.attributes,
        }
    }
}

pub struct TelemetryManager {
    app_handle: AppHandle,
    events: Mutex<VecDeque<TelemetryEvent>>,
    dropped_events: AtomicU64,
}

impl TelemetryManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        Self {
            app_handle: app_handle.clone(),
            events: Mutex::new(VecDeque::new()),
            dropped_events: AtomicU64::new(0),
        }
    }

    pub fn record(&self, input: TelemetryRecordInput) {
        let settings = self.settings();
        if !settings.enabled {
            return;
        }

        let event = sanitize_event(input, settings.data_mode);
        let limit = clamp_limit(settings.max_events);

        {
            let mut events = self.events.lock().unwrap();
            while events.len() >= limit {
                events.pop_front();
                self.dropped_events.fetch_add(1, Ordering::Relaxed);
            }
            events.push_back(event.clone());
        }

        emit_to_log(&event);

        if settings.emit_live_events {
            let _ = self.app_handle.emit("telemetry-event", &event);
        }
    }

    pub fn clear(&self) {
        let mut events = self.events.lock().unwrap();
        events.clear();
    }

    pub fn trim_to_limit(&self) {
        let limit = clamp_limit(self.settings().max_events);
        let mut events = self.events.lock().unwrap();
        while events.len() > limit {
            events.pop_front();
            self.dropped_events.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn snapshot(&self) -> TelemetrySnapshot {
        let settings = self.settings();
        let events = self.events.lock().unwrap().iter().cloned().collect::<Vec<_>>();
        let stats = build_stats(&events, &settings, self.dropped_events.load(Ordering::Relaxed));
        TelemetrySnapshot {
            settings,
            stats,
            events,
        }
    }

    pub fn recent_events(&self, limit: Option<u32>) -> Vec<TelemetryEvent> {
        let limit = limit.map(|value| value.clamp(1, MAX_EVENT_BUFFER as u32) as usize);
        let events = self.events.lock().unwrap();
        let iter = events.iter().cloned();
        match limit {
            Some(limit) => iter.rev().take(limit).collect::<Vec<_>>().into_iter().rev().collect(),
            None => iter.collect(),
        }
    }

    fn settings(&self) -> ObservabilitySettings {
        get_settings(&self.app_handle).observability
    }
}

pub fn initialize_global_telemetry(app_handle: &AppHandle) {
    let _ = GLOBAL_APP_HANDLE.set(app_handle.clone());
}

pub fn record_input(input: TelemetryRecordInput) {
    if let Some(manager) = global_manager() {
        manager.record(input);
        return;
    }

    let fallback = sanitize_event(input, ObservabilityDataMode::MetadataOnly);
    emit_to_log(&fallback);
}

fn global_manager() -> Option<Arc<TelemetryManager>> {
    let app = GLOBAL_APP_HANDLE.get()?;
    let state = app.try_state::<Arc<TelemetryManager>>()?;
    Some(state.inner().clone())
}

fn clamp_limit(limit: u32) -> usize {
    limit.clamp(MIN_EVENT_BUFFER as u32, MAX_EVENT_BUFFER as u32) as usize
}

fn sanitize_event(input: TelemetryRecordInput, data_mode: ObservabilityDataMode) -> TelemetryEvent {
    let attributes = input
        .attributes
        .into_iter()
        .map(|attribute| sanitize_attribute(attribute, data_mode))
        .collect::<Vec<_>>();

    TelemetryEvent {
        id: Uuid::new_v4().to_string(),
        timestamp_ms: Utc::now().timestamp_millis(),
        component: input.component,
        action: input.action,
        level: input.level,
        message: input.message,
        trace_id: input.trace_id,
        session_id: input.session_id,
        duration_ms: input.duration_ms,
        status: input.status,
        source: input.source.unwrap_or_else(|| "backend".to_string()),
        attributes,
    }
}

fn sanitize_attribute(
    attribute: TelemetryAttributeInput,
    data_mode: ObservabilityDataMode,
) -> TelemetryAttribute {
    if attribute.sensitive && !data_mode.includes_sensitive_content() {
        return TelemetryAttribute {
            key: attribute.key,
            value: REDACTED_VALUE.to_string(),
            redacted: true,
        };
    }

    TelemetryAttribute {
        key: attribute.key,
        value: attribute.value,
        redacted: false,
    }
}

fn build_stats(
    events: &[TelemetryEvent],
    settings: &ObservabilitySettings,
    dropped_events: u64,
) -> TelemetryStats {
    let mut components = HashMap::<String, TelemetryComponentStat>::new();
    let mut error_events = 0u32;
    let mut warn_events = 0u32;
    let mut last_event_at_ms = None;

    for event in events {
        let stat = components
            .entry(event.component.clone())
            .or_insert(TelemetryComponentStat {
                component: event.component.clone(),
                total_events: 0,
                error_events: 0,
                warn_events: 0,
                last_event_at_ms: None,
            });

        stat.total_events += 1;
        stat.last_event_at_ms = Some(event.timestamp_ms);

        match event.level {
            TelemetryLevel::Error => {
                stat.error_events += 1;
                error_events += 1;
            }
            TelemetryLevel::Warn => {
                stat.warn_events += 1;
                warn_events += 1;
            }
            _ => {}
        }

        last_event_at_ms = Some(event.timestamp_ms);
    }

    let mut component_breakdown = components.into_values().collect::<Vec<_>>();
    component_breakdown.sort_by(|left, right| {
        right
            .total_events
            .cmp(&left.total_events)
            .then_with(|| left.component.cmp(&right.component))
    });

    TelemetryStats {
        total_events: events.len() as u32 + dropped_events as u32,
        buffered_events: events.len() as u32,
        dropped_events,
        error_events,
        warn_events,
        last_event_at_ms,
        live_streaming_enabled: settings.emit_live_events,
        component_breakdown,
    }
}

fn emit_to_log(event: &TelemetryEvent) {
    let payload = serde_json::to_string(event).unwrap_or_else(|_| {
        format!(
            "{{\"component\":\"{}\",\"action\":\"{}\",\"message\":\"{}\"}}",
            event.component, event.action, event.message
        )
    });

    match event.level {
        TelemetryLevel::Trace => log::trace!(target: "observability", "{}", payload),
        TelemetryLevel::Debug => log::debug!(target: "observability", "{}", payload),
        TelemetryLevel::Info => log::info!(target: "observability", "{}", payload),
        TelemetryLevel::Warn => log::warn!(target: "observability", "{}", payload),
        TelemetryLevel::Error => log::error!(target: "observability", "{}", payload),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_mode_redacts_sensitive_attributes() {
        let event = sanitize_event(
            TelemetryRecordInput {
                component: "wake_word".to_string(),
                action: "transcription".to_string(),
                level: TelemetryLevel::Info,
                message: "wake transcription processed".to_string(),
                trace_id: None,
                session_id: None,
                duration_ms: None,
                status: None,
                source: None,
                attributes: vec![TelemetryAttributeInput {
                    key: "transcript".to_string(),
                    value: "hey handy what time is it".to_string(),
                    sensitive: true,
                }],
            },
            ObservabilityDataMode::MetadataOnly,
        );

        assert_eq!(event.attributes.len(), 1);
        assert_eq!(event.attributes[0].value, REDACTED_VALUE);
        assert!(event.attributes[0].redacted);
    }

    #[test]
    fn sensitive_mode_keeps_sensitive_attributes() {
        let event = sanitize_event(
            TelemetryRecordInput {
                component: "ask_ai".to_string(),
                action: "question".to_string(),
                level: TelemetryLevel::Info,
                message: "question accepted".to_string(),
                trace_id: None,
                session_id: None,
                duration_ms: None,
                status: None,
                source: Some("frontend".to_string()),
                attributes: vec![TelemetryAttributeInput {
                    key: "text".to_string(),
                    value: "summarize this meeting".to_string(),
                    sensitive: true,
                }],
            },
            ObservabilityDataMode::SensitiveText,
        );

        assert_eq!(event.attributes[0].value, "summarize this meeting");
        assert!(!event.attributes[0].redacted);
        assert_eq!(event.source, "frontend");
    }
}
