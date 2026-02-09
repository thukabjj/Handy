import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock Tauri core invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
  Channel: vi.fn(),
}));

// Mock Tauri event API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
  once: vi.fn(() => Promise.resolve(() => {})),
}));

// Mock Tauri window API
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: vi.fn(() => ({
    close: vi.fn(),
    hide: vi.fn(),
    show: vi.fn(),
    setFocus: vi.fn(),
  })),
}));

// Mock Tauri webview window API
vi.mock('@tauri-apps/api/webviewWindow', () => ({
  WebviewWindow: vi.fn(),
}));

// Mock Tauri plugins
vi.mock('@tauri-apps/plugin-os', () => ({
  platform: vi.fn(() => Promise.resolve('darwin')),
  type: vi.fn(() => Promise.resolve('Darwin')),
  version: vi.fn(() => Promise.resolve('14.0')),
}));

vi.mock('@tauri-apps/plugin-opener', () => ({
  openUrl: vi.fn(),
  openPath: vi.fn(),
}));

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  writeText: vi.fn(),
  readText: vi.fn(() => Promise.resolve('')),
}));

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(),
      set: vi.fn(),
      save: vi.fn(),
    })
  ),
}));

vi.mock('@tauri-apps/plugin-autostart', () => ({
  enable: vi.fn(),
  disable: vi.fn(),
  isEnabled: vi.fn(() => Promise.resolve(false)),
}));

// Mock i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: vi.fn(),
      language: 'en',
    },
  }),
  Trans: ({ children }: { children: React.ReactNode }) => children,
  initReactI18next: {
    type: '3rdParty',
    init: vi.fn(),
  },
}));

// Mock bindings with default test values
vi.mock('@/bindings', () => {
  const mockResult = <T>(data: T) => ({ status: 'ok' as const, data });

  return {
    commands: {
      getAppSettings: vi.fn(() =>
        Promise.resolve(
          mockResult({
            push_to_talk: true,
            start_hidden: false,
            autostart_enabled: false,
            update_checks_enabled: true,
            mute_while_recording: false,
            append_trailing_space: true,
            app_language: 'en',
            bindings: {},
            audio_feedback: true,
            audio_feedback_volume: 0.5,
            sound_theme: 'marimba',
            selected_model: null,
            always_on_microphone: false,
            selected_microphone: null,
            clamshell_microphone: null,
            selected_output_device: null,
            translate_to_english: false,
            selected_language: 'auto',
            overlay_position: 'top',
            debug_mode: false,
            log_level: 'info',
            custom_words: [],
            model_unload_timeout: 'min_5',
            word_correction_threshold: 0.5,
            history_limit: 100,
            recording_retention_period: 'preserve_limit',
            paste_method: 'ctrl_v',
            clipboard_handling: 'copy_to_clipboard',
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
              ollama_base_url: 'http://localhost:11434',
              ollama_model: 'llama3',
              prompts: [],
              selected_prompt_id: null,
              context_window_size: 5,
            },
            ask_ai: {
              enabled: false,
            },
          })
        )
      ),
      getDefaultSettings: vi.fn(() =>
        Promise.resolve(
          mockResult({
            push_to_talk: true,
            bindings: {},
            audio_feedback: true,
          })
        )
      ),
      changePttSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAudioFeedbackSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAudioFeedbackVolumeSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeSoundThemeSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeStartHiddenSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAutostartSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeTranslateToEnglishSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeSelectedLanguageSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeOverlayPositionSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeDebugModeSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changePasteMethodSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeClipboardHandlingSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changePostProcessEnabledSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAppLanguageSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeUpdateChecksSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeMuteWhileRecordingSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAppendTrailingSpaceSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeBinding: vi.fn(() =>
        Promise.resolve(mockResult({ success: true, binding: null, error: null }))
      ),
      resetBinding: vi.fn(() =>
        Promise.resolve(mockResult({ success: true, binding: null, error: null }))
      ),
      suspendBinding: vi.fn(() => Promise.resolve(mockResult(null))),
      resumeBinding: vi.fn(() => Promise.resolve(mockResult(null))),
      getAvailableModels: vi.fn(() => Promise.resolve(mockResult([]))),
      getModelInfo: vi.fn(() => Promise.resolve(mockResult(null))),
      downloadModel: vi.fn(() => Promise.resolve(mockResult(null))),
      deleteModel: vi.fn(() => Promise.resolve(mockResult(null))),
      cancelDownload: vi.fn(() => Promise.resolve(mockResult(null))),
      setActiveModel: vi.fn(() => Promise.resolve(mockResult(null))),
      getCurrentModel: vi.fn(() => Promise.resolve(mockResult(''))),
      getTranscriptionModelStatus: vi.fn(() => Promise.resolve(mockResult(null))),
      isModelLoading: vi.fn(() => Promise.resolve(mockResult(false))),
      hasAnyModelsAvailable: vi.fn(() => Promise.resolve(mockResult(false))),
      hasAnyModelsOrDownloads: vi.fn(() => Promise.resolve(mockResult(false))),
      getRecommendedFirstModel: vi.fn(() => Promise.resolve(mockResult('whisper-tiny'))),
      getAvailableMicrophones: vi.fn(() => Promise.resolve(mockResult([]))),
      setSelectedMicrophone: vi.fn(() => Promise.resolve(mockResult(null))),
      getSelectedMicrophone: vi.fn(() => Promise.resolve(mockResult(''))),
      getAvailableOutputDevices: vi.fn(() => Promise.resolve(mockResult([]))),
      setSelectedOutputDevice: vi.fn(() => Promise.resolve(mockResult(null))),
      getSelectedOutputDevice: vi.fn(() => Promise.resolve(mockResult(''))),
      playTestSound: vi.fn(),
      checkCustomSounds: vi.fn(() => Promise.resolve({ start: false, stop: false })),
      isRecording: vi.fn(() => Promise.resolve(false)),
      cancelOperation: vi.fn(),
      getAppDirPath: vi.fn(() => Promise.resolve(mockResult('/app'))),
      getLogDirPath: vi.fn(() => Promise.resolve(mockResult('/logs'))),
      setLogLevel: vi.fn(() => Promise.resolve(mockResult(null))),
      openRecordingsFolder: vi.fn(() => Promise.resolve(mockResult(null))),
      openLogDir: vi.fn(() => Promise.resolve(mockResult(null))),
      openAppDataDir: vi.fn(() => Promise.resolve(mockResult(null))),
      checkAppleIntelligenceAvailable: vi.fn(() => Promise.resolve(false)),
      initializeEnigo: vi.fn(() => Promise.resolve(mockResult(null))),
      triggerUpdateCheck: vi.fn(() => Promise.resolve(mockResult(null))),
      getHistoryEntries: vi.fn(() => Promise.resolve(mockResult([]))),
      toggleHistoryEntrySaved: vi.fn(() => Promise.resolve(mockResult(null))),
      getAudioFilePath: vi.fn(() => Promise.resolve(mockResult(''))),
      deleteHistoryEntry: vi.fn(() => Promise.resolve(mockResult(null))),
      updateHistoryLimit: vi.fn(() => Promise.resolve(mockResult(null))),
      updateRecordingRetentionPeriod: vi.fn(() => Promise.resolve(mockResult(null))),
      updateMicrophoneMode: vi.fn(() => Promise.resolve(mockResult(null))),
      getMicrophoneMode: vi.fn(() => Promise.resolve(mockResult(false))),
      setModelUnloadTimeout: vi.fn(),
      getModelLoadStatus: vi.fn(() =>
        Promise.resolve(mockResult({ is_loaded: false, current_model: null }))
      ),
      unloadModelManually: vi.fn(() => Promise.resolve(mockResult(null))),
      updateCustomWords: vi.fn(() => Promise.resolve(mockResult(null))),
      changeWordCorrectionThresholdSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      setClamshellMicrophone: vi.fn(() => Promise.resolve(mockResult(null))),
      getClamshellMicrophone: vi.fn(() => Promise.resolve(mockResult(''))),
      isLaptop: vi.fn(() => Promise.resolve(mockResult(true))),
      // Post-process commands
      setPostProcessProvider: vi.fn(() => Promise.resolve(mockResult(null))),
      fetchPostProcessModels: vi.fn(() => Promise.resolve(mockResult([]))),
      addPostProcessPrompt: vi.fn(() =>
        Promise.resolve(mockResult({ id: '1', name: 'Test', prompt: 'Test prompt' }))
      ),
      updatePostProcessPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      deletePostProcessPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      setPostProcessSelectedPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      changePostProcessBaseUrlSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changePostProcessApiKeySetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changePostProcessModelSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      // Active listening commands
      startActiveListeningSession: vi.fn(() => Promise.resolve(mockResult('session-1'))),
      stopActiveListeningSession: vi.fn(() => Promise.resolve(mockResult(null))),
      getActiveListeningState: vi.fn(() => Promise.resolve('idle')),
      getActiveListeningSession: vi.fn(() => Promise.resolve(null)),
      checkOllamaConnection: vi.fn(() => Promise.resolve(mockResult(true))),
      fetchOllamaModels: vi.fn(() => Promise.resolve(mockResult([]))),
      changeActiveListeningEnabledSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeActiveListeningSegmentDurationSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeOllamaBaseUrlSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeOllamaModelSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeActiveListeningContextWindowSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      addActiveListeningPrompt: vi.fn(() =>
        Promise.resolve(
          mockResult({
            id: '1',
            name: 'Test',
            prompt_template: 'Test template',
            created_at: Date.now(),
          })
        )
      ),
      updateActiveListeningPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      deleteActiveListeningPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      setActiveListeningSelectedPrompt: vi.fn(() => Promise.resolve(mockResult(null))),
      // Ask AI commands
      getAskAiState: vi.fn(() => Promise.resolve('idle')),
      isAskAiActive: vi.fn(() => Promise.resolve(false)),
      getAskAiQuestion: vi.fn(() => Promise.resolve(null)),
      getAskAiResponse: vi.fn(() => Promise.resolve('')),
      cancelAskAiSession: vi.fn(() => Promise.resolve(mockResult(null))),
      resetAskAiSession: vi.fn(() => Promise.resolve(mockResult(null))),
      dismissAskAiSession: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAskAiEnabledSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAskAiOllamaBaseUrlSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAskAiOllamaModelSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      changeAskAiSystemPromptSetting: vi.fn(() => Promise.resolve(mockResult(null))),
      listAskAiConversations: vi.fn(() => Promise.resolve(mockResult([]))),
      getAskAiConversation: vi.fn(() => Promise.resolve(mockResult(null))),
      deleteAskAiConversationFromHistory: vi.fn(() => Promise.resolve(mockResult(null))),
      loadAskAiConversation: vi.fn(() => Promise.resolve(mockResult(null))),
    },
  };
});

// Global test utilities
globalThis.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});
