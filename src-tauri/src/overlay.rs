use crate::input;
use crate::settings;
use crate::settings::OverlayPosition;
use log::debug;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize};

#[cfg(not(target_os = "macos"))]
use tauri::WebviewWindowBuilder;

#[cfg(target_os = "macos")]
use tauri::WebviewUrl;

#[cfg(target_os = "macos")]
use tauri_nspanel::{tauri_panel, CollectionBehavior, PanelBuilder, PanelLevel};

#[cfg(target_os = "macos")]
tauri_panel! {
    panel!(RecordingOverlayPanel {
        config: {
            can_become_key_window: false,
            is_floating_panel: true
        }
    })
}

const OVERLAY_WIDTH: f64 = 172.0;
const OVERLAY_HEIGHT: f64 = 36.0;

// Ask AI response overlay dimensions (defaults)
const ASK_AI_RESPONSE_WIDTH: f64 = 400.0;
const ASK_AI_RESPONSE_HEIGHT: f64 = 300.0;

// Ask AI response overlay size constraints
const ASK_AI_MIN_WIDTH: f64 = 320.0;
const ASK_AI_MIN_HEIGHT: f64 = 200.0;
const ASK_AI_MAX_WIDTH: f64 = 800.0;
const ASK_AI_MAX_HEIGHT: f64 = 600.0;

#[cfg(target_os = "macos")]
const OVERLAY_TOP_OFFSET: f64 = 46.0;
#[cfg(any(target_os = "windows", target_os = "linux"))]
const OVERLAY_TOP_OFFSET: f64 = 4.0;

#[cfg(target_os = "macos")]
const OVERLAY_BOTTOM_OFFSET: f64 = 15.0;

#[cfg(any(target_os = "windows", target_os = "linux"))]
const OVERLAY_BOTTOM_OFFSET: f64 = 40.0;

/// Forces a window to be topmost using Win32 API (Windows only)
/// This is more reliable than Tauri's set_always_on_top which can be overridden
#[cfg(target_os = "windows")]
fn force_overlay_topmost(overlay_window: &tauri::webview::WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
    };

    // Clone because run_on_main_thread takes 'static
    let overlay_clone = overlay_window.clone();

    // Make sure the Win32 call happens on the UI thread
    let _ = overlay_clone.clone().run_on_main_thread(move || {
        if let Ok(hwnd) = overlay_clone.hwnd() {
            unsafe {
                // Force Z-order: make this window topmost without changing size/pos or stealing focus
                let _ = SetWindowPos(
                    hwnd,
                    Some(HWND_TOPMOST),
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
                );
            }
        }
    });
}

/// Excludes or includes a window from screen capture (Windows 10 1903+)
/// When excluded, the window is visible locally but hidden from screen sharing
#[cfg(target_os = "windows")]
pub fn set_screen_capture_excluded(
    overlay_window: &tauri::webview::WebviewWindow,
    excluded: bool,
) {
    use windows::Win32::UI::WindowsAndMessaging::{
        SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
    };

    let overlay_clone = overlay_window.clone();

    let _ = overlay_clone.clone().run_on_main_thread(move || {
        if let Ok(hwnd) = overlay_clone.hwnd() {
            let affinity = if excluded {
                WDA_EXCLUDEFROMCAPTURE
            } else {
                WDA_NONE
            };
            let result = unsafe { SetWindowDisplayAffinity(hwnd, affinity) };
            if result.is_err() {
                debug!(
                    "Failed to set window display affinity (excluded={}): {:?}",
                    excluded, result
                );
            } else {
                debug!("Window display affinity set (excluded={})", excluded);
            }
        }
    });
}

/// Excludes or includes a window from screen capture (macOS)
/// Uses NSWindow.sharingType property
#[cfg(target_os = "macos")]
pub fn set_screen_capture_excluded(
    overlay_window: &tauri::webview::WebviewWindow,
    excluded: bool,
) {
    use objc2::msg_send;
    use objc2::runtime::AnyObject;

    if let Ok(ns_window) = overlay_window.ns_window() {
        let ns_window = ns_window as *mut AnyObject;
        // NSWindowSharingType: .none = 0, .readOnly = 1, .readWrite = 2
        // We use .none (0) to exclude from capture, .readOnly (1) to include
        let sharing_type: i64 = if excluded { 0 } else { 1 };
        unsafe {
            let _: () = msg_send![ns_window, setSharingType: sharing_type];
        }
        debug!(
            "macOS window sharing type set (excluded={}, sharingType={})",
            excluded, sharing_type
        );
    }
}

/// Linux: No standard API for screen capture exclusion
/// Log a debug message and do nothing
#[cfg(target_os = "linux")]
pub fn set_screen_capture_excluded(
    _overlay_window: &tauri::webview::WebviewWindow,
    excluded: bool,
) {
    if excluded {
        debug!("Screen capture exclusion is not supported on Linux");
    }
}

fn get_monitor_with_cursor(app_handle: &AppHandle) -> Option<tauri::Monitor> {
    if let Some(mouse_location) = input::get_cursor_position(app_handle) {
        if let Ok(monitors) = app_handle.available_monitors() {
            for monitor in monitors {
                let is_within =
                    is_mouse_within_monitor(mouse_location, monitor.position(), monitor.size());
                if is_within {
                    return Some(monitor);
                }
            }
        }
    }

    app_handle.primary_monitor().ok().flatten()
}

fn is_mouse_within_monitor(
    mouse_pos: (i32, i32),
    monitor_pos: &PhysicalPosition<i32>,
    monitor_size: &PhysicalSize<u32>,
) -> bool {
    let (mouse_x, mouse_y) = mouse_pos;
    let PhysicalPosition {
        x: monitor_x,
        y: monitor_y,
    } = *monitor_pos;
    let PhysicalSize {
        width: monitor_width,
        height: monitor_height,
    } = *monitor_size;

    mouse_x >= monitor_x
        && mouse_x < (monitor_x + monitor_width as i32)
        && mouse_y >= monitor_y
        && mouse_y < (monitor_y + monitor_height as i32)
}

fn calculate_overlay_position(app_handle: &AppHandle) -> Option<(f64, f64)> {
    if let Some(monitor) = get_monitor_with_cursor(app_handle) {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();
        let work_area_width = work_area.size.width as f64 / scale;
        let work_area_height = work_area.size.height as f64 / scale;
        let work_area_x = work_area.position.x as f64 / scale;
        let work_area_y = work_area.position.y as f64 / scale;

        let settings = settings::get_settings(app_handle);

        let x = work_area_x + (work_area_width - OVERLAY_WIDTH) / 2.0;
        let y = match settings.overlay_position {
            OverlayPosition::Top => work_area_y + OVERLAY_TOP_OFFSET,
            OverlayPosition::Bottom | OverlayPosition::None => {
                // don't subtract the overlay height it puts it too far up
                work_area_y + work_area_height - OVERLAY_BOTTOM_OFFSET
            }
        };

        return Some((x, y));
    }
    None
}

/// Creates the recording overlay window and keeps it hidden by default
#[cfg(not(target_os = "macos"))]
pub fn create_recording_overlay(app_handle: &AppHandle) {
    if let Some((x, y)) = calculate_overlay_position(app_handle) {
        match WebviewWindowBuilder::new(
            app_handle,
            "recording_overlay",
            tauri::WebviewUrl::App("src/overlay/index.html".into()),
        )
        .title("Recording")
        .position(x, y)
        .resizable(false)
        .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
        .shadow(false)
        .maximizable(false)
        .minimizable(false)
        .closable(false)
        .accept_first_mouse(true)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)
        .focused(false)
        .visible(false)
        .build()
        {
            Ok(window) => {
                debug!("Recording overlay window created successfully (hidden)");

                // Apply private overlay setting (exclude from screen capture)
                let current_settings = settings::get_settings(app_handle);
                if current_settings.general.private_overlay {
                    set_screen_capture_excluded(&window, true);
                }
            }
            Err(e) => {
                debug!("Failed to create recording overlay window: {}", e);
            }
        }
    }
}

/// Creates the recording overlay panel and keeps it hidden by default (macOS)
#[cfg(target_os = "macos")]
pub fn create_recording_overlay(app_handle: &AppHandle) {
    if let Some((x, y)) = calculate_overlay_position(app_handle) {
        // PanelBuilder creates a Tauri window then converts it to NSPanel.
        // The window remains registered, so get_webview_window() still works.
        match PanelBuilder::<_, RecordingOverlayPanel>::new(app_handle, "recording_overlay")
            .url(WebviewUrl::App("src/overlay/index.html".into()))
            .title("Recording")
            .position(tauri::Position::Logical(tauri::LogicalPosition { x, y }))
            .level(PanelLevel::Status)
            .size(tauri::Size::Logical(tauri::LogicalSize {
                width: OVERLAY_WIDTH,
                height: OVERLAY_HEIGHT,
            }))
            .has_shadow(false)
            .transparent(true)
            .no_activate(true)
            .corner_radius(0.0)
            .with_window(|w| w.decorations(false).transparent(true))
            .collection_behavior(
                CollectionBehavior::new()
                    .can_join_all_spaces()
                    .full_screen_auxiliary(),
            )
            .build()
        {
            Ok(panel) => {
                let _ = panel.hide();

                // Apply private overlay setting (exclude from screen capture)
                if let Some(window) = app_handle.get_webview_window("recording_overlay") {
                    let current_settings = settings::get_settings(app_handle);
                    if current_settings.general.private_overlay {
                        set_screen_capture_excluded(&window, true);
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to create recording overlay panel: {}", e);
            }
        }
    }
}

/// Shows the recording overlay window with fade-in animation
pub fn show_recording_overlay(app_handle: &AppHandle) {
    // Check if overlay should be shown based on position setting
    let settings = settings::get_settings(app_handle);
    if settings.overlay_position == OverlayPosition::None {
        return;
    }

    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Update position before showing to prevent flicker from position changes
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }

        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to trigger fade-in animation with recording state
        let _ = overlay_window.emit("show-overlay", "recording");
    }
}

/// Shows the transcribing overlay window
pub fn show_transcribing_overlay(app_handle: &AppHandle) {
    // Check if overlay should be shown based on position setting
    let settings = settings::get_settings(app_handle);
    if settings.overlay_position == OverlayPosition::None {
        return;
    }

    update_overlay_position(app_handle);

    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to switch to transcribing state
        let _ = overlay_window.emit("show-overlay", "transcribing");
    }
}

/// Updates the overlay window position based on current settings
pub fn update_overlay_position(app_handle: &AppHandle) {
    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }
    }
}

/// Hides the recording overlay window with fade-out animation
pub fn hide_recording_overlay(app_handle: &AppHandle) {
    // Always hide the overlay regardless of settings - if setting was changed while recording,
    // we still want to hide it properly
    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Emit event to trigger fade-out animation
        let _ = overlay_window.emit("hide-overlay", ());
        // Hide the window after a short delay to allow animation to complete
        let window_clone = overlay_window.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            let _ = window_clone.hide();
        });
    }
}

pub fn emit_levels(app_handle: &AppHandle, levels: &Vec<f32>) {
    // emit levels to main app
    let _ = app_handle.emit("mic-level", levels);

    // also emit to the recording overlay if it's open
    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        let _ = overlay_window.emit("mic-level", levels);
    }
}

/// Shows the Ask AI recording overlay window (uses same overlay as transcribe)
pub fn show_ask_ai_overlay(app_handle: &AppHandle) {
    // Check if overlay should be shown based on position setting
    let settings = settings::get_settings(app_handle);
    if settings.overlay_position == OverlayPosition::None {
        return;
    }

    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Update position before showing to prevent flicker from position changes
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }

        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to trigger fade-in animation with ask-ai recording state
        let _ = overlay_window.emit("show-overlay", "ask-ai-recording");
    }
}

/// Shows the Ask AI transcribing overlay window
pub fn show_ask_ai_transcribing_overlay(app_handle: &AppHandle) {
    // Check if overlay should be shown based on position setting
    let settings = settings::get_settings(app_handle);
    if settings.overlay_position == OverlayPosition::None {
        return;
    }

    update_overlay_position(app_handle);

    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to switch to ask-ai transcribing state
        let _ = overlay_window.emit("show-overlay", "ask-ai-transcribing");
    }
}

/// Shows the active listening overlay window
pub fn show_active_listening_overlay(app_handle: &AppHandle) {
    // Check if overlay should be shown based on position setting
    let settings = settings::get_settings(app_handle);
    if settings.overlay_position == OverlayPosition::None {
        return;
    }

    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Update position before showing to prevent flicker from position changes
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }

        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to trigger fade-in animation with active-listening state
        let _ = overlay_window.emit("show-overlay", "active-listening");
    }
}

/// Calculates the center position for the Ask AI response overlay
fn calculate_ask_ai_response_position(app_handle: &AppHandle) -> Option<(f64, f64)> {
    if let Some(monitor) = get_monitor_with_cursor(app_handle) {
        let work_area = monitor.work_area();
        let scale = monitor.scale_factor();
        let work_area_width = work_area.size.width as f64 / scale;
        let work_area_height = work_area.size.height as f64 / scale;
        let work_area_x = work_area.position.x as f64 / scale;
        let work_area_y = work_area.position.y as f64 / scale;

        let x = work_area_x + (work_area_width - ASK_AI_RESPONSE_WIDTH) / 2.0;
        let y = work_area_y + (work_area_height - ASK_AI_RESPONSE_HEIGHT) / 2.0;

        return Some((x, y));
    }
    None
}

/// Shows the Ask AI response overlay with expanded size
pub fn show_ask_ai_response_overlay(app_handle: &AppHandle) {
    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Get saved window bounds or use defaults
        let settings = settings::get_settings(app_handle);
        let width = settings
            .ask_ai
            .window_width
            .unwrap_or(ASK_AI_RESPONSE_WIDTH)
            .clamp(ASK_AI_MIN_WIDTH, ASK_AI_MAX_WIDTH);
        let height = settings
            .ask_ai
            .window_height
            .unwrap_or(ASK_AI_RESPONSE_HEIGHT)
            .clamp(ASK_AI_MIN_HEIGHT, ASK_AI_MAX_HEIGHT);

        // Calculate centered position or use saved position
        let (x, y) = if let (Some(saved_x), Some(saved_y)) =
            (settings.ask_ai.window_x, settings.ask_ai.window_y)
        {
            (saved_x, saved_y)
        } else if let Some((calc_x, calc_y)) = calculate_ask_ai_response_position(app_handle) {
            (calc_x, calc_y)
        } else {
            // Fallback to defaults
            (100.0, 100.0)
        };

        // Resize the overlay for Ask AI response display
        let _ = overlay_window.set_size(tauri::Size::Logical(tauri::LogicalSize { width, height }));

        // Set min/max size constraints
        let _ = overlay_window.set_min_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: ASK_AI_MIN_WIDTH,
            height: ASK_AI_MIN_HEIGHT,
        })));
        let _ = overlay_window.set_max_size(Some(tauri::Size::Logical(tauri::LogicalSize {
            width: ASK_AI_MAX_WIDTH,
            height: ASK_AI_MAX_HEIGHT,
        })));

        // Enable resizing for Ask AI mode
        let _ = overlay_window.set_resizable(true);

        // Reposition
        let _ =
            overlay_window.set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));

        let _ = overlay_window.show();

        // On Windows, aggressively re-assert "topmost" in the native Z-order after showing
        #[cfg(target_os = "windows")]
        force_overlay_topmost(&overlay_window);

        // Emit event to show Ask AI generating state
        let _ = overlay_window.emit("show-overlay", "ask-ai-generating");
    }
}

/// Resets the overlay to its default size and hides it
pub fn reset_overlay_size(app_handle: &AppHandle) {
    if let Some(overlay_window) = app_handle.get_webview_window("recording_overlay") {
        // Disable resizing for normal overlay mode
        let _ = overlay_window.set_resizable(false);

        // Clear min/max size constraints
        let _ = overlay_window.set_min_size(None::<tauri::Size>);
        let _ = overlay_window.set_max_size(None::<tauri::Size>);

        // Reset to default overlay size
        let _ = overlay_window.set_size(tauri::Size::Logical(tauri::LogicalSize {
            width: OVERLAY_WIDTH,
            height: OVERLAY_HEIGHT,
        }));

        // Reset position to default
        if let Some((x, y)) = calculate_overlay_position(app_handle) {
            let _ = overlay_window
                .set_position(tauri::Position::Logical(tauri::LogicalPosition { x, y }));
        }
    }
}
