//! Universal audio file decoder using symphonia.
//!
//! Decodes various audio formats (WAV, MP3, M4A, AAC, FLAC, OGG, MP4) to
//! f32 samples at 16kHz mono, ready for transcription.

use std::fs::File;
use std::path::Path;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use specta::Type;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Target sample rate for transcription models
const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Result of decoding an audio file
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DecodedAudioInfo {
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Original file format (e.g., "mp3", "wav")
    pub original_format: String,
    /// Original sample rate
    pub sample_rate: u32,
    /// Number of samples after decoding to 16kHz mono
    pub num_samples: usize,
}

/// Decoded audio data with samples and metadata
pub struct DecodedAudio {
    /// Audio samples as f32, resampled to 16kHz mono
    pub samples: Vec<f32>,
    /// Duration in seconds
    pub duration_seconds: f64,
    /// Original file format
    pub original_format: String,
    /// Original sample rate before resampling
    pub sample_rate: u32,
}

impl DecodedAudio {
    /// Get metadata info (without the large sample buffer)
    pub fn info(&self) -> DecodedAudioInfo {
        DecodedAudioInfo {
            duration_seconds: self.duration_seconds,
            original_format: self.original_format.clone(),
            sample_rate: self.sample_rate,
            num_samples: self.samples.len(),
        }
    }
}

/// Decode an audio file to f32 samples at 16kHz mono.
///
/// Supports WAV, MP3, M4A, AAC, FLAC, OGG, and MP4 formats.
pub fn decode_audio_file(path: &Path) -> Result<DecodedAudio, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    // Provide a hint based on file extension
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let format_opts = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    let metadata_opts = MetadataOptions::default();
    let decoder_opts = DecoderOptions::default();

    // Probe the media source to detect format
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| format!("Failed to probe audio format: {}", e))?;

    let mut format = probed.format;

    // Find the first audio track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| "No supported audio track found in file".to_string())?;

    let track_id = track.id;
    let codec_params = track.codec_params.clone();

    let original_sample_rate = codec_params.sample_rate.unwrap_or(44100);
    let channels = codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(1);

    // Determine format name from extension or codec
    let original_format = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("unknown")
        .to_lowercase();

    debug!(
        "Decoding audio file: {} (format={}, sample_rate={}, channels={})",
        path.display(),
        original_format,
        original_sample_rate,
        channels
    );

    // Create the decoder
    let mut decoder = symphonia::default::get_codecs()
        .make(&codec_params, &decoder_opts)
        .map_err(|e| format!("Failed to create audio decoder: {}", e))?;

    // Decode all packets and collect samples
    let mut all_samples: Vec<f32> = Vec::new();

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                // End of stream
                break;
            }
            Err(e) => {
                debug!("Error reading packet (may be end of stream): {}", e);
                break;
            }
        };

        // Skip packets from other tracks
        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let spec = *decoded.spec();
                let num_frames = decoded.frames();
                let num_channels = spec.channels.count();

                let mut sample_buf = SampleBuffer::<f32>::new(
                    num_frames as u64,
                    *decoded.spec(),
                );
                sample_buf.copy_interleaved_ref(decoded);

                let interleaved = sample_buf.samples();

                // Convert to mono by averaging channels
                if num_channels == 1 {
                    all_samples.extend_from_slice(interleaved);
                } else {
                    for chunk in interleaved.chunks(num_channels) {
                        let sum: f32 = chunk.iter().sum();
                        all_samples.push(sum / num_channels as f32);
                    }
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(msg)) => {
                debug!("Decode error (skipping packet): {}", msg);
                continue;
            }
            Err(e) => {
                return Err(format!("Fatal decode error: {}", e));
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were decoded from the file".to_string());
    }

    // Resample to target sample rate if needed
    let original_sample_count = all_samples.len();
    let resampled = if original_sample_rate != TARGET_SAMPLE_RATE {
        resample_linear(&all_samples, original_sample_rate, TARGET_SAMPLE_RATE)
    } else {
        all_samples
    };

    let duration_seconds = resampled.len() as f64 / TARGET_SAMPLE_RATE as f64;

    info!(
        "Decoded audio: {:.1}s, {} samples at {}Hz -> {} samples at {}Hz",
        duration_seconds,
        original_sample_count,
        original_sample_rate,
        resampled.len(),
        TARGET_SAMPLE_RATE,
    );

    Ok(DecodedAudio {
        samples: resampled,
        duration_seconds,
        original_format,
        sample_rate: original_sample_rate,
    })
}

/// Simple linear interpolation resampler for converting between sample rates.
///
/// For transcription purposes, this provides sufficient quality while being
/// fast and dependency-free. The audio is already being processed by a
/// neural network that is robust to minor artifacts.
fn resample_linear(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate || samples.is_empty() {
        return samples.to_vec();
    }

    let ratio = from_rate as f64 / to_rate as f64;
    let output_len = ((samples.len() as f64) / ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos.floor() as usize;
        let frac = (src_pos - src_idx as f64) as f32;

        if src_idx + 1 < samples.len() {
            // Linear interpolation between adjacent samples
            let sample = samples[src_idx] * (1.0 - frac) + samples[src_idx + 1] * frac;
            output.push(sample);
        } else if src_idx < samples.len() {
            output.push(samples[src_idx]);
        }
    }

    output
}

/// Get list of supported audio file extensions
pub fn get_supported_extensions() -> Vec<&'static str> {
    vec!["wav", "mp3", "m4a", "aac", "flac", "ogg", "mp4"]
}

/// Check if a file extension is a supported audio format
pub fn is_supported_format(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| get_supported_extensions().contains(&ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_linear_same_rate() {
        let samples = vec![1.0, 2.0, 3.0, 4.0];
        let result = resample_linear(&samples, 16000, 16000);
        assert_eq!(result, samples);
    }

    #[test]
    fn test_resample_linear_empty() {
        let samples: Vec<f32> = vec![];
        let result = resample_linear(&samples, 44100, 16000);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resample_linear_downsample() {
        // 4 samples at 44100 -> fewer samples at 16000
        let samples = vec![0.0, 0.5, 1.0, 0.5];
        let result = resample_linear(&samples, 44100, 16000);
        // Output should have fewer samples
        assert!(result.len() < samples.len());
        assert!(!result.is_empty());
    }

    #[test]
    fn test_resample_linear_upsample() {
        let samples = vec![0.0, 1.0];
        let result = resample_linear(&samples, 8000, 16000);
        // Output should have more samples
        assert!(result.len() > samples.len());
    }

    #[test]
    fn test_supported_extensions() {
        let extensions = get_supported_extensions();
        assert!(extensions.contains(&"wav"));
        assert!(extensions.contains(&"mp3"));
        assert!(extensions.contains(&"flac"));
        assert!(extensions.contains(&"m4a"));
    }

    #[test]
    fn test_is_supported_format() {
        assert!(is_supported_format(Path::new("test.mp3")));
        assert!(is_supported_format(Path::new("test.WAV")));
        assert!(is_supported_format(Path::new("/path/to/file.flac")));
        assert!(!is_supported_format(Path::new("test.txt")));
        assert!(!is_supported_format(Path::new("noext")));
    }
}
