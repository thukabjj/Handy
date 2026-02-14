mod actions;
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod apple_intelligence;
mod audio_feedback;
pub mod audio_toolkit;
mod clipboard;
mod commands;
pub mod error;
mod helpers;
mod input;
mod llm_client;
mod managers;
mod ollama_client;
mod overlay;
mod settings;
mod shortcut;
mod signal_handle;
mod tray;
mod tray_i18n;
mod utils;
use specta_typescript::{BigIntExportBehavior, Typescript};
use tauri_specta::{collect_commands, Builder};

use env_filter::Builder as EnvFilterBuilder;
use managers::active_listening::ActiveListeningManager;
use managers::ask_ai::AskAiManager;
use managers::ask_ai_history::AskAiHistoryManager;
use managers::audio::AudioRecordingManager;
use managers::batch_processor::BatchProcessor;
use managers::history::HistoryManager;
use managers::model::ModelManager;
use managers::rag::RagManager;
use managers::suggestion_engine::SuggestionEngine;
use managers::task_extractor::TaskExtractor;
use managers::transcription::TranscriptionManager;
use managers::vocabulary::VocabularyManager;
#[cfg(unix)]
use signal_hook::consts::SIGUSR2;
#[cfg(unix)]
use signal_hook::iterator::Signals;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use tauri::image::Image;

use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_log::{Builder as LogBuilder, RotationStrategy, Target, TargetKind};

use crate::settings::get_settings;

// Global atomic to store the file log level filter
// We use u8 to store the log::LevelFilter as a number
pub static FILE_LOG_LEVEL: AtomicU8 = AtomicU8::new(log::LevelFilter::Debug as u8);

fn level_filter_from_u8(value: u8) -> log::LevelFilter {
    match value {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        5 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    }
}

fn build_console_filter() -> env_filter::Filter {
    let mut builder = EnvFilterBuilder::new();

    match std::env::var("RUST_LOG") {
        Ok(spec) if !spec.trim().is_empty() => {
            if let Err(err) = builder.try_parse(&spec) {
                log::warn!(
                    "Ignoring invalid RUST_LOG value '{}': {}. Falling back to info-level console logging",
                    spec,
                    err
                );
                builder.filter_level(log::LevelFilter::Info);
            }
        }
        _ => {
            builder.filter_level(log::LevelFilter::Info);
        }
    }

    builder.build()
}

#[derive(Default)]
struct ShortcutToggleStates {
    // Map: shortcut_binding_id -> is_active
    active_toggles: HashMap<String, bool>,
}

type ManagedToggleState = Mutex<ShortcutToggleStates>;

fn show_main_window(app: &AppHandle) {
    if let Some(main_window) = app.get_webview_window("main") {
        // First, ensure the window is visible
        if let Err(e) = main_window.show() {
            log::error!("Failed to show window: {}", e);
        }
        // Then, bring it to the front and give it focus
        if let Err(e) = main_window.set_focus() {
            log::error!("Failed to focus window: {}", e);
        }
        // Optional: On macOS, ensure the app becomes active if it was an accessory
        #[cfg(target_os = "macos")]
        {
            if let Err(e) = app.set_activation_policy(tauri::ActivationPolicy::Regular) {
                log::error!("Failed to set activation policy to Regular: {}", e);
            }
        }
    } else {
        log::error!("Main window not found.");
    }
}

fn initialize_core_logic(app_handle: &AppHandle) {
    // Note: Enigo (keyboard/mouse simulation) is NOT initialized here.
    // The frontend is responsible for calling the `initialize_enigo` command
    // after onboarding completes. This avoids triggering permission dialogs
    // on macOS before the user is ready.

    // Initialize the managers
    let recording_manager = Arc::new(
        AudioRecordingManager::new(app_handle).expect("Failed to initialize recording manager"),
    );
    let model_manager =
        Arc::new(ModelManager::new(app_handle).expect("Failed to initialize model manager"));
    let transcription_manager = Arc::new(
        TranscriptionManager::new(app_handle, model_manager.clone())
            .expect("Failed to initialize transcription manager"),
    );
    let history_manager =
        Arc::new(HistoryManager::new(app_handle).expect("Failed to initialize history manager"));
    let active_listening_manager = Arc::new(
        ActiveListeningManager::new(app_handle, transcription_manager.clone())
            .expect("Failed to initialize active listening manager"),
    );
    let ask_ai_manager = Arc::new(
        AskAiManager::new(app_handle, transcription_manager.clone())
            .expect("Failed to initialize ask ai manager"),
    );
    let ask_ai_history_manager = Arc::new(
        AskAiHistoryManager::new(app_handle).expect("Failed to initialize ask ai history manager"),
    );

    // Initialize RAG manager with Ollama client
    let settings = settings::get_settings(app_handle);
    let ollama_base_url = settings.active_listening.ollama_base_url.clone();
    let rag_db_path = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir")
        .join("rag.db");
    let ollama_client = Arc::new(
        ollama_client::OllamaClient::new(&ollama_base_url)
            .expect("Failed to initialize Ollama client for RAG"),
    );
    let rag_manager = Arc::new(
        RagManager::new(rag_db_path, ollama_client.clone()).expect("Failed to initialize RAG manager"),
    );

    // Initialize the Suggestion Engine
    let suggestion_engine = SuggestionEngine::new(
        app_handle,
        Some(rag_manager.clone()),
        ollama_client.clone(),
        settings.suggestions.clone(),
    );

    // Initialize Batch Processor
    let mut batch_processor = BatchProcessor::new();
    batch_processor.set_app_handle(app_handle.clone());

    // Initialize Task Extractor
    let mut task_extractor = TaskExtractor::new();
    task_extractor.set_app_handle(app_handle.clone());

    // Initialize Vocabulary Manager
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    let vocabulary_manager =
        VocabularyManager::new(&app_data_dir).expect("Failed to initialize vocabulary manager");

    // Add managers to Tauri's managed state
    app_handle.manage(recording_manager.clone());
    app_handle.manage(model_manager.clone());
    app_handle.manage(transcription_manager.clone());
    app_handle.manage(history_manager.clone());
    app_handle.manage(active_listening_manager.clone());
    app_handle.manage(ask_ai_manager.clone());
    app_handle.manage(ask_ai_history_manager.clone());
    app_handle.manage(rag_manager.clone());
    app_handle.manage(suggestion_engine);
    app_handle.manage(tokio::sync::Mutex::new(batch_processor));
    app_handle.manage(Mutex::new(task_extractor));
    app_handle.manage(Mutex::new(vocabulary_manager));

    // Initialize Sound Detector
    let mut sound_detector = audio_toolkit::SoundDetector::new();
    let sd_settings = settings::get_settings(app_handle);
    sound_detector.update_settings(&sd_settings.sound_detection);
    app_handle.manage(Mutex::new(sound_detector));

    // Initialize the shortcuts
    shortcut::init_shortcuts(app_handle);

    #[cfg(unix)]
    let signals = Signals::new(&[SIGUSR2]).unwrap();
    // Set up SIGUSR2 signal handler for toggling transcription
    #[cfg(unix)]
    signal_handle::setup_signal_handler(app_handle.clone(), signals);

    // Apply macOS Accessory policy if starting hidden
    #[cfg(target_os = "macos")]
    {
        let settings = settings::get_settings(app_handle);
        if settings.general.start_hidden {
            let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
        }
    }
    // Get the current theme to set the appropriate initial icon
    let initial_theme = tray::get_current_theme(app_handle);

    // Choose the appropriate initial icon based on theme
    let initial_icon_path = tray::get_icon_path(initial_theme, tray::TrayIconState::Idle);

    let tray = TrayIconBuilder::new()
        .icon(
            Image::from_path(
                app_handle
                    .path()
                    .resolve(initial_icon_path, tauri::path::BaseDirectory::Resource)
                    .unwrap(),
            )
            .unwrap(),
        )
        .show_menu_on_left_click(true)
        .icon_as_template(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => {
                show_main_window(app);
            }
            "check_updates" => {
                let settings = settings::get_settings(app);
                if settings.general.update_checks_enabled {
                    show_main_window(app);
                    let _ = app.emit("check-for-updates", ());
                }
            }
            "cancel" => {
                use crate::utils::cancel_current_operation;

                // Use centralized cancellation that handles all operations
                cancel_current_operation(app);
            }
            "start_active_listening" => {
                let al_manager = app.state::<Arc<ActiveListeningManager>>();
                let audio_manager = app.state::<Arc<AudioRecordingManager>>();

                // Check if session is already active
                if al_manager.is_session_active() {
                    log::warn!("Active listening session already in progress, ignoring start request");
                    return;
                }

                // Start session
                match al_manager.start_session(None) {
                    Ok(session_id) => {
                        log::info!("Started active listening session from tray: {}", session_id);

                        // Create callback to forward audio samples
                        let al_manager_clone = al_manager.inner().clone();
                        let callback = std::sync::Arc::new(move |samples: &[f32]| {
                            al_manager_clone.push_audio_samples(samples);
                        });

                        // Start audio
                        if let Err(e) = audio_manager.start_active_listening(callback) {
                            log::error!("Failed to start active listening audio: {}", e);
                            let _ = al_manager.stop_session();
                        } else {
                            // Update tray icon and show overlay
                            utils::change_tray_icon(app, utils::TrayIconState::ActiveListening);
                            utils::show_active_listening_overlay(app);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to start active listening session: {}", e);
                    }
                }
            }
            "stop_active_listening" => {
                let al_manager = app.state::<Arc<ActiveListeningManager>>();
                let audio_manager = app.state::<Arc<AudioRecordingManager>>();

                // Check if session is active before stopping
                if !al_manager.is_session_active() {
                    log::debug!("No active listening session to stop");
                    return;
                }

                // Flush remaining audio
                al_manager.flush_segment();

                // Stop audio
                if let Err(e) = audio_manager.stop_active_listening() {
                    log::error!("Failed to stop active listening audio: {}", e);
                }

                // Stop session
                match al_manager.stop_session() {
                    Ok(Some(session)) => {
                        log::info!(
                            "Stopped active listening session from tray: {} with {} insights",
                            session.id,
                            session.insights.len()
                        );
                    }
                    Ok(None) => {
                        log::debug!("No active session was running");
                    }
                    Err(e) => {
                        log::error!("Error stopping active listening session: {}", e);
                    }
                }

                // Update tray icon and hide overlay
                utils::change_tray_icon(app, utils::TrayIconState::Idle);
                utils::hide_recording_overlay(app);
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app_handle)
        .unwrap();
    app_handle.manage(tray);

    // Initialize tray menu with idle state
    utils::update_tray_menu(app_handle, &utils::TrayIconState::Idle, None);

    // Get the autostart manager and configure based on user setting
    let autostart_manager = app_handle.autolaunch();
    let settings = settings::get_settings(&app_handle);

    if settings.general.autostart_enabled {
        // Enable autostart if user has opted in
        let _ = autostart_manager.enable();
    } else {
        // Disable autostart if user has opted out
        let _ = autostart_manager.disable();
    }

    // Create the recording overlay window (hidden by default)
    utils::create_recording_overlay(app_handle);
}

#[tauri::command]
#[specta::specta]
fn trigger_update_check(app: AppHandle) -> Result<(), String> {
    let settings = settings::get_settings(&app);
    if !settings.general.update_checks_enabled {
        return Ok(());
    }
    app.emit("check-for-updates", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Parse console logging directives from RUST_LOG, falling back to info-level logging
    // when the variable is unset
    let console_filter = build_console_filter();

    let specta_builder = Builder::<tauri::Wry>::new().commands(collect_commands![
        shortcut::change_binding,
        shortcut::reset_binding,
        shortcut::change_ptt_setting,
        shortcut::change_audio_feedback_setting,
        shortcut::change_audio_feedback_volume_setting,
        shortcut::change_sound_theme_setting,
        shortcut::change_start_hidden_setting,
        shortcut::change_autostart_setting,
        shortcut::change_translate_to_english_setting,
        shortcut::change_selected_language_setting,
        shortcut::change_overlay_position_setting,
        shortcut::change_debug_mode_setting,
        shortcut::change_word_correction_threshold_setting,
        shortcut::change_paste_method_setting,
        shortcut::change_clipboard_handling_setting,
        shortcut::change_post_process_enabled_setting,
        shortcut::change_post_process_base_url_setting,
        shortcut::change_post_process_api_key_setting,
        shortcut::change_post_process_model_setting,
        shortcut::set_post_process_provider,
        shortcut::fetch_post_process_models,
        shortcut::add_post_process_prompt,
        shortcut::update_post_process_prompt,
        shortcut::delete_post_process_prompt,
        shortcut::set_post_process_selected_prompt,
        shortcut::update_custom_words,
        shortcut::suspend_binding,
        shortcut::resume_binding,
        shortcut::change_mute_while_recording_setting,
        shortcut::change_append_trailing_space_setting,
        shortcut::change_app_language_setting,
        shortcut::change_update_checks_setting,
        shortcut::change_private_overlay_setting,
        trigger_update_check,
        commands::cancel_operation,
        commands::get_app_dir_path,
        commands::get_app_settings,
        commands::get_default_settings,
        commands::get_log_dir_path,
        commands::set_log_level,
        commands::open_recordings_folder,
        commands::open_log_dir,
        commands::open_app_data_dir,
        commands::check_apple_intelligence_available,
        commands::initialize_enigo,
        commands::models::get_available_models,
        commands::models::get_model_info,
        commands::models::download_model,
        commands::models::delete_model,
        commands::models::cancel_download,
        commands::models::set_active_model,
        commands::models::get_current_model,
        commands::models::get_transcription_model_status,
        commands::models::is_model_loading,
        commands::models::has_any_models_available,
        commands::models::has_any_models_or_downloads,
        commands::models::get_recommended_first_model,
        commands::audio::update_microphone_mode,
        commands::audio::get_microphone_mode,
        commands::audio::get_available_microphones,
        commands::audio::set_selected_microphone,
        commands::audio::get_selected_microphone,
        commands::audio::get_available_output_devices,
        commands::audio::set_selected_output_device,
        commands::audio::get_selected_output_device,
        commands::audio::play_test_sound,
        commands::audio::check_custom_sounds,
        commands::audio::set_clamshell_microphone,
        commands::audio::get_clamshell_microphone,
        commands::audio::is_recording,
        commands::transcription::set_model_unload_timeout,
        commands::transcription::get_model_load_status,
        commands::transcription::unload_model_manually,
        commands::history::get_history_entries,
        commands::history::toggle_history_entry_saved,
        commands::history::get_audio_file_path,
        commands::history::delete_history_entry,
        commands::history::update_history_limit,
        commands::history::update_recording_retention_period,
        commands::active_listening::start_active_listening_session,
        commands::active_listening::stop_active_listening_session,
        commands::active_listening::get_active_listening_state,
        commands::active_listening::get_active_listening_session,
        commands::active_listening::check_ollama_connection,
        commands::active_listening::fetch_ollama_models,
        commands::active_listening::change_active_listening_enabled_setting,
        commands::active_listening::change_active_listening_segment_duration_setting,
        commands::active_listening::change_ollama_base_url_setting,
        commands::active_listening::change_ollama_model_setting,
        commands::active_listening::change_active_listening_context_window_setting,
        commands::active_listening::change_audio_source_type_setting,
        commands::active_listening::change_audio_mix_ratio_setting,
        commands::active_listening::get_audio_source_type,
        commands::active_listening::get_audio_mix_ratio,
        commands::active_listening::get_loopback_support_level,
        commands::active_listening::is_loopback_supported,
        commands::active_listening::list_loopback_devices,
        commands::active_listening::add_active_listening_prompt,
        commands::active_listening::update_active_listening_prompt,
        commands::active_listening::delete_active_listening_prompt,
        commands::active_listening::set_active_listening_selected_prompt,
        commands::active_listening::generate_meeting_summary,
        commands::active_listening::export_meeting_summary,
        commands::ask_ai::get_ask_ai_state,
        commands::ask_ai::is_ask_ai_active,
        commands::ask_ai::get_ask_ai_question,
        commands::ask_ai::get_ask_ai_response,
        commands::ask_ai::get_ask_ai_conversation,
        commands::ask_ai::can_start_ask_ai_recording,
        commands::ask_ai::cancel_ask_ai_session,
        commands::ask_ai::reset_ask_ai_session,
        commands::ask_ai::dismiss_ask_ai_session,
        commands::ask_ai::start_new_ask_ai_conversation,
        commands::ask_ai::change_ask_ai_enabled_setting,
        commands::ask_ai::change_ask_ai_ollama_base_url_setting,
        commands::ask_ai::change_ask_ai_ollama_model_setting,
        commands::ask_ai::change_ask_ai_system_prompt_setting,
        commands::ask_ai::get_ask_ai_settings,
        commands::ask_ai::save_ask_ai_window_bounds,
        commands::ask_ai::get_ask_ai_window_bounds,
        commands::ask_ai::save_ask_ai_conversation_to_history,
        commands::ask_ai::list_ask_ai_conversations,
        commands::ask_ai::get_ask_ai_conversation_from_history,
        commands::ask_ai::delete_ask_ai_conversation_from_history,
        commands::rag::rag_add_document,
        commands::rag::rag_search,
        commands::rag::rag_delete_document,
        commands::rag::rag_list_documents,
        commands::rag::rag_get_stats,
        commands::rag::rag_get_embedding_model,
        commands::rag::rag_set_embedding_model,
        commands::rag::rag_clear_all,
        commands::rag::get_knowledge_base_settings,
        commands::rag::change_knowledge_base_enabled_setting,
        commands::rag::change_auto_index_transcriptions_setting,
        commands::rag::change_kb_embedding_model_setting,
        commands::rag::change_kb_top_k_setting,
        commands::rag::change_kb_similarity_threshold_setting,
        commands::rag::change_kb_use_in_active_listening_setting,
        commands::suggestions::get_suggestions_settings,
        commands::suggestions::update_suggestions_settings,
        commands::suggestions::change_suggestions_enabled_setting,
        commands::suggestions::get_quick_responses,
        commands::suggestions::get_quick_responses_by_category,
        commands::suggestions::add_quick_response,
        commands::suggestions::update_quick_response,
        commands::suggestions::delete_quick_response,
        commands::suggestions::toggle_quick_response,
        commands::suggestions::change_rag_suggestions_enabled,
        commands::suggestions::change_llm_suggestions_enabled,
        commands::suggestions::change_max_suggestions,
        commands::suggestions::change_min_confidence,
        commands::suggestions::change_auto_dismiss_on_copy,
        commands::suggestions::change_display_duration,
        commands::batch_processing::add_to_batch_queue,
        commands::batch_processing::start_batch_processing,
        commands::batch_processing::cancel_batch_processing,
        commands::batch_processing::get_batch_status,
        commands::batch_processing::remove_batch_item,
        commands::batch_processing::clear_completed_batch_items,
        commands::tasks::extract_action_items,
        commands::tasks::get_action_items,
        commands::tasks::toggle_action_item,
        commands::tasks::delete_action_item,
        commands::tasks::export_action_items,
        commands::vocabulary::get_vocabulary,
        commands::vocabulary::add_vocabulary_term,
        commands::vocabulary::remove_vocabulary_term,
        commands::vocabulary::import_vocabulary,
        commands::vocabulary::export_vocabulary,
        commands::sound_detection::get_sound_detection_settings,
        commands::sound_detection::change_sound_detection_enabled,
        commands::sound_detection::change_sound_detection_threshold,
        commands::sound_detection::change_sound_detection_categories,
        commands::sound_detection::change_sound_detection_notification,
        helpers::clamshell::is_laptop,
    ]);

    #[cfg(debug_assertions)] // <- Only export on non-release builds
    specta_builder
        .export(
            Typescript::default().bigint(BigIntExportBehavior::Number),
            "../src/bindings.ts",
        )
        .expect("Failed to export typescript bindings");

    let mut builder = tauri::Builder::default().plugin(
        LogBuilder::new()
            .level(log::LevelFilter::Trace) // Set to most verbose level globally
            .max_file_size(500_000)
            .rotation_strategy(RotationStrategy::KeepOne)
            .clear_targets()
            .targets([
                // Console output respects RUST_LOG environment variable
                Target::new(TargetKind::Stdout).filter({
                    let console_filter = console_filter.clone();
                    move |metadata| console_filter.enabled(metadata)
                }),
                // File logs respect the user's settings (stored in FILE_LOG_LEVEL atomic)
                Target::new(TargetKind::LogDir {
                    file_name: Some("dictum".into()),
                })
                .filter(|metadata| {
                    let file_level = FILE_LOG_LEVEL.load(Ordering::Relaxed);
                    metadata.level() <= level_filter_from_u8(file_level)
                }),
            ])
            .build(),
    );

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_nspanel::init());
    }

    builder
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main_window(app);
        }))
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_macos_permissions::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .manage(Mutex::new(ShortcutToggleStates::default()))
        .setup(move |app| {
            let settings = get_settings(&app.handle());
            let tauri_log_level: tauri_plugin_log::LogLevel = settings.log_level.into();
            let file_log_level: log::Level = tauri_log_level.into();
            // Store the file log level in the atomic for the filter to use
            FILE_LOG_LEVEL.store(file_log_level.to_level_filter() as u8, Ordering::Relaxed);
            let app_handle = app.handle().clone();

            initialize_core_logic(&app_handle);

            // Show main window only if not starting hidden
            if !settings.general.start_hidden {
                if let Some(main_window) = app_handle.get_webview_window("main") {
                    main_window.show().unwrap();
                    main_window.set_focus().unwrap();
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();
                let _res = window.hide();
                #[cfg(target_os = "macos")]
                {
                    let res = window
                        .app_handle()
                        .set_activation_policy(tauri::ActivationPolicy::Accessory);
                    if let Err(e) = res {
                        log::error!("Failed to set activation policy: {}", e);
                    }
                }
            }
            tauri::WindowEvent::ThemeChanged(theme) => {
                log::info!("Theme changed to: {:?}", theme);
                // Update tray icon to match new theme, maintaining idle state
                utils::change_tray_icon(&window.app_handle(), utils::TrayIconState::Idle);
            }
            _ => {}
        })
        .invoke_handler(specta_builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
