use crate::llm_client::send_vision_chat_completion;
use crate::settings::active_listening::LlmProvider;
use crate::settings::screen_vision::ScreenVisionSettings;
use crate::settings::{get_settings, write_settings, PostProcessProvider};
use crate::telemetry::{TelemetryEventBuilder, TelemetryLevel};
use chrono::Utc;
use log::{debug, error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{oneshot, Mutex};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ScreenVisionSnapshotResult {
    pub timestamp_ms: i64,
    pub capture_path: Option<String>,
    pub analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ScreenVisionStatus {
    pub running: bool,
    pub session_id: Option<String>,
    pub started_at_ms: Option<i64>,
    pub expires_at_ms: Option<i64>,
    pub snapshots_processed: u32,
    pub last_result: Option<ScreenVisionSnapshotResult>,
    pub last_error: Option<String>,
}

impl Default for ScreenVisionStatus {
    fn default() -> Self {
        Self {
            running: false,
            session_id: None,
            started_at_ms: None,
            expires_at_ms: None,
            snapshots_processed: 0,
            last_result: None,
            last_error: None,
        }
    }
}

struct ScreenVisionRuntime {
    status: ScreenVisionStatus,
    cancel_tx: Option<oneshot::Sender<()>>,
}

static SCREEN_VISION_RUNTIME: Lazy<Arc<Mutex<ScreenVisionRuntime>>> = Lazy::new(|| {
    Arc::new(Mutex::new(ScreenVisionRuntime {
        status: ScreenVisionStatus::default(),
        cancel_tx: None,
    }))
});

fn cloud_base_url(provider: LlmProvider, custom_base_url: Option<&String>) -> String {
    match provider {
        LlmProvider::Custom => custom_base_url
            .cloned()
            .unwrap_or_else(|| "https://openrouter.ai/api/v1".to_string()),
        LlmProvider::LocalOpenAi => custom_base_url
            .cloned()
            .unwrap_or_else(|| "http://127.0.0.1:8000/v1".to_string()),
        LlmProvider::Ollama => "http://localhost:11434/v1".to_string(),
        _ => provider
            .default_base_url()
            .unwrap_or("https://openrouter.ai/api/v1")
            .to_string(),
    }
}

fn to_provider_id(provider: LlmProvider) -> &'static str {
    match provider {
        LlmProvider::Ollama => "ollama",
        LlmProvider::LocalOpenAi => "local_open_ai",
        LlmProvider::OpenRouter => "openrouter",
        LlmProvider::OpenAi => "openai",
        LlmProvider::Groq => "groq",
        LlmProvider::Together => "together",
        LlmProvider::Fireworks => "fireworks",
        LlmProvider::Custom => "custom",
    }
}

fn build_provider(
    #[allow(unused_variables)] shared_provider: LlmProvider,
    shared_base_url: Option<&String>,
) -> PostProcessProvider {
    #[cfg(target_os = "macos")]
    let effective_provider = LlmProvider::OpenRouter;
    #[cfg(not(target_os = "macos"))]
    let effective_provider = shared_provider;

    PostProcessProvider {
        id: to_provider_id(effective_provider).to_string(),
        label: "Screen Vision".to_string(),
        base_url: cloud_base_url(effective_provider, shared_base_url),
        allow_base_url_edit: true,
        models_endpoint: Some("/models".to_string()),
        supports_structured_output: false,
    }
}

fn normalize_screen_vision_settings(mut settings: ScreenVisionSettings) -> ScreenVisionSettings {
    #[cfg(target_os = "macos")]
    {
        settings.llm_provider = LlmProvider::OpenRouter;
        if settings
            .llm_base_url
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            settings.llm_base_url = Some("https://openrouter.ai/api/v1".to_string());
        }
        if settings.llm_model.trim().is_empty() {
            settings.llm_model = "qwen/qwen2.5-vl-72b-instruct:free".to_string();
        }
    }
    settings
}

#[cfg(target_os = "macos")]
fn capture_screen_png() -> Result<(Vec<u8>, String), String> {
    let file_path = std::env::temp_dir().join(format!("handy-screen-{}.png", Uuid::new_v4()));
    let output = std::process::Command::new("screencapture")
        .arg("-x")
        .arg("-t")
        .arg("png")
        .arg(&file_path)
        .output()
        .map_err(|e| format!("Failed to run screencapture: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("screencapture failed: {}", stderr));
    }

    let bytes =
        std::fs::read(&file_path).map_err(|e| format!("Failed to read screenshot file: {}", e))?;
    Ok((bytes, file_path.to_string_lossy().to_string()))
}

#[cfg(target_os = "windows")]
fn capture_screen_png() -> Result<(Vec<u8>, String), String> {
    let file_path = std::env::temp_dir().join(format!("handy-screen-{}.png", Uuid::new_v4()));
    let script = build_windows_capture_script(&file_path.to_string_lossy());
    let output = std::process::Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()
        .map_err(|e| format!("Failed to run PowerShell screenshot capture: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("PowerShell screen capture failed: {}", stderr));
    }

    let bytes =
        std::fs::read(&file_path).map_err(|e| format!("Failed to read screenshot file: {}", e))?;
    Ok((bytes, file_path.to_string_lossy().to_string()))
}

#[cfg(target_os = "windows")]
fn build_windows_capture_script(path: &str) -> String {
    let escaped = path.replace('\'', "''");
    format!(
        r#"
$path = '{escaped}'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
$graphics = [System.Drawing.Graphics]::FromImage($bmp)
$graphics.CopyFromScreen($bounds.Location, [System.Drawing.Point]::Empty, $bounds.Size)
$bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
$graphics.Dispose()
$bmp.Dispose()
"#
    )
}

#[cfg(target_os = "linux")]
fn capture_screen_png() -> Result<(Vec<u8>, String), String> {
    let file_path = std::env::temp_dir().join(format!("handy-screen-{}.png", Uuid::new_v4()));
    let file_path_str = file_path.to_string_lossy().to_string();

    let candidates: [(&str, Vec<&str>); 4] = [
        ("grim", vec![&file_path_str]),
        ("gnome-screenshot", vec!["-f", &file_path_str]),
        ("import", vec!["-window", "root", &file_path_str]),
        ("spectacle", vec!["-b", "-n", "-o", &file_path_str]),
    ];

    let mut last_error = String::new();
    for (bin, args) in candidates {
        match std::process::Command::new(bin).args(&args).output() {
            Ok(output) => {
                if output.status.success() && file_path.exists() {
                    let bytes = std::fs::read(&file_path)
                        .map_err(|e| format!("Failed to read screenshot file: {}", e))?;
                    return Ok((bytes, file_path_str));
                }
                last_error = format!(
                    "{} exited with status {}: {}",
                    bin,
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            Err(err) => {
                last_error = format!("{} unavailable: {}", bin, err);
            }
        }
    }

    Err(format!(
        "No supported Linux screenshot tool succeeded (tried grim, gnome-screenshot, import, spectacle). Last error: {}",
        last_error
    ))
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
fn capture_screen_png() -> Result<(Vec<u8>, String), String> {
    Err("Screen capture is not implemented on this platform".to_string())
}

async fn run_single_snapshot(
    app: &AppHandle,
    settings: &ScreenVisionSettings,
) -> Result<ScreenVisionSnapshotResult, String> {
    let app_settings = get_settings(app);
    let shared = app_settings.active_listening;
    let started = std::time::Instant::now();

    let (image_bytes, capture_path) = capture_screen_png()?;
    let provider = build_provider(shared.llm_provider, shared.llm_base_url.as_ref());
    let model = if settings.llm_model.trim().is_empty() {
        match shared.llm_provider {
            LlmProvider::Ollama => shared.ollama_model,
            _ => shared.llm_model.unwrap_or_default(),
        }
    } else {
        settings.llm_model.clone()
    };

    let model = model.trim();
    if model.is_empty() {
        TelemetryEventBuilder::new("screen_vision", "snapshot_failed")
            .level(TelemetryLevel::Warn)
            .message("Screen vision snapshot aborted because no model is configured")
            .status("missing_model")
            .emit();
        return Err("Screen Vision model is empty".to_string());
    }

    TelemetryEventBuilder::new("screen_vision", "snapshot_started")
        .message("Screen vision snapshot started")
        .attr("provider_id", provider.id.clone())
        .attr("model", model)
        .attr("image_bytes", image_bytes.len())
        .emit();

    let analysis = send_vision_chat_completion(
        &provider,
        shared
            .llm_api_key
            .or(settings.llm_api_key.clone())
            .unwrap_or_default(),
        model,
        settings.prompt.clone(),
        &image_bytes,
    )
    .await?
    .unwrap_or_else(|| "No analysis returned by model".to_string());

    TelemetryEventBuilder::new("screen_vision", "snapshot_completed")
        .message("Screen vision snapshot completed")
        .attr("provider_id", provider.id)
        .attr("model", model)
        .duration_ms(started.elapsed().as_millis() as u64)
        .attr("analysis_chars", analysis.len())
        .emit();

    Ok(ScreenVisionSnapshotResult {
        timestamp_ms: Utc::now().timestamp_millis(),
        capture_path: Some(capture_path),
        analysis,
    })
}

fn emit_status(app: &AppHandle, status: &ScreenVisionStatus) {
    if let Err(e) = app.emit("screen-vision-status", status.clone()) {
        debug!("Failed to emit screen vision status event: {}", e);
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_screen_vision_settings(app: AppHandle) -> Result<ScreenVisionSettings, String> {
    Ok(normalize_screen_vision_settings(
        get_settings(&app).screen_vision,
    ))
}

#[tauri::command]
#[specta::specta]
pub fn update_screen_vision_settings(
    app: AppHandle,
    settings: ScreenVisionSettings,
) -> Result<(), String> {
    let mut app_settings = get_settings(&app);
    app_settings.screen_vision = normalize_screen_vision_settings(settings);
    write_settings(&app, app_settings);
    TelemetryEventBuilder::new("screen_vision", "settings_updated")
        .message("Screen vision settings updated")
        .emit();
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_screen_vision_status() -> Result<ScreenVisionStatus, String> {
    let runtime = SCREEN_VISION_RUNTIME.lock().await;
    Ok(runtime.status.clone())
}

#[tauri::command]
#[specta::specta]
pub async fn test_screen_vision_once(app: AppHandle) -> Result<ScreenVisionSnapshotResult, String> {
    let settings = normalize_screen_vision_settings(get_settings(&app).screen_vision);
    TelemetryEventBuilder::new("screen_vision", "test_once_requested")
        .message("One-off screen vision analysis requested")
        .emit();
    run_single_snapshot(&app, &settings).await
}

#[tauri::command]
#[specta::specta]
pub async fn start_screen_vision_session(app: AppHandle) -> Result<String, String> {
    let settings = normalize_screen_vision_settings(get_settings(&app).screen_vision);
    let session_id = Uuid::new_v4().to_string();
    let started_at = Utc::now().timestamp_millis();
    let expires_at = started_at + (settings.duration_seconds as i64 * 1000);

    if settings.interval_seconds == 0 {
        return Err("Interval must be greater than 0".to_string());
    }

    {
        let mut runtime = SCREEN_VISION_RUNTIME.lock().await;
        if runtime.status.running {
            TelemetryEventBuilder::new("screen_vision", "session_start_rejected")
                .level(TelemetryLevel::Warn)
                .message("Screen vision session start rejected because another session is active")
                .emit();
            return Err("Screen vision session already running".to_string());
        }
        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        runtime.cancel_tx = Some(cancel_tx);
        runtime.status = ScreenVisionStatus {
            running: true,
            session_id: Some(session_id.clone()),
            started_at_ms: Some(started_at),
            expires_at_ms: Some(expires_at),
            snapshots_processed: 0,
            last_result: None,
            last_error: None,
        };
        emit_status(&app, &runtime.status);

        let app_handle = app.clone();
        let settings_clone = settings.clone();
        tokio::spawn(async move {
            run_session_loop(app_handle, settings_clone, cancel_rx).await;
        });
    }

    info!("Started screen vision session: {}", session_id);
    TelemetryEventBuilder::new("screen_vision", "session_started")
        .message("Screen vision session started")
        .session_id(session_id.clone())
        .attr("interval_seconds", settings.interval_seconds)
        .attr("duration_seconds", settings.duration_seconds)
        .attr("max_snapshots", settings.max_snapshots)
        .emit();
    Ok(session_id)
}

async fn run_session_loop(
    app: AppHandle,
    settings: ScreenVisionSettings,
    mut cancel_rx: oneshot::Receiver<()>,
) {
    let started_at = Utc::now().timestamp_millis();
    let max_duration_ms = (settings.duration_seconds as i64) * 1000;
    let mut snapshots_taken: u32 = 0;

    loop {
        let now_ms = Utc::now().timestamp_millis();
        if now_ms - started_at >= max_duration_ms || snapshots_taken >= settings.max_snapshots {
            break;
        }

        let snapshot_result = run_single_snapshot(&app, &settings).await;
        let mut runtime = SCREEN_VISION_RUNTIME.lock().await;
        match snapshot_result {
            Ok(result) => {
                snapshots_taken += 1;
                runtime.status.snapshots_processed = snapshots_taken;
                runtime.status.last_result = Some(result);
                runtime.status.last_error = None;
                TelemetryEventBuilder::new("screen_vision", "session_snapshot_completed")
                    .message("Screen vision session snapshot completed")
                    .session_id(runtime.status.session_id.clone().unwrap_or_default())
                    .attr("snapshots_processed", snapshots_taken)
                    .emit();
            }
            Err(err) => {
                error!("Screen vision snapshot failed: {}", err);
                runtime.status.last_error = Some(err);
                TelemetryEventBuilder::new("screen_vision", "session_snapshot_failed")
                    .level(TelemetryLevel::Error)
                    .message("Screen vision session snapshot failed")
                    .session_id(runtime.status.session_id.clone().unwrap_or_default())
                    .attr("error", runtime.status.last_error.clone().unwrap_or_default())
                    .emit();
            }
        }
        emit_status(&app, &runtime.status);
        drop(runtime);

        let wait = sleep(Duration::from_secs(settings.interval_seconds));
        tokio::pin!(wait);
        tokio::select! {
            _ = &mut cancel_rx => {
                break;
            }
            _ = &mut wait => {}
        }
    }

    let mut runtime = SCREEN_VISION_RUNTIME.lock().await;
    runtime.status.running = false;
    runtime.cancel_tx = None;
    emit_status(&app, &runtime.status);
    info!("Screen vision session finished");
    TelemetryEventBuilder::new("screen_vision", "session_finished")
        .message("Screen vision session finished")
        .session_id(runtime.status.session_id.clone().unwrap_or_default())
        .attr("snapshots_processed", runtime.status.snapshots_processed)
        .emit();
}

#[tauri::command]
#[specta::specta]
pub async fn stop_screen_vision_session() -> Result<(), String> {
    let mut runtime = SCREEN_VISION_RUNTIME.lock().await;
    if let Some(cancel_tx) = runtime.cancel_tx.take() {
        let _ = cancel_tx.send(());
    }
    runtime.status.running = false;
    TelemetryEventBuilder::new("screen_vision", "session_stop_requested")
        .message("Screen vision session stop requested")
        .session_id(runtime.status.session_id.clone().unwrap_or_default())
        .emit();
    Ok(())
}

#[cfg(all(test, target_os = "windows"))]
mod windows_capture_tests {
    use super::build_windows_capture_script;

    #[test]
    fn script_escapes_single_quotes() {
        let script = build_windows_capture_script("C:\\Users\\O'Hara\\screen.png");
        assert!(script.contains("O''Hara"));
        assert!(script.contains("$bmp.Save"));
    }
}
