/**
 * Centralized mock state for E2E browser testing.
 * All mock modules read from / write to this state.
 * Exposes window.__E2E_MOCK__ for Playwright's browser_evaluate to manipulate state mid-test.
 */

export interface MockSettings {
  push_to_talk: boolean;
  start_hidden: boolean;
  autostart_enabled: boolean;
  update_checks_enabled: boolean;
  mute_while_recording: boolean;
  append_trailing_space: boolean;
  app_language: string;
  bindings: Record<string, unknown>;
  audio_feedback: boolean;
  audio_feedback_volume: number;
  sound_theme: string;
  selected_model: string | null;
  always_on_microphone: boolean;
  selected_microphone: string | null;
  clamshell_microphone: string | null;
  selected_output_device: string | null;
  translate_to_english: boolean;
  selected_language: string;
  overlay_position: string;
  debug_mode: boolean;
  log_level: string;
  custom_words: string[];
  model_unload_timeout: string;
  word_correction_threshold: number;
  history_limit: number;
  recording_retention_period: string;
  paste_method: string;
  clipboard_handling: string;
  post_process_enabled: boolean;
  post_process_provider_id: string | null;
  post_process_providers: unknown[];
  post_process_api_keys: Record<string, string>;
  post_process_models: Record<string, string>;
  post_process_prompts: unknown[];
  post_process_selected_prompt_id: string | null;
  active_listening: {
    enabled: boolean;
    segment_duration_seconds: number;
    ollama_base_url: string;
    ollama_model: string;
    prompts: unknown[];
    selected_prompt_id: string | null;
    context_window_size: number;
    audio_source_type: string;
    audio_mix_settings: { mix_ratio: number };
  };
  ask_ai: {
    enabled: boolean;
    ollama_base_url: string;
    ollama_model: string;
    system_prompt: string;
    window_width: number | null;
    window_height: number | null;
    window_x: number | null;
    window_y: number | null;
  };
  knowledge_base: {
    enabled: boolean;
    auto_index_transcriptions: boolean;
    embedding_model: string;
    top_k: number;
    similarity_threshold: number;
    use_in_active_listening: boolean;
  };
  suggestions: {
    enabled: boolean;
    quick_responses: unknown[];
    rag_suggestions_enabled: boolean;
    llm_suggestions_enabled: boolean;
    max_suggestions: number;
    min_confidence: number;
    auto_dismiss_on_copy: boolean;
    display_duration_seconds: number;
  };
  sound_detection: {
    enabled: boolean;
    categories: string[];
    threshold: number;
    notification_enabled: boolean;
  };
  paste_delay_ms: number;
}

export interface MockModel {
  id: string;
  name: string;
  description: string;
  filename: string;
  url: string | null;
  size_mb: number;
  is_downloaded: boolean;
  is_downloading: boolean;
  partial_size: number;
  is_directory: boolean;
  engine_type: string;
  accuracy_score: number;
  speed_score: number;
  supported_languages: string[];
  supports_translation: boolean;
  is_recommended: boolean;
  is_custom: boolean;
}

export interface MockHistoryEntry {
  id: number;
  file_name: string;
  timestamp: number;
  saved: boolean;
  title: string;
  transcription_text: string;
  post_processed_text: string | null;
  post_process_prompt: string | null;
}

export interface MockState {
  settings: MockSettings;
  hasAnyModels: boolean;
  models: MockModel[];
  currentModel: string;
  microphones: Array<{ index: string; name: string; is_default: boolean }>;
  outputDevices: Array<{ index: string; name: string; is_default: boolean }>;
  selectedMicrophone: string;
  selectedOutputDevice: string;
  historyEntries: MockHistoryEntry[];
  isRecording: boolean;
  modelLoadStatus: { is_loaded: boolean; current_model: string | null };
  activeListeningState: string;
  askAiState: string;
  ollamaConnected: boolean;
  ollamaModels: Array<{ name: string }>;
  clipboard: string;
  autostart: boolean;
  fileSystem: Record<string, string>;
  eventListeners: Map<string, Set<(event: unknown) => void>>;
}

function createDefaultSettings(): MockSettings {
  return {
    push_to_talk: true,
    start_hidden: false,
    autostart_enabled: false,
    update_checks_enabled: true,
    mute_while_recording: false,
    append_trailing_space: true,
    app_language: "en",
    bindings: {},
    audio_feedback: true,
    audio_feedback_volume: 0.5,
    sound_theme: "marimba",
    selected_model: null,
    always_on_microphone: false,
    selected_microphone: null,
    clamshell_microphone: null,
    selected_output_device: null,
    translate_to_english: false,
    selected_language: "auto",
    overlay_position: "top",
    debug_mode: false,
    log_level: "info",
    custom_words: [],
    model_unload_timeout: "min_5",
    word_correction_threshold: 0.5,
    history_limit: 100,
    recording_retention_period: "preserve_limit",
    paste_method: "ctrl_v",
    clipboard_handling: "copy_to_clipboard",
    post_process_enabled: false,
    post_process_provider_id: null,
    post_process_providers: [],
    post_process_api_keys: {},
    post_process_models: {},
    post_process_prompts: [],
    post_process_selected_prompt_id: null,
    active_listening: {
      enabled: false,
      segment_duration_seconds: 30,
      ollama_base_url: "http://localhost:11434",
      ollama_model: "llama3",
      prompts: [],
      selected_prompt_id: null,
      context_window_size: 5,
      audio_source_type: "microphone",
      audio_mix_settings: { mix_ratio: 0.5 },
    },
    ask_ai: {
      enabled: false,
      ollama_base_url: "http://localhost:11434",
      ollama_model: "llama3",
      system_prompt: "You are a helpful assistant.",
      window_width: null,
      window_height: null,
      window_x: null,
      window_y: null,
    },
    knowledge_base: {
      enabled: false,
      auto_index_transcriptions: false,
      embedding_model: "nomic-embed-text",
      top_k: 5,
      similarity_threshold: 0.7,
      use_in_active_listening: false,
    },
    suggestions: {
      enabled: false,
      quick_responses: [],
      rag_suggestions_enabled: false,
      llm_suggestions_enabled: false,
      max_suggestions: 3,
      min_confidence: 0.5,
      auto_dismiss_on_copy: true,
      display_duration_seconds: 0,
    },
    sound_detection: {
      enabled: false,
      categories: ["doorbell", "alarm", "phone_ring", "dog_bark", "baby_cry", "knocking", "siren", "applause"],
      threshold: 0.5,
      notification_enabled: true,
    },
    paste_delay_ms: 50,
  };
}

function createDefaultModels(): MockModel[] {
  return [
    {
      id: "whisper-tiny",
      name: "Whisper Tiny",
      description: "Fastest, lowest accuracy. Good for quick notes.",
      filename: "ggml-tiny.bin",
      url: null,
      size_mb: 75,
      is_downloaded: true,
      is_downloading: false,
      partial_size: 0,
      is_directory: false,
      engine_type: "Whisper",
      accuracy_score: 3,
      speed_score: 10,
      supported_languages: ["en", "es", "fr", "de", "ja", "zh"],
      supports_translation: true,
      is_recommended: false,
      is_custom: false,
    },
    {
      id: "whisper-base",
      name: "Whisper Base",
      description: "Good balance of speed and accuracy.",
      filename: "ggml-base.bin",
      url: null,
      size_mb: 142,
      is_downloaded: true,
      is_downloading: false,
      partial_size: 0,
      is_directory: false,
      engine_type: "Whisper",
      accuracy_score: 5,
      speed_score: 8,
      supported_languages: ["en", "es", "fr", "de", "ja", "zh"],
      supports_translation: true,
      is_recommended: false,
      is_custom: false,
    },
    {
      id: "whisper-small",
      name: "Whisper Small",
      description: "Better accuracy, moderate speed.",
      filename: "ggml-small.bin",
      url: null,
      size_mb: 466,
      is_downloaded: false,
      is_downloading: false,
      partial_size: 0,
      is_directory: false,
      engine_type: "Whisper",
      accuracy_score: 7,
      speed_score: 5,
      supported_languages: ["en", "es", "fr", "de", "ja", "zh"],
      supports_translation: true,
      is_recommended: false,
      is_custom: false,
    },
  ];
}

/** Check URL params for state overrides (e.g. ?hasAnyModels=false) */
function getUrlOverrides(): Partial<MockState> {
  if (typeof window === "undefined") return {};
  const params = new URLSearchParams(window.location.search);
  const overrides: Partial<MockState> = {};
  if (params.get("hasAnyModels") === "false") overrides.hasAnyModels = false;
  return overrides;
}

function createDefaultState(): MockState {
  const urlOverrides = getUrlOverrides();
  return {
    settings: createDefaultSettings(),
    hasAnyModels: urlOverrides.hasAnyModels ?? true,
    models: createDefaultModels(),
    currentModel: "whisper-tiny",
    microphones: [
      { index: "0", name: "Built-in Microphone", is_default: true },
      { index: "1", name: "External USB Microphone", is_default: false },
    ],
    outputDevices: [
      { index: "0", name: "Built-in Speakers", is_default: true },
      { index: "1", name: "External Headphones", is_default: false },
    ],
    selectedMicrophone: "Built-in Microphone",
    selectedOutputDevice: "Built-in Speakers",
    historyEntries: [
      {
        id: 1,
        file_name: "recording_001.wav",
        timestamp: Date.now() - 3600000,
        saved: false,
        title: "Meeting notes",
        transcription_text:
          "This is a sample transcription from the meeting earlier today.",
        post_processed_text: null,
        post_process_prompt: null,
      },
      {
        id: 2,
        file_name: "recording_002.wav",
        timestamp: Date.now() - 7200000,
        saved: true,
        title: "Quick memo",
        transcription_text: "Remember to send the report by Friday.",
        post_processed_text: null,
        post_process_prompt: null,
      },
    ],
    isRecording: false,
    modelLoadStatus: { is_loaded: false, current_model: null },
    activeListeningState: "idle",
    askAiState: "idle",
    ollamaConnected: true,
    ollamaModels: [{ name: "llama3" }, { name: "mistral" }],
    clipboard: "",
    autostart: false,
    fileSystem: {},
    eventListeners: new Map(),
  };
}

// Global mock state instance
export let mockState: MockState = createDefaultState();

/** Reset state to defaults */
export function resetMockState(): void {
  mockState = createDefaultState();
}

/** Deep-merge partial updates into settings */
export function updateSettings(partial: Partial<MockSettings>): void {
  mockState.settings = { ...mockState.settings, ...partial };
}

/** Patch any top-level state fields */
export function updateState(partial: Partial<Omit<MockState, "eventListeners">>): void {
  Object.assign(mockState, partial);
}

/** Get a plain-object snapshot of the current state (excluding non-serializable fields) */
export function getStateSnapshot(): Omit<MockState, "eventListeners"> {
  const { eventListeners: _, ...rest } = mockState;
  return JSON.parse(JSON.stringify(rest));
}

// Expose on window for Playwright's browser_evaluate
interface E2EMockAPI {
  getState: () => Omit<MockState, "eventListeners">;
  reset: () => void;
  updateSettings: (partial: Partial<MockSettings>) => void;
  update: (partial: Partial<Omit<MockState, "eventListeners">>) => void;
  emit: (event: string, payload?: unknown) => void;
}

declare global {
  interface Window {
    __E2E_MOCK__: E2EMockAPI;
  }
}

function emitEvent(event: string, payload?: unknown): void {
  const listeners = mockState.eventListeners.get(event);
  if (listeners) {
    for (const cb of listeners) {
      try {
        cb({ event, id: Math.random(), windowLabel: "main", payload });
      } catch (e) {
        console.warn(`[E2E Mock] Error in event listener for "${event}":`, e);
      }
    }
  }
}

if (typeof window !== "undefined") {
  window.__E2E_MOCK__ = {
    getState: getStateSnapshot,
    reset: resetMockState,
    updateSettings,
    update: updateState,
    emit: emitEvent,
  };
}
