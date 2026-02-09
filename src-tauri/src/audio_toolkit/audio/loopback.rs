//! Platform-specific loopback (system audio) capture module.
//!
//! This module provides the ability to capture system audio output (what's playing
//! through speakers/headphones) for transcription. This is useful for capturing
//! what others say in video calls.
//!
//! ## Platform Support
//!
//! - **Windows**: Uses WASAPI loopback mode via cpal
//! - **macOS**: Requires macOS 12.3+ with ScreenCaptureKit or virtual audio devices
//! - **Linux**: Uses PulseAudio/PipeWire monitor sources
//!
//! ## Usage
//!
//! ```rust,ignore
//! use crate::audio_toolkit::audio::loopback::{LoopbackCapture, LoopbackSupport};
//!
//! if LoopbackCapture::is_supported() {
//!     let devices = LoopbackCapture::list_devices()?;
//!     let capture = LoopbackCapture::new(devices.first())?;
//!     capture.start(|samples| {
//!         // Process captured system audio
//!     })?;
//! }
//! ```

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::audio_toolkit::constants;

/// Information about a loopback capture device
#[derive(Clone)]
pub struct LoopbackDeviceInfo {
    /// Unique identifier for the device
    pub id: String,
    /// Human-readable device name
    pub name: String,
    /// Whether this is the default output device
    pub is_default: bool,
    /// The underlying cpal device (if available)
    #[allow(dead_code)]
    device: Option<cpal::Device>,
}

impl std::fmt::Debug for LoopbackDeviceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoopbackDeviceInfo")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("is_default", &self.is_default)
            .field("device", &self.device.as_ref().map(|_| "<cpal::Device>"))
            .finish()
    }
}

/// Represents the level of loopback support on the current platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopbackSupport {
    /// Full native support for loopback capture
    Native,
    /// Requires additional software/virtual audio device
    RequiresVirtualDevice,
    /// Not supported on this platform
    NotSupported,
}

/// Errors that can occur during loopback capture
#[derive(Debug)]
pub enum LoopbackError {
    /// Loopback is not supported on this platform
    NotSupported(String),
    /// No loopback devices found
    NoDevicesFound,
    /// Failed to open the device
    DeviceOpenError(String),
    /// Failed to build the audio stream
    StreamBuildError(String),
    /// Failed to start the stream
    StreamStartError(String),
    /// The capture is already running
    AlreadyRunning,
    /// The capture is not running
    NotRunning,
}

impl std::fmt::Display for LoopbackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoopbackError::NotSupported(msg) => write!(f, "Loopback not supported: {}", msg),
            LoopbackError::NoDevicesFound => write!(f, "No loopback devices found"),
            LoopbackError::DeviceOpenError(msg) => write!(f, "Failed to open device: {}", msg),
            LoopbackError::StreamBuildError(msg) => write!(f, "Failed to build stream: {}", msg),
            LoopbackError::StreamStartError(msg) => write!(f, "Failed to start stream: {}", msg),
            LoopbackError::AlreadyRunning => write!(f, "Loopback capture is already running"),
            LoopbackError::NotRunning => write!(f, "Loopback capture is not running"),
        }
    }
}

impl std::error::Error for LoopbackError {}

/// Callback type for receiving captured audio samples
pub type LoopbackCallback = Arc<dyn Fn(&[f32]) + Send + Sync + 'static>;

/// Loopback (system audio) capture handler
pub struct LoopbackCapture {
    device: cpal::Device,
    stream: Mutex<Option<cpal::Stream>>,
    is_running: AtomicBool,
    callback: Mutex<Option<LoopbackCallback>>,
}

impl LoopbackCapture {
    /// Check the level of loopback support on the current platform
    pub fn support_level() -> LoopbackSupport {
        #[cfg(target_os = "windows")]
        {
            // Windows has native WASAPI loopback support
            LoopbackSupport::Native
        }

        #[cfg(target_os = "macos")]
        {
            // macOS requires ScreenCaptureKit (12.3+) or virtual audio devices
            // For now, we mark it as requiring a virtual device
            // TODO: Add ScreenCaptureKit support when available in cpal
            LoopbackSupport::RequiresVirtualDevice
        }

        #[cfg(target_os = "linux")]
        {
            // Linux can use PulseAudio/PipeWire monitor sources
            // Check if we can find any monitor sources
            let host = crate::audio_toolkit::get_cpal_host();
            if host.input_devices().map(|d| d.count()).unwrap_or(0) > 0 {
                // PulseAudio/PipeWire monitor sources appear as input devices
                LoopbackSupport::Native
            } else {
                LoopbackSupport::NotSupported
            }
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            LoopbackSupport::NotSupported
        }
    }

    /// Check if loopback capture is supported on this platform
    pub fn is_supported() -> bool {
        !matches!(Self::support_level(), LoopbackSupport::NotSupported)
    }

    /// List available loopback devices
    pub fn list_devices() -> Result<Vec<LoopbackDeviceInfo>, LoopbackError> {
        let mut devices = Vec::new();

        #[cfg(target_os = "windows")]
        {
            // On Windows, use output devices for WASAPI loopback
            let host = crate::audio_toolkit::get_cpal_host();
            let default_name = host.default_output_device().and_then(|d| d.name().ok());

            if let Ok(output_devices) = host.output_devices() {
                for (index, device) in output_devices.enumerate() {
                    let name = device.name().unwrap_or_else(|_| "Unknown".into());
                    let is_default = Some(name.clone()) == default_name;

                    devices.push(LoopbackDeviceInfo {
                        id: format!("wasapi_loopback_{}", index),
                        name: format!("{} (Loopback)", name),
                        is_default,
                        device: Some(device),
                    });
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            // On Linux, look for monitor sources (PulseAudio/PipeWire)
            let host = crate::audio_toolkit::get_cpal_host();

            if let Ok(input_devices) = host.input_devices() {
                for (index, device) in input_devices.enumerate() {
                    let name = device.name().unwrap_or_else(|_| "Unknown".into());

                    // Monitor sources typically contain "Monitor" in the name
                    if name.to_lowercase().contains("monitor") {
                        devices.push(LoopbackDeviceInfo {
                            id: format!("pulse_monitor_{}", index),
                            name,
                            is_default: false,
                            device: Some(device),
                        });
                    }
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            // On macOS, look for virtual audio devices like BlackHole or Soundflower
            let host = crate::audio_toolkit::get_cpal_host();

            if let Ok(input_devices) = host.input_devices() {
                for (index, device) in input_devices.enumerate() {
                    let name = device.name().unwrap_or_else(|_| "Unknown".into());
                    let name_lower = name.to_lowercase();

                    // Look for known virtual audio devices
                    if name_lower.contains("blackhole")
                        || name_lower.contains("soundflower")
                        || name_lower.contains("loopback")
                        || name_lower.contains("virtual")
                    {
                        devices.push(LoopbackDeviceInfo {
                            id: format!("macos_virtual_{}", index),
                            name,
                            is_default: false,
                            device: Some(device),
                        });
                    }
                }
            }
        }

        if devices.is_empty() {
            Err(LoopbackError::NoDevicesFound)
        } else {
            Ok(devices)
        }
    }

    /// Create a new loopback capture for the specified device
    pub fn new(device_info: &LoopbackDeviceInfo) -> Result<Self, LoopbackError> {
        let device = device_info
            .device
            .clone()
            .ok_or_else(|| LoopbackError::DeviceOpenError("Device not available".into()))?;

        Ok(Self {
            device,
            stream: Mutex::new(None),
            is_running: AtomicBool::new(false),
            callback: Mutex::new(None),
        })
    }

    /// Create a loopback capture for the default output device (Windows only)
    #[cfg(target_os = "windows")]
    pub fn new_default() -> Result<Self, LoopbackError> {
        let host = crate::audio_toolkit::get_cpal_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| LoopbackError::NoDevicesFound)?;

        Ok(Self {
            device,
            stream: Mutex::new(None),
            is_running: AtomicBool::new(false),
            callback: Mutex::new(None),
        })
    }

    /// Start capturing audio with the given callback
    pub fn start<F>(&self, callback: F) -> Result<(), LoopbackError>
    where
        F: Fn(&[f32]) + Send + Sync + 'static,
    {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(LoopbackError::AlreadyRunning);
        }

        let callback = Arc::new(callback);
        *self.callback.lock().unwrap() = Some(callback.clone());

        let stream = self.build_stream(callback)?;
        stream
            .play()
            .map_err(|e| LoopbackError::StreamStartError(e.to_string()))?;

        *self.stream.lock().unwrap() = Some(stream);
        self.is_running.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// Stop capturing audio
    pub fn stop(&self) -> Result<(), LoopbackError> {
        if !self.is_running.load(Ordering::SeqCst) {
            return Err(LoopbackError::NotRunning);
        }

        // Drop the stream to stop capture
        *self.stream.lock().unwrap() = None;
        *self.callback.lock().unwrap() = None;
        self.is_running.store(false, Ordering::SeqCst);

        Ok(())
    }

    /// Check if capture is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    /// Build the audio stream for capturing
    fn build_stream(&self, callback: LoopbackCallback) -> Result<cpal::Stream, LoopbackError> {
        // Get device config - for loopback we use the output config
        #[cfg(target_os = "windows")]
        let config = self.device.default_output_config();

        #[cfg(not(target_os = "windows"))]
        let config = self.device.default_input_config();

        let config = config.map_err(|e| LoopbackError::DeviceOpenError(e.to_string()))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;

        log::info!(
            "Loopback capture: {} Hz, {} channels, {:?}",
            sample_rate,
            channels,
            config.sample_format()
        );

        // Build resampler for converting to Whisper's expected sample rate
        let mut resampler = crate::audio_toolkit::audio::FrameResampler::new(
            sample_rate as usize,
            constants::WHISPER_SAMPLE_RATE as usize,
            std::time::Duration::from_millis(30),
        );

        let callback_clone = callback.clone();

        let stream_callback = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            // Convert to mono if needed
            let mono_samples: Vec<f32> = if channels == 1 {
                data.to_vec()
            } else {
                data.chunks_exact(channels)
                    .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                    .collect()
            };

            // Resample and forward to callback
            resampler.push(&mono_samples, &mut |frame: &[f32]| {
                callback_clone(frame);
            });
        };

        let error_callback = |err: cpal::StreamError| {
            log::error!("Loopback stream error: {}", err);
        };

        // Build the stream based on platform
        #[cfg(target_os = "windows")]
        {
            // On Windows, we need to use the output device with loopback mode
            // cpal doesn't directly support WASAPI loopback, so we use the input stream
            // configuration but connect to the output device
            self.device
                .build_input_stream(&config.into(), stream_callback, error_callback, None)
                .map_err(|e| LoopbackError::StreamBuildError(e.to_string()))
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On other platforms, use standard input stream
            self.device
                .build_input_stream(&config.into(), stream_callback, error_callback, None)
                .map_err(|e| LoopbackError::StreamBuildError(e.to_string()))
        }
    }
}

impl Drop for LoopbackCapture {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_support_level() {
        let support = LoopbackCapture::support_level();
        // Just verify it returns a valid variant
        match support {
            LoopbackSupport::Native => println!("Native loopback support"),
            LoopbackSupport::RequiresVirtualDevice => println!("Requires virtual device"),
            LoopbackSupport::NotSupported => println!("Not supported"),
        }
    }

    #[test]
    fn test_is_supported() {
        let supported = LoopbackCapture::is_supported();
        println!("Loopback supported: {}", supported);
    }

    #[test]
    fn test_list_devices() {
        match LoopbackCapture::list_devices() {
            Ok(devices) => {
                println!("Found {} loopback devices:", devices.len());
                for device in devices {
                    println!(
                        "  - {} (id: {}, default: {})",
                        device.name, device.id, device.is_default
                    );
                }
            }
            Err(LoopbackError::NoDevicesFound) => {
                println!("No loopback devices found (expected on some platforms)");
            }
            Err(e) => {
                println!("Error listing devices: {}", e);
            }
        }
    }
}
