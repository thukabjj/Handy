use crate::audio_toolkit::{list_input_devices, vad::SmoothedVad, AudioRecorder, SileroVad};
use crate::helpers::clamshell;
use crate::settings::{get_settings, AppSettings};
use crate::utils;
use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::Manager;

/// Helper macro to safely acquire a mutex lock and return early on failure
macro_rules! safe_lock {
    ($mutex:expr) => {
        match $mutex.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Mutex poisoned: {}", e);
                return;
            }
        }
    };
    ($mutex:expr, $ret:expr) => {
        match $mutex.lock() {
            Ok(guard) => guard,
            Err(e) => {
                error!("Mutex poisoned: {}", e);
                return $ret;
            }
        }
    };
}

/// Helper macro for Result-returning functions
macro_rules! safe_lock_err {
    ($mutex:expr) => {
        $mutex
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex poisoned: {}", e))?
    };
}

fn set_mute(mute: bool) {
    // Expected behavior:
    // - Windows: works on most systems using standard audio drivers.
    // - Linux: works on many systems (PipeWire, PulseAudio, ALSA),
    //   but some distros may lack the tools used.
    // - macOS: works on most standard setups via AppleScript.
    // If unsupported, fails silently.

    #[cfg(target_os = "windows")]
    {
        unsafe {
            use windows::Win32::{
                Media::Audio::{
                    eMultimedia, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator,
                    MMDeviceEnumerator,
                },
                System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            };

            macro_rules! unwrap_or_return {
                ($expr:expr) => {
                    match $expr {
                        Ok(val) => val,
                        Err(_) => return,
                    }
                };
            }

            // Initialize the COM library for this thread.
            // If already initialized (e.g., by another library like Tauri), this does nothing.
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            let all_devices: IMMDeviceEnumerator =
                unwrap_or_return!(CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL));
            let default_device =
                unwrap_or_return!(all_devices.GetDefaultAudioEndpoint(eRender, eMultimedia));
            let volume_interface = unwrap_or_return!(
                default_device.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            );

            let _ = volume_interface.SetMute(mute, std::ptr::null());
        }
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let mute_val = if mute { "1" } else { "0" };
        let amixer_state = if mute { "mute" } else { "unmute" };

        // Try multiple backends to increase compatibility
        // 1. PipeWire (wpctl)
        if Command::new("wpctl")
            .args(["set-mute", "@DEFAULT_AUDIO_SINK@", mute_val])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return;
        }

        // 2. PulseAudio (pactl)
        if Command::new("pactl")
            .args(["set-sink-mute", "@DEFAULT_SINK@", mute_val])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return;
        }

        // 3. ALSA (amixer)
        let _ = Command::new("amixer")
            .args(["set", "Master", amixer_state])
            .output();
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let script = format!(
            "set volume output muted {}",
            if mute { "true" } else { "false" }
        );
        let _ = Command::new("osascript").args(["-e", &script]).output();
    }
}

const WHISPER_SAMPLE_RATE: usize = 16000;

/* ──────────────────────────────────────────────────────────────── */

#[derive(Clone, Debug)]
pub enum RecordingState {
    Idle,
    Recording { binding_id: String },
}

#[derive(Clone, Debug, PartialEq)]
pub enum MicrophoneMode {
    AlwaysOn,
    OnDemand,
    ActiveListening,
}

/* ──────────────────────────────────────────────────────────────── */

fn create_audio_recorder(
    vad_path: &str,
    app_handle: &tauri::AppHandle,
    sample_callback: Option<ActiveListeningCallback>,
) -> Result<AudioRecorder, anyhow::Error> {
    let silero = SileroVad::new(vad_path, 0.3)
        .map_err(|e| anyhow::anyhow!("Failed to create SileroVad: {}", e))?;
    let smoothed_vad = SmoothedVad::new(Box::new(silero), 15, 15, 2);

    // Recorder with VAD plus a spectrum-level callback that forwards updates to
    // the frontend.
    let mut recorder = AudioRecorder::new()
        .map_err(|e| anyhow::anyhow!("Failed to create AudioRecorder: {}", e))?
        .with_vad(Box::new(smoothed_vad))
        .with_level_callback({
            let app_handle = app_handle.clone();
            move |levels| {
                utils::emit_levels(&app_handle, &levels);
            }
        });

    // If a sample callback is provided, wire it up for Active Listening
    if let Some(cb) = sample_callback {
        recorder = recorder.with_sample_callback(move |samples| {
            cb(samples);
        });
    }

    Ok(recorder)
}

/* ──────────────────────────────────────────────────────────────── */

/// Callback type for active listening audio samples
pub type ActiveListeningCallback = Arc<dyn Fn(&[f32]) + Send + Sync + 'static>;

#[derive(Clone)]
pub struct AudioRecordingManager {
    state: Arc<Mutex<RecordingState>>,
    mode: Arc<Mutex<MicrophoneMode>>,
    app_handle: tauri::AppHandle,

    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    is_open: Arc<Mutex<bool>>,
    is_recording: Arc<Mutex<bool>>,
    did_mute: Arc<Mutex<bool>>,

    /// Callback for forwarding audio samples in active listening mode
    active_listening_callback: Arc<Mutex<Option<ActiveListeningCallback>>>,
}

impl AudioRecordingManager {
    /* ---------- construction ------------------------------------------------ */

    pub fn new(app: &tauri::AppHandle) -> Result<Self, anyhow::Error> {
        let settings = get_settings(app);
        let mode = if settings.always_on_microphone {
            MicrophoneMode::AlwaysOn
        } else {
            MicrophoneMode::OnDemand
        };

        let manager = Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            mode: Arc::new(Mutex::new(mode.clone())),
            app_handle: app.clone(),

            recorder: Arc::new(Mutex::new(None)),
            is_open: Arc::new(Mutex::new(false)),
            is_recording: Arc::new(Mutex::new(false)),
            did_mute: Arc::new(Mutex::new(false)),
            active_listening_callback: Arc::new(Mutex::new(None)),
        };

        // Always-on?  Open immediately.
        if matches!(mode, MicrophoneMode::AlwaysOn) {
            manager.start_microphone_stream()?;
        }

        Ok(manager)
    }

    /* ---------- helper methods --------------------------------------------- */

    fn get_effective_microphone_device(&self, settings: &AppSettings) -> Option<cpal::Device> {
        // Check if we're in clamshell mode and have a clamshell microphone configured
        let use_clamshell_mic = if let Ok(is_clamshell) = clamshell::is_clamshell() {
            is_clamshell && settings.clamshell_microphone.is_some()
        } else {
            false
        };

        let device_name = if use_clamshell_mic {
            settings.clamshell_microphone.as_ref().unwrap()
        } else {
            settings.selected_microphone.as_ref()?
        };

        // Find the device by name
        match list_input_devices() {
            Ok(devices) => devices
                .into_iter()
                .find(|d| d.name == *device_name)
                .map(|d| d.device),
            Err(e) => {
                debug!("Failed to list devices, using default: {}", e);
                None
            }
        }
    }

    /* ---------- microphone life-cycle -------------------------------------- */

    /// Applies mute if mute_while_recording is enabled and stream is open
    pub fn apply_mute(&self) {
        let settings = get_settings(&self.app_handle);
        let mut did_mute_guard = safe_lock!(self.did_mute);

        let is_open = match self.is_open.lock() {
            Ok(guard) => *guard,
            Err(e) => {
                warn!("Failed to check is_open for mute: {}", e);
                return;
            }
        };

        if settings.general.mute_while_recording && is_open {
            set_mute(true);
            *did_mute_guard = true;
            debug!("Mute applied");
        }
    }

    /// Removes mute if it was applied
    pub fn remove_mute(&self) {
        let mut did_mute_guard = safe_lock!(self.did_mute);
        if *did_mute_guard {
            set_mute(false);
            *did_mute_guard = false;
            debug!("Mute removed");
        }
    }

    pub fn start_microphone_stream(&self) -> Result<(), anyhow::Error> {
        let mut open_flag = safe_lock_err!(self.is_open);
        if *open_flag {
            debug!("Microphone stream already active");
            return Ok(());
        }

        let start_time = Instant::now();

        // Don't mute immediately - caller will handle muting after audio feedback
        let mut did_mute_guard = safe_lock_err!(self.did_mute);
        *did_mute_guard = false;

        let vad_path = self
            .app_handle
            .path()
            .resolve(
                "resources/models/silero_vad_v4.onnx",
                tauri::path::BaseDirectory::Resource,
            )
            .map_err(|e| anyhow::anyhow!("Failed to resolve VAD path: {}", e))?;
        let mut recorder_opt = safe_lock_err!(self.recorder);

        if recorder_opt.is_none() {
            // Get sample callback if in Active Listening mode
            let sample_callback = {
                let mode = safe_lock_err!(self.mode);
                if *mode == MicrophoneMode::ActiveListening {
                    safe_lock_err!(self.active_listening_callback).clone()
                } else {
                    None
                }
            };

            let vad_path_str = vad_path
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid VAD path"))?;
            *recorder_opt = Some(create_audio_recorder(
                vad_path_str,
                &self.app_handle,
                sample_callback,
            )?);
        }

        // Get the selected device from settings, considering clamshell mode
        let settings = get_settings(&self.app_handle);
        let selected_device = self.get_effective_microphone_device(&settings);

        if let Some(rec) = recorder_opt.as_mut() {
            rec.open(selected_device)
                .map_err(|e| anyhow::anyhow!("Failed to open recorder: {}", e))?;
        }

        *open_flag = true;
        info!(
            "Microphone stream initialized in {:?}",
            start_time.elapsed()
        );
        Ok(())
    }

    pub fn stop_microphone_stream(&self) {
        let mut open_flag = safe_lock!(self.is_open);
        if !*open_flag {
            return;
        }

        let mut did_mute_guard = safe_lock!(self.did_mute);
        if *did_mute_guard {
            set_mute(false);
        }
        *did_mute_guard = false;

        if let Ok(mut recorder_guard) = self.recorder.lock() {
            if let Some(rec) = recorder_guard.as_mut() {
                // If still recording, stop first.
                if let Ok(mut is_rec) = self.is_recording.lock() {
                    if *is_rec {
                        let _ = rec.stop();
                        *is_rec = false;
                    }
                }
                let _ = rec.close();
            }
        }

        *open_flag = false;
        debug!("Microphone stream stopped");
    }

    /* ---------- mode switching --------------------------------------------- */

    pub fn update_mode(&self, new_mode: MicrophoneMode) -> Result<(), anyhow::Error> {
        let mode_guard = safe_lock_err!(self.mode);
        let cur_mode = mode_guard.clone();

        match (cur_mode, &new_mode) {
            (MicrophoneMode::AlwaysOn, MicrophoneMode::OnDemand) => {
                let is_idle = match self.state.lock() {
                    Ok(state) => matches!(*state, RecordingState::Idle),
                    Err(_) => false,
                };
                if is_idle {
                    drop(mode_guard);
                    self.stop_microphone_stream();
                }
            }
            (MicrophoneMode::OnDemand, MicrophoneMode::AlwaysOn) => {
                drop(mode_guard);
                self.start_microphone_stream()?;
            }
            _ => {}
        }

        *safe_lock_err!(self.mode) = new_mode;
        Ok(())
    }

    /* ---------- recording --------------------------------------------------- */

    pub fn try_start_recording(&self, binding_id: &str) -> bool {
        let mut state = safe_lock!(self.state, false);

        if let RecordingState::Idle = *state {
            // Ensure microphone is open in on-demand mode
            let is_on_demand = match self.mode.lock() {
                Ok(mode) => matches!(*mode, MicrophoneMode::OnDemand),
                Err(_) => false,
            };
            if is_on_demand {
                if let Err(e) = self.start_microphone_stream() {
                    error!("Failed to open microphone stream: {e}");
                    return false;
                }
            }

            if let Ok(recorder_guard) = self.recorder.lock() {
                if let Some(rec) = recorder_guard.as_ref() {
                    if rec.start().is_ok() {
                        if let Ok(mut is_rec) = self.is_recording.lock() {
                            *is_rec = true;
                        }
                        *state = RecordingState::Recording {
                            binding_id: binding_id.to_string(),
                        };
                        debug!("Recording started for binding {binding_id}");
                        return true;
                    }
                }
            }
            error!("Recorder not available");
            false
        } else {
            false
        }
    }

    pub fn update_selected_device(&self) -> Result<(), anyhow::Error> {
        // If currently open, restart the microphone stream to use the new device
        let is_open = match self.is_open.lock() {
            Ok(guard) => *guard,
            Err(e) => return Err(anyhow::anyhow!("Failed to check is_open: {}", e)),
        };
        if is_open {
            self.stop_microphone_stream();
            self.start_microphone_stream()?;
        }
        Ok(())
    }

    pub fn stop_recording(&self, binding_id: &str) -> Option<Vec<f32>> {
        let mut state = safe_lock!(self.state, None);

        match *state {
            RecordingState::Recording {
                binding_id: ref active,
            } if active == binding_id => {
                *state = RecordingState::Idle;
                drop(state);

                let samples = if let Ok(recorder_guard) = self.recorder.lock() {
                    if let Some(rec) = recorder_guard.as_ref() {
                        match rec.stop() {
                            Ok(buf) => buf,
                            Err(e) => {
                                error!("stop() failed: {e}");
                                Vec::new()
                            }
                        }
                    } else {
                        error!("Recorder not available");
                        Vec::new()
                    }
                } else {
                    error!("Failed to lock recorder");
                    Vec::new()
                };

                if let Ok(mut is_rec) = self.is_recording.lock() {
                    *is_rec = false;
                }

                // In on-demand mode turn the mic off again
                let is_on_demand = match self.mode.lock() {
                    Ok(mode) => matches!(*mode, MicrophoneMode::OnDemand),
                    Err(_) => false,
                };
                if is_on_demand {
                    self.stop_microphone_stream();
                }

                // Pad if very short
                let s_len = samples.len();
                // debug!("Got {} samples", s_len);
                if s_len < WHISPER_SAMPLE_RATE && s_len > 0 {
                    let mut padded = samples;
                    padded.resize(WHISPER_SAMPLE_RATE * 5 / 4, 0.0);
                    Some(padded)
                } else {
                    Some(samples)
                }
            }
            _ => None,
        }
    }
    pub fn is_recording(&self) -> bool {
        match self.state.lock() {
            Ok(state) => matches!(*state, RecordingState::Recording { .. }),
            Err(_) => false,
        }
    }

    /// Cancel any ongoing recording without returning audio samples
    pub fn cancel_recording(&self) {
        let mut state = safe_lock!(self.state);

        if let RecordingState::Recording { .. } = *state {
            *state = RecordingState::Idle;
            drop(state);

            if let Ok(recorder_guard) = self.recorder.lock() {
                if let Some(rec) = recorder_guard.as_ref() {
                    let _ = rec.stop(); // Discard the result
                }
            }

            if let Ok(mut is_rec) = self.is_recording.lock() {
                *is_rec = false;
            }

            // In on-demand mode turn the mic off again
            let is_on_demand = match self.mode.lock() {
                Ok(mode) => matches!(*mode, MicrophoneMode::OnDemand),
                Err(_) => false,
            };
            if is_on_demand {
                self.stop_microphone_stream();
            }
        }
    }

    /* ---------- active listening -------------------------------------------- */

    /// Start active listening mode with a callback for audio samples
    pub fn start_active_listening(
        &self,
        callback: ActiveListeningCallback,
    ) -> Result<(), anyhow::Error> {
        // Check if we're already in active listening mode
        {
            let mode = safe_lock_err!(self.mode);
            if *mode == MicrophoneMode::ActiveListening {
                debug!("Already in active listening mode");
                return Ok(());
            }
        }

        // Stop and close any existing microphone stream to recreate with callback
        self.stop_microphone_stream();

        // Clear existing recorder so it will be recreated with the callback
        {
            let mut recorder_opt = safe_lock_err!(self.recorder);
            *recorder_opt = None;
        }

        // Set the callback
        {
            let mut cb = safe_lock_err!(self.active_listening_callback);
            *cb = Some(callback);
        }

        // Update mode (must be set before start_microphone_stream so callback gets wired)
        {
            let mut mode = safe_lock_err!(self.mode);
            *mode = MicrophoneMode::ActiveListening;
        }

        // Start microphone stream (will create recorder with callback)
        self.start_microphone_stream()?;

        // Start "recording" in streaming mode (continuous capture)
        {
            let recorder_guard = safe_lock_err!(self.recorder);
            if let Some(rec) = recorder_guard.as_ref() {
                rec.start()
                    .map_err(|e| anyhow::anyhow!("Failed to start recording: {}", e))?;
                let mut is_rec = safe_lock_err!(self.is_recording);
                *is_rec = true;
            }
        }

        info!("Active listening started");
        Ok(())
    }

    /// Stop active listening mode
    pub fn stop_active_listening(&self) -> Result<(), anyhow::Error> {
        // Check if we're in active listening mode
        {
            let mode = safe_lock_err!(self.mode);
            if *mode != MicrophoneMode::ActiveListening {
                debug!("Not in active listening mode");
                return Ok(());
            }
        }

        // Stop recording
        if let Ok(recorder_guard) = self.recorder.lock() {
            if let Some(rec) = recorder_guard.as_ref() {
                let _ = rec.stop();
            }
        }
        if let Ok(mut is_rec) = self.is_recording.lock() {
            *is_rec = false;
        }

        // Stop microphone stream
        self.stop_microphone_stream();

        // Clear the recorder so it gets recreated without the callback
        {
            let mut recorder_opt = safe_lock_err!(self.recorder);
            *recorder_opt = None;
        }

        // Clear callback
        {
            let mut cb = safe_lock_err!(self.active_listening_callback);
            *cb = None;
        }

        // Restore previous mode based on settings
        let settings = get_settings(&self.app_handle);
        {
            let mut mode = safe_lock_err!(self.mode);
            *mode = if settings.always_on_microphone {
                MicrophoneMode::AlwaysOn
            } else {
                MicrophoneMode::OnDemand
            };
        }

        // If was AlwaysOn, restart the stream
        if settings.always_on_microphone {
            self.start_microphone_stream()?;
        }

        info!("Active listening stopped");
        Ok(())
    }

    /// Check if active listening mode is enabled
    pub fn is_active_listening(&self) -> bool {
        match self.mode.lock() {
            Ok(mode) => *mode == MicrophoneMode::ActiveListening,
            Err(_) => false,
        }
    }

    /// Forward samples via the callback. Note: With the new architecture,
    /// samples are forwarded directly via the sample_callback in AudioRecorder.
    /// This method is kept for potential direct forwarding use cases.
    #[allow(dead_code)]
    pub fn forward_active_listening_samples(&self, samples: &[f32]) {
        let is_active = match self.mode.lock() {
            Ok(mode) => *mode == MicrophoneMode::ActiveListening,
            Err(_) => return,
        };
        if !is_active {
            return;
        }

        if let Ok(cb_guard) = self.active_listening_callback.lock() {
            if let Some(ref callback) = *cb_guard {
                callback(samples);
            }
        }
    }
}
