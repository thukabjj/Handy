use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::fmt;

use crate::managers::history::HistoryEntry;

/// Supported export formats for transcriptions
#[derive(Clone, Debug, Serialize, Deserialize, Type)]
pub enum ExportFormat {
    #[serde(rename = "txt")]
    Txt,
    #[serde(rename = "srt")]
    Srt,
    #[serde(rename = "vtt")]
    Vtt,
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "markdown")]
    Markdown,
}

impl fmt::Display for ExportFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExportFormat::Txt => write!(f, "txt"),
            ExportFormat::Srt => write!(f, "srt"),
            ExportFormat::Vtt => write!(f, "vtt"),
            ExportFormat::Json => write!(f, "json"),
            ExportFormat::Markdown => write!(f, "md"),
        }
    }
}

impl ExportFormat {
    pub fn file_extension(&self) -> &str {
        match self {
            ExportFormat::Txt => "txt",
            ExportFormat::Srt => "srt",
            ExportFormat::Vtt => "vtt",
            ExportFormat::Json => "json",
            ExportFormat::Markdown => "md",
        }
    }

    pub fn mime_type(&self) -> &str {
        match self {
            ExportFormat::Txt => "text/plain",
            ExportFormat::Srt => "application/x-subrip",
            ExportFormat::Vtt => "text/vtt",
            ExportFormat::Json => "application/json",
            ExportFormat::Markdown => "text/markdown",
        }
    }
}

/// Format a unix timestamp (seconds) to a human-readable local datetime string
fn format_timestamp(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|dt| dt.with_timezone(&Local).format("%B %e, %Y %l:%M %p").to_string())
        .unwrap_or_else(|| format!("Unknown ({})", timestamp))
}

/// Format seconds into SRT/VTT timestamp format (HH:MM:SS,mmm for SRT, HH:MM:SS.mmm for VTT)
fn format_subtitle_time(seconds: f64, use_comma: bool) -> String {
    let total_ms = (seconds * 1000.0) as u64;
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let secs = (total_ms % 60_000) / 1000;
    let ms = total_ms % 1000;

    let separator = if use_comma { ',' } else { '.' };
    format!("{:02}:{:02}:{:02}{}{:03}", hours, minutes, secs, separator, ms)
}

/// Export a single history entry as plain text
pub fn export_as_txt(entry: &HistoryEntry) -> String {
    let date = format_timestamp(entry.timestamp);
    let text = entry.post_processed_text.as_deref().unwrap_or(&entry.transcription_text);

    format!("Transcription - {}\n\n{}\n", date, text)
}

/// Export a single history entry as SRT subtitle format.
/// Since we don't have word-level timestamps, we create a single subtitle block
/// spanning the full duration estimate based on text length.
pub fn export_as_srt(entry: &HistoryEntry) -> String {
    let text = entry.post_processed_text.as_deref().unwrap_or(&entry.transcription_text);

    // Estimate duration: ~150 words per minute reading speed
    let word_count = text.split_whitespace().count();
    let estimated_duration = (word_count as f64 / 150.0) * 60.0;
    let duration = estimated_duration.max(1.0); // minimum 1 second

    // Split text into segments of ~10 words for better subtitle readability
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut srt = String::new();
    let words_per_segment = 10;
    let time_per_word = duration / words.len().max(1) as f64;

    for (i, chunk) in words.chunks(words_per_segment).enumerate() {
        let start_time = i as f64 * words_per_segment as f64 * time_per_word;
        let end_time = ((i + 1) as f64 * words_per_segment as f64 * time_per_word).min(duration);

        srt.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            i + 1,
            format_subtitle_time(start_time, true),
            format_subtitle_time(end_time, true),
            chunk.join(" ")
        ));
    }

    srt
}

/// Export a single history entry as WebVTT subtitle format
pub fn export_as_vtt(entry: &HistoryEntry) -> String {
    let text = entry.post_processed_text.as_deref().unwrap_or(&entry.transcription_text);

    let word_count = text.split_whitespace().count();
    let estimated_duration = (word_count as f64 / 150.0) * 60.0;
    let duration = estimated_duration.max(1.0);

    let words: Vec<&str> = text.split_whitespace().collect();
    let mut vtt = String::from("WEBVTT\n\n");
    let words_per_segment = 10;
    let time_per_word = duration / words.len().max(1) as f64;

    for (i, chunk) in words.chunks(words_per_segment).enumerate() {
        let start_time = i as f64 * words_per_segment as f64 * time_per_word;
        let end_time = ((i + 1) as f64 * words_per_segment as f64 * time_per_word).min(duration);

        vtt.push_str(&format!(
            "{} --> {}\n{}\n\n",
            format_subtitle_time(start_time, false),
            format_subtitle_time(end_time, false),
            chunk.join(" ")
        ));
    }

    vtt
}

/// Export a single history entry as JSON
pub fn export_as_json(entry: &HistoryEntry) -> Result<String, String> {
    #[derive(Serialize)]
    struct ExportedEntry {
        id: i64,
        timestamp: i64,
        date: String,
        transcription: String,
        post_processed: Option<String>,
    }

    let exported = ExportedEntry {
        id: entry.id,
        timestamp: entry.timestamp,
        date: format_timestamp(entry.timestamp),
        transcription: entry.transcription_text.clone(),
        post_processed: entry.post_processed_text.clone(),
    };

    serde_json::to_string_pretty(&exported).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Export a single history entry as Markdown
pub fn export_as_markdown(entry: &HistoryEntry) -> String {
    let date = format_timestamp(entry.timestamp);
    let text = entry.post_processed_text.as_deref().unwrap_or(&entry.transcription_text);

    let mut md = format!("# Transcription\n\n**Date:** {}\n\n", date);

    md.push_str("## Text\n\n");
    md.push_str(text);
    md.push('\n');

    if entry.post_processed_text.is_some() {
        md.push_str("\n## Original Transcription\n\n");
        md.push_str(&entry.transcription_text);
        md.push('\n');
    }

    md
}

/// Export a history entry in the specified format
pub fn export_entry(entry: &HistoryEntry, format: &ExportFormat) -> Result<String, String> {
    match format {
        ExportFormat::Txt => Ok(export_as_txt(entry)),
        ExportFormat::Srt => Ok(export_as_srt(entry)),
        ExportFormat::Vtt => Ok(export_as_vtt(entry)),
        ExportFormat::Json => export_as_json(entry),
        ExportFormat::Markdown => Ok(export_as_markdown(entry)),
    }
}

/// Export multiple history entries as a single text file
pub fn export_entries_as_txt(entries: &[HistoryEntry]) -> String {
    entries
        .iter()
        .map(|e| export_as_txt(e))
        .collect::<Vec<_>>()
        .join("\n---\n\n")
}

/// Export multiple history entries as JSON array
pub fn export_entries_as_json(entries: &[HistoryEntry]) -> Result<String, String> {
    #[derive(Serialize)]
    struct ExportedEntry {
        id: i64,
        timestamp: i64,
        date: String,
        transcription: String,
        post_processed: Option<String>,
    }

    let exported: Vec<ExportedEntry> = entries
        .iter()
        .map(|e| ExportedEntry {
            id: e.id,
            timestamp: e.timestamp,
            date: format_timestamp(e.timestamp),
            transcription: e.transcription_text.clone(),
            post_processed: e.post_processed_text.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&exported).map_err(|e| format!("Failed to serialize: {}", e))
}

/// Export multiple history entries as Markdown
pub fn export_entries_as_markdown(entries: &[HistoryEntry]) -> String {
    let mut md = String::from("# Transcription Export\n\n");
    md.push_str(&format!(
        "**Exported:** {}  \n",
        Utc::now()
            .with_timezone(&Local)
            .format("%B %e, %Y %l:%M %p")
    ));
    md.push_str(&format!("**Entries:** {}\n\n---\n\n", entries.len()));

    for entry in entries {
        let date = format_timestamp(entry.timestamp);
        let text = entry
            .post_processed_text
            .as_deref()
            .unwrap_or(&entry.transcription_text);

        md.push_str(&format!("## {}\n\n{}\n\n---\n\n", date, text));
    }

    md
}

/// Export multiple entries in the specified format
pub fn export_entries(entries: &[HistoryEntry], format: &ExportFormat) -> Result<String, String> {
    match format {
        ExportFormat::Txt => Ok(export_entries_as_txt(entries)),
        ExportFormat::Json => export_entries_as_json(entries),
        ExportFormat::Markdown => Ok(export_entries_as_markdown(entries)),
        // For subtitle formats, concatenate individual exports
        ExportFormat::Srt => {
            let parts: Vec<String> = entries.iter().map(|e| export_as_srt(e)).collect();
            Ok(parts.join("\n"))
        }
        ExportFormat::Vtt => {
            // VTT should have a single header
            let mut vtt = String::from("WEBVTT\n\n");
            for entry in entries {
                let text = entry
                    .post_processed_text
                    .as_deref()
                    .unwrap_or(&entry.transcription_text);
                let word_count = text.split_whitespace().count();
                let estimated_duration = (word_count as f64 / 150.0) * 60.0;
                let duration = estimated_duration.max(1.0);
                let words: Vec<&str> = text.split_whitespace().collect();
                let words_per_segment = 10;
                let time_per_word = duration / words.len().max(1) as f64;

                // Add a note cue with the date
                let date = format_timestamp(entry.timestamp);
                vtt.push_str(&format!("NOTE {}\n\n", date));

                for (i, chunk) in words.chunks(words_per_segment).enumerate() {
                    let start_time = i as f64 * words_per_segment as f64 * time_per_word;
                    let end_time = ((i + 1) as f64 * words_per_segment as f64 * time_per_word)
                        .min(duration);
                    vtt.push_str(&format!(
                        "{} --> {}\n{}\n\n",
                        format_subtitle_time(start_time, false),
                        format_subtitle_time(end_time, false),
                        chunk.join(" ")
                    ));
                }
            }
            Ok(vtt)
        }
    }
}

/// Generate a default filename for export
pub fn generate_export_filename(entry: &HistoryEntry, format: &ExportFormat) -> String {
    let date = DateTime::from_timestamp(entry.timestamp, 0)
        .map(|dt| dt.with_timezone(&Local).format("%Y-%m-%d_%H%M%S").to_string())
        .unwrap_or_else(|| entry.timestamp.to_string());

    format!("handy-transcription-{}.{}", date, format.file_extension())
}

/// Generate a default filename for batch export
pub fn generate_batch_export_filename(format: &ExportFormat) -> String {
    let date = Utc::now()
        .with_timezone(&Local)
        .format("%Y-%m-%d_%H%M%S")
        .to_string();

    format!("handy-export-{}.{}", date, format.file_extension())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry() -> HistoryEntry {
        HistoryEntry {
            id: 1,
            file_name: "handy-1700000000.wav".to_string(),
            timestamp: 1700000000,
            saved: false,
            title: "Test Entry".to_string(),
            transcription_text: "Hello world this is a test transcription with enough words to make multiple subtitle segments for testing purposes".to_string(),
            post_processed_text: None,
            post_process_prompt: None,
        }
    }

    fn sample_entry_with_post_processed() -> HistoryEntry {
        HistoryEntry {
            id: 2,
            file_name: "handy-1700000001.wav".to_string(),
            timestamp: 1700000001,
            saved: true,
            title: "Processed Entry".to_string(),
            transcription_text: "hello world".to_string(),
            post_processed_text: Some("Hello, World!".to_string()),
            post_process_prompt: Some("Fix grammar".to_string()),
        }
    }

    #[test]
    fn test_export_as_txt() {
        let entry = sample_entry();
        let result = export_as_txt(&entry);
        assert!(result.contains("Transcription"));
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_export_as_txt_uses_post_processed() {
        let entry = sample_entry_with_post_processed();
        let result = export_as_txt(&entry);
        assert!(result.contains("Hello, World!"));
        assert!(!result.contains("hello world"));
    }

    #[test]
    fn test_export_as_srt() {
        let entry = sample_entry();
        let result = export_as_srt(&entry);
        // SRT must start with sequence number 1
        assert!(result.starts_with("1\n"));
        // Must contain timestamp arrows
        assert!(result.contains(" --> "));
        // SRT uses comma for millisecond separator
        assert!(result.contains(","));
    }

    #[test]
    fn test_export_as_vtt() {
        let entry = sample_entry();
        let result = export_as_vtt(&entry);
        // VTT must start with WEBVTT header
        assert!(result.starts_with("WEBVTT\n"));
        // Must contain timestamp arrows
        assert!(result.contains(" --> "));
        // VTT uses period for millisecond separator
        assert!(result.contains("."));
    }

    #[test]
    fn test_export_as_json() {
        let entry = sample_entry();
        let result = export_as_json(&entry).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["id"], 1);
        assert_eq!(parsed["transcription"], "Hello world this is a test transcription with enough words to make multiple subtitle segments for testing purposes");
    }

    #[test]
    fn test_export_as_markdown() {
        let entry = sample_entry();
        let result = export_as_markdown(&entry);
        assert!(result.starts_with("# Transcription"));
        assert!(result.contains("## Text"));
        assert!(result.contains("Hello world"));
    }

    #[test]
    fn test_export_as_markdown_with_post_processed() {
        let entry = sample_entry_with_post_processed();
        let result = export_as_markdown(&entry);
        assert!(result.contains("Hello, World!"));
        assert!(result.contains("## Original Transcription"));
        assert!(result.contains("hello world"));
    }

    #[test]
    fn test_format_subtitle_time_srt() {
        assert_eq!(format_subtitle_time(0.0, true), "00:00:00,000");
        assert_eq!(format_subtitle_time(1.5, true), "00:00:01,500");
        assert_eq!(format_subtitle_time(3661.123, true), "01:01:01,123");
    }

    #[test]
    fn test_format_subtitle_time_vtt() {
        assert_eq!(format_subtitle_time(0.0, false), "00:00:00.000");
        assert_eq!(format_subtitle_time(1.5, false), "00:00:01.500");
    }

    #[test]
    fn test_generate_export_filename() {
        let entry = sample_entry();
        let filename = generate_export_filename(&entry, &ExportFormat::Srt);
        assert!(filename.starts_with("handy-transcription-"));
        assert!(filename.ends_with(".srt"));
    }

    #[test]
    fn test_export_entries_as_json() {
        let entries = vec![sample_entry(), sample_entry_with_post_processed()];
        let result = export_entries_as_json(&entries).unwrap();
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed.len(), 2);
    }

    #[test]
    fn test_export_entry_all_formats() {
        let entry = sample_entry();
        for format in [
            ExportFormat::Txt,
            ExportFormat::Srt,
            ExportFormat::Vtt,
            ExportFormat::Json,
            ExportFormat::Markdown,
        ] {
            let result = export_entry(&entry, &format);
            assert!(result.is_ok(), "Failed to export as {:?}", format);
            assert!(!result.unwrap().is_empty(), "Empty export for {:?}", format);
        }
    }
}
