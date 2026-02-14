/**
 * Mock for @tauri-apps/api/core
 * Routes all invoke() calls to handlers that read/write mock state.
 * This intercepts every command called via bindings.ts.
 */
import { mockState, updateSettings } from "./mock-state";

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Args = Record<string, any>;

const delay = (ms: number) => new Promise((r) => setTimeout(r, ms));

function ok<T>(data: T) {
  return data;
}

/**
 * Command handlers grouped by feature domain.
 * Each returns the raw data value (not wrapped in Result — bindings.ts wraps it).
 */
const handlers: Record<string, (args: Args) => unknown> = {
  // ── Settings ──────────────────────────────────────────────────────────
  get_app_settings: () => mockState.settings,
  get_default_settings: () => mockState.settings,

  change_ptt_setting: (a) => { updateSettings({ push_to_talk: a.enabled }); return null; },
  change_audio_feedback_setting: (a) => { updateSettings({ audio_feedback: a.enabled }); return null; },
  change_audio_feedback_volume_setting: (a) => { updateSettings({ audio_feedback_volume: a.volume }); return null; },
  change_sound_theme_setting: (a) => { updateSettings({ sound_theme: a.theme }); return null; },
  change_start_hidden_setting: (a) => { updateSettings({ start_hidden: a.enabled }); return null; },
  change_autostart_setting: (a) => { updateSettings({ autostart_enabled: a.enabled }); mockState.autostart = a.enabled; return null; },
  change_translate_to_english_setting: (a) => { updateSettings({ translate_to_english: a.enabled }); return null; },
  change_selected_language_setting: (a) => { updateSettings({ selected_language: a.language }); return null; },
  change_overlay_position_setting: (a) => { updateSettings({ overlay_position: a.position }); return null; },
  change_debug_mode_setting: (a) => { updateSettings({ debug_mode: a.enabled }); return null; },
  change_word_correction_threshold_setting: (a) => { updateSettings({ word_correction_threshold: a.threshold }); return null; },
  change_paste_method_setting: (a) => { updateSettings({ paste_method: a.method }); return null; },
  change_clipboard_handling_setting: (a) => { updateSettings({ clipboard_handling: a.handling }); return null; },
  change_post_process_enabled_setting: (a) => { updateSettings({ post_process_enabled: a.enabled }); return null; },
  change_post_process_base_url_setting: () => null,
  change_post_process_api_key_setting: () => null,
  change_post_process_model_setting: () => null,
  change_app_language_setting: (a) => { updateSettings({ app_language: a.language }); return null; },
  change_update_checks_setting: (a) => { updateSettings({ update_checks_enabled: a.enabled }); return null; },
  change_mute_while_recording_setting: (a) => { updateSettings({ mute_while_recording: a.enabled }); return null; },
  change_append_trailing_space_setting: (a) => { updateSettings({ append_trailing_space: a.enabled }); return null; },

  // Bindings
  change_binding: (_a) => ({ success: true, binding: null, error: null }),
  reset_binding: (_a) => ({ success: true, binding: null, error: null }),
  suspend_binding: () => null,
  resume_binding: () => null,

  // Post-process
  set_post_process_provider: () => null,
  fetch_post_process_models: () => [],
  add_post_process_prompt: (a) => ({ id: crypto.randomUUID(), name: a.name, prompt: a.prompt }),
  update_post_process_prompt: () => null,
  delete_post_process_prompt: () => null,
  set_post_process_selected_prompt: () => null,

  // Custom words
  update_custom_words: (a) => { updateSettings({ custom_words: a.words }); return null; },

  // ── Models ────────────────────────────────────────────────────────────
  get_available_models: () => mockState.models,
  get_model_info: (a) => mockState.models.find((m) => m.id === a.modelId) || null,
  download_model: () => null,
  delete_model: () => null,
  cancel_download: () => null,
  set_active_model: (a) => { mockState.currentModel = a.modelId; return null; },
  get_current_model: () => mockState.currentModel,
  get_transcription_model_status: () => null,
  is_model_loading: () => false,
  has_any_models_available: () => mockState.hasAnyModels,
  has_any_models_or_downloads: () => mockState.hasAnyModels,
  get_recommended_first_model: () => "whisper-tiny",
  set_model_unload_timeout: () => undefined,
  get_model_load_status: () => mockState.modelLoadStatus,
  unload_model_manually: () => null,

  // ── Audio devices ─────────────────────────────────────────────────────
  get_available_microphones: () => mockState.microphones,
  set_selected_microphone: (a) => { mockState.selectedMicrophone = a.deviceName; return null; },
  get_selected_microphone: () => mockState.selectedMicrophone,
  get_available_output_devices: () => mockState.outputDevices,
  set_selected_output_device: (a) => { mockState.selectedOutputDevice = a.deviceName; return null; },
  get_selected_output_device: () => mockState.selectedOutputDevice,
  play_test_sound: () => undefined,
  check_custom_sounds: () => ({ start: false, stop: false }),
  set_clamshell_microphone: (a) => { updateSettings({ clamshell_microphone: a.deviceName }); return null; },
  get_clamshell_microphone: () => mockState.settings.clamshell_microphone ?? "",
  update_microphone_mode: (a) => { updateSettings({ always_on_microphone: a.alwaysOn }); return null; },
  get_microphone_mode: () => mockState.settings.always_on_microphone ?? false,

  // ── Recording ─────────────────────────────────────────────────────────
  is_recording: () => mockState.isRecording,
  cancel_operation: () => undefined,

  // ── History ───────────────────────────────────────────────────────────
  get_history_entries: () => mockState.historyEntries,
  toggle_history_entry_saved: (a) => {
    const entry = mockState.historyEntries.find((e) => e.id === a.id);
    if (entry) entry.saved = !entry.saved;
    return null;
  },
  get_audio_file_path: (a) => `/mock/recordings/${a.fileName}`,
  delete_history_entry: (a) => {
    mockState.historyEntries = mockState.historyEntries.filter((e) => e.id !== a.id);
    return null;
  },
  update_history_limit: (a) => { updateSettings({ history_limit: a.limit }); return null; },
  update_recording_retention_period: (a) => { updateSettings({ recording_retention_period: a.period }); return null; },

  // ── App utilities ─────────────────────────────────────────────────────
  get_app_dir_path: () => "/mock/app-data",
  get_log_dir_path: () => "/mock/logs",
  set_log_level: (a) => { updateSettings({ log_level: a.level }); return null; },
  open_recordings_folder: () => null,
  open_log_dir: () => null,
  open_app_data_dir: () => null,
  check_apple_intelligence_available: () => false,
  initialize_enigo: () => null,
  initialize_shortcuts: () => null,
  trigger_update_check: () => null,
  is_laptop: () => true,

  // ── Active Listening ──────────────────────────────────────────────────
  start_active_listening_session: () => "mock-session-1",
  stop_active_listening_session: () => null,
  get_active_listening_state: () => mockState.activeListeningState,
  get_active_listening_session: () => null,
  check_ollama_connection: () => mockState.ollamaConnected,
  fetch_ollama_models: () => mockState.ollamaModels,
  change_active_listening_enabled_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, enabled: a.enabled };
    return null;
  },
  change_active_listening_segment_duration_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, segment_duration_seconds: a.durationSeconds };
    return null;
  },
  change_ollama_base_url_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, ollama_base_url: a.baseUrl };
    return null;
  },
  change_ollama_model_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, ollama_model: a.model };
    return null;
  },
  change_active_listening_context_window_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, context_window_size: a.size };
    return null;
  },
  change_audio_source_type_setting: (a) => {
    mockState.settings.active_listening = { ...mockState.settings.active_listening, audio_source_type: a.sourceType };
    return null;
  },
  change_audio_mix_ratio_setting: (a) => {
    mockState.settings.active_listening = {
      ...mockState.settings.active_listening,
      audio_mix_settings: { mix_ratio: a.mixRatio },
    };
    return null;
  },
  get_audio_source_type: () => mockState.settings.active_listening.audio_source_type,
  get_audio_mix_ratio: () => mockState.settings.active_listening.audio_mix_settings.mix_ratio,
  get_loopback_support_level: () => "native",
  is_loopback_supported: () => true,
  list_loopback_devices: () => [
    { id: "default", name: "System Audio", is_default: true },
  ],
  add_active_listening_prompt: (a) => ({
    id: crypto.randomUUID(),
    name: a.name,
    prompt_template: a.promptTemplate,
    created_at: Date.now(),
  }),
  update_active_listening_prompt: () => null,
  delete_active_listening_prompt: () => null,
  set_active_listening_selected_prompt: () => null,
  generate_meeting_summary: () => ({
    session_id: "mock-session-1",
    executive_summary: "Mock meeting summary.",
    decisions: [],
    action_items: [],
    topics: [],
    follow_ups: [],
    duration_minutes: 30,
    generated_at: Date.now(),
  }),
  export_meeting_summary: () => "Exported summary content",

  // ── Ask AI ────────────────────────────────────────────────────────────
  get_ask_ai_state: () => mockState.askAiState,
  is_ask_ai_active: () => false,
  get_ask_ai_question: () => null,
  get_ask_ai_response: () => "",
  get_ask_ai_conversation: () => null,
  can_start_ask_ai_recording: () => true,
  cancel_ask_ai_session: () => null,
  reset_ask_ai_session: () => null,
  dismiss_ask_ai_session: () => null,
  start_new_ask_ai_conversation: () => null,
  change_ask_ai_enabled_setting: (a) => {
    mockState.settings.ask_ai = { ...mockState.settings.ask_ai, enabled: a.enabled };
    return null;
  },
  change_ask_ai_ollama_base_url_setting: (a) => {
    mockState.settings.ask_ai = { ...mockState.settings.ask_ai, ollama_base_url: a.baseUrl };
    return null;
  },
  change_ask_ai_ollama_model_setting: (a) => {
    mockState.settings.ask_ai = { ...mockState.settings.ask_ai, ollama_model: a.model };
    return null;
  },
  change_ask_ai_system_prompt_setting: (a) => {
    mockState.settings.ask_ai = { ...mockState.settings.ask_ai, system_prompt: a.prompt };
    return null;
  },
  get_ask_ai_settings: () => mockState.settings.ask_ai,
  save_ask_ai_window_bounds: () => null,
  get_ask_ai_window_bounds: () => ({
    width: mockState.settings.ask_ai.window_width,
    height: mockState.settings.ask_ai.window_height,
    x: mockState.settings.ask_ai.window_x,
    y: mockState.settings.ask_ai.window_y,
  }),
  save_ask_ai_conversation_to_history: () => null,
  list_ask_ai_conversations: () => [],
  get_ask_ai_conversation_from_history: () => null,
  delete_ask_ai_conversation_from_history: () => null,
  load_ask_ai_conversation: () => null,

  // ── RAG / Knowledge Base ──────────────────────────────────────────────
  rag_add_document: () => 1,
  rag_search: () => [],
  rag_delete_document: () => null,
  rag_list_documents: () => [],
  rag_get_stats: () => ({ document_count: 0, embedding_count: 0 }),
  rag_get_embedding_model: () => mockState.settings.knowledge_base.embedding_model,
  rag_set_embedding_model: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, embedding_model: a.model };
    return null;
  },
  rag_clear_all: () => null,
  get_knowledge_base_settings: () => mockState.settings.knowledge_base,
  change_knowledge_base_enabled_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, enabled: a.enabled };
    return null;
  },
  change_auto_index_transcriptions_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, auto_index_transcriptions: a.autoIndex };
    return null;
  },
  change_kb_embedding_model_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, embedding_model: a.model };
    return null;
  },
  change_kb_top_k_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, top_k: a.topK };
    return null;
  },
  change_kb_similarity_threshold_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, similarity_threshold: a.threshold };
    return null;
  },
  change_kb_use_in_active_listening_setting: (a) => {
    mockState.settings.knowledge_base = { ...mockState.settings.knowledge_base, use_in_active_listening: a.useInAl };
    return null;
  },

  // ── Sound Detection ───────────────────────────────────────────────────
  get_sound_detection_settings: () => mockState.settings.sound_detection,
  change_sound_detection_enabled: (a) => {
    mockState.settings.sound_detection = { ...mockState.settings.sound_detection, enabled: a.enabled };
    return null;
  },
  change_sound_detection_threshold: (a) => {
    mockState.settings.sound_detection = { ...mockState.settings.sound_detection, threshold: a.threshold };
    return null;
  },
  change_sound_detection_categories: (a) => {
    mockState.settings.sound_detection = { ...mockState.settings.sound_detection, categories: a.categories };
    return null;
  },
  change_sound_detection_notification: (a) => {
    mockState.settings.sound_detection = { ...mockState.settings.sound_detection, notification_enabled: a.enabled };
    return null;
  },

  // ── Batch Processing ──────────────────────────────────────────────────
  batch_process_files: () => null,
  get_batch_processing_status: () => ({ status: "idle", processed: 0, total: 0 }),
  cancel_batch_processing: () => null,

  // ── Vocabulary ────────────────────────────────────────────────────────
  get_vocabulary: () => [],
  add_vocabulary_entry: () => null,
  remove_vocabulary_entry: () => null,
  import_vocabulary: () => null,
  export_vocabulary: () => "",
};

/**
 * Mock invoke — the core interception point.
 * Matches command names to handlers and returns their results.
 */
export async function invoke<T = unknown>(
  cmd: string,
  args?: Args,
): Promise<T> {
  const handler = handlers[cmd];
  if (handler) {
    await delay(1); // Yield to event loop — prevents synchronous re-render cascades
    const result = handler(args ?? {});
    return result as T;
  }

  console.warn(`[E2E Mock] Unhandled invoke command: "${cmd}"`, args);
  return null as T;
}

/**
 * Mock Channel class used by tauri-specta for streaming.
 */
export class Channel<T = unknown> {
  onmessage: ((message: T) => void) | null = null;

  /** Simulate receiving a message */
  _receive(message: T): void {
    if (this.onmessage) {
      this.onmessage(message);
    }
  }
}

/**
 * Mock convertFileSrc — returns a data URL placeholder.
 */
export function convertFileSrc(path: string, _protocol?: string): string {
  return `http://localhost:1420/mock-asset?path=${encodeURIComponent(path)}`;
}
