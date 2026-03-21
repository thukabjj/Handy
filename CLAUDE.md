# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Quick Reference

| Command               | Description                                     |
| --------------------- | ----------------------------------------------- |
| `make install`        | Install all dependencies (frontend + VAD model) |
| `make dev`            | Run in development mode                         |
| `make check`          | Full check (Rust + TypeScript)                  |
| `make lint`           | Run all linters                                 |
| `make test`           | Run all tests                                   |
| `bun run tauri build` | Build for production                            |

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable via `rust-toolchain.toml`)
- [Bun](https://bun.sh/) (JavaScript runtime)
- macOS: Xcode Command Line Tools
- Linux: `build-essential`, `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`

## Development Commands

```bash
# Install dependencies
make install
# Or manually:
bun install
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx

# Development
make dev                    # Run with hot reload
bun run tauri dev           # Direct Tauri dev command
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev  # macOS cmake workaround

# Code Quality
bun run lint                # ESLint frontend
bun run lint:fix            # ESLint with auto-fix
bun run format              # Prettier + cargo fmt
bun run format:check        # Check formatting
cargo clippy                # Rust linter (in src-tauri/)
cargo fmt                   # Rust formatter (in src-tauri/)

# Testing
bun run test                # Frontend tests (Vitest)
bun run test:watch          # Frontend tests in watch mode
cargo test                  # Rust tests (in src-tauri/)
make test                   # All tests

# Building
bun run tauri build         # Production build
bun run build               # Frontend only
```

---

## Architecture Overview

Handy is a cross-platform desktop speech-to-text application built with:

- **Backend**: Tauri 2.x (Rust)
- **Frontend**: React + TypeScript + Tailwind CSS
- **State**: Zustand (frontend) + Tauri managed state (backend)
- **Persistence**: SQLite (history) + tauri-plugin-store (settings)

### Backend Structure (`src-tauri/src/`)

```
src-tauri/src/
├── lib.rs                    # Main entry, manager initialization
├── main.rs                   # Entry point
├── error.rs                  # HandyError structured errors
├── ollama_client.rs          # Streaming Ollama LLM client
├── llm_client.rs             # Generic LLM client abstraction
├── managers/                 # Core business logic
│   ├── audio.rs             # Recording, device management
│   ├── model.rs             # Model downloads, lifecycle
│   ├── transcription.rs     # Speech-to-text pipeline
│   ├── history.rs           # SQLite persistence
│   ├── active_listening.rs  # Continuous transcription + AI
│   ├── ask_ai.rs            # Voice conversations
│   ├── ask_ai_history.rs    # Conversation persistence
│   ├── rag.rs               # Vector embeddings, search
│   └── suggestion_engine.rs # Context-aware suggestions
├── commands/                 # Tauri command handlers
│   ├── audio.rs, models.rs, transcription.rs, history.rs
│   ├── active_listening.rs, ask_ai.rs, rag.rs, suggestions.rs
├── settings/                 # Configuration modules
│   ├── mod.rs               # Main AppSettings struct
│   ├── active_listening.rs, ask_ai.rs, knowledge_base.rs
├── audio_toolkit/            # Low-level audio processing
│   ├── audio/               # Recording, resampling, loopback, mixer
│   ├── vad/                 # Voice Activity Detection (Silero)
│   └── diarization/         # Speaker identification
├── shortcut/                 # Global keyboard shortcuts
├── utils/                    # SafeLock, clipboard, overlay helpers
├── tray.rs                   # System tray management
├── actions.rs                # Transcription orchestration
└── transcription_coordinator.rs  # Pipeline state machine
```

### Frontend Structure (`src/`)

```
src/
├── App.tsx                   # Root component, onboarding flow
├── bindings.ts               # Auto-generated Tauri bindings (DO NOT EDIT)
├── components/
│   ├── ui/                  # Reusable components (Button, Dropdown, etc.)
│   ├── settings/            # Feature settings (70+ files)
│   │   ├── general/, advanced/, post-processing/
│   │   ├── active-listening/, ask-ai/, knowledge-base/
│   ├── onboarding/          # First-run experience
│   └── model-selector/      # Model management UI
├── stores/
│   ├── settingsStore.ts     # Settings state + Rust sync
│   ├── modelStore.ts        # Model availability, downloads
│   └── errorStore.ts        # Global error handling
├── hooks/
│   ├── useSettings.ts       # Settings CRUD hook
│   ├── useSettingsSearch.ts # Fuzzy search (Fuse.js)
│   └── useOsType.ts         # Platform detection
├── i18n/
│   ├── index.ts             # i18next setup
│   ├── languages.ts         # Language metadata (17 locales)
│   └── locales/             # Translation files
└── lib/
    ├── errors/              # Error types, normalization
    └── utils/               # Format, RTL, toast helpers
```

---

## Key Patterns

### 1. Manager Pattern (Rust)

Managers encapsulate domain logic and are initialized at app startup.

**Location**: `src-tauri/src/managers/`

```rust
// src-tauri/src/managers/audio.rs
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};
use crate::error::HandyError;
use crate::utils::SafeLock;

pub struct AudioRecordingManager {
    recorder: Arc<Mutex<Option<AudioRecorder>>>,
    app_handle: AppHandle,
}

impl AudioRecordingManager {
    pub fn new(app_handle: &AppHandle) -> Result<Self, HandyError> {
        // Initialize resources, create directories
        Ok(Self {
            recorder: Arc::new(Mutex::new(None)),
            app_handle: app_handle.clone(),
        })
    }

    pub fn start_recording(&self) -> Result<(), HandyError> {
        let mut recorder = self.recorder.safe_lock()?;
        // ... recording logic
        Ok(())
    }
}

// src-tauri/src/lib.rs - Initialization
fn initialize_core_logic(app_handle: &AppHandle) {
    let recording_manager = Arc::new(
        AudioRecordingManager::new(app_handle)
            .expect("Failed to initialize recording manager"),
    );
    app_handle.manage(recording_manager.clone());
}
```

**Managers in this project:**

| Manager                  | Purpose                              | File                            |
| ------------------------ | ------------------------------------ | ------------------------------- |
| `AudioRecordingManager`  | Audio recording, device enumeration  | `managers/audio.rs`             |
| `ModelManager`           | Model downloads, info, caching       | `managers/model.rs`             |
| `TranscriptionManager`   | Speech-to-text with lazy loading     | `managers/transcription.rs`     |
| `HistoryManager`         | SQLite transcription history         | `managers/history.rs`           |
| `ActiveListeningManager` | Continuous transcription + Ollama AI | `managers/active_listening.rs`  |
| `AskAiManager`           | Multi-turn voice conversations       | `managers/ask_ai.rs`            |
| `AskAiHistoryManager`    | Conversation persistence             | `managers/ask_ai_history.rs`    |
| `RagManager`             | Vector embeddings, semantic search   | `managers/rag.rs`               |
| `SuggestionEngine`       | Quick responses, LLM suggestions     | `managers/suggestion_engine.rs` |

### 2. Command Pattern (Rust)

Commands expose manager functionality to the frontend.

**Location**: `src-tauri/src/commands/`

```rust
// src-tauri/src/commands/audio.rs
use crate::managers::audio::AudioRecordingManager;
use std::sync::Arc;
use tauri::{AppHandle, State};

#[derive(Clone, serde::Serialize, specta::Type)]
pub struct MicrophoneInfo {
    pub name: String,
    pub is_default: bool,
}

#[tauri::command]
#[specta::specta]
pub fn get_available_microphones(
    manager: State<'_, Arc<AudioRecordingManager>>,
) -> Result<Vec<MicrophoneInfo>, String> {
    manager.get_available_microphones().map_err(|e| e.to_string())
}

#[tauri::command]
#[specta::specta]
pub async fn set_selected_microphone(
    app: AppHandle,
    manager: State<'_, Arc<AudioRecordingManager>>,
    device_name: String,
) -> Result<(), String> {
    manager.set_microphone(&device_name).map_err(|e| e.to_string())?;

    // Update settings
    let mut settings = crate::settings::get_settings(&app);
    settings.selected_microphone = Some(device_name);
    crate::settings::write_settings(&app, settings);

    Ok(())
}

// Register in lib.rs specta_builder:
// commands::audio::get_available_microphones,
// commands::audio::set_selected_microphone,
```

**Naming conventions:**

- `get_*` - Retrieve data
- `set_*` / `update_*` - Modify data
- `change_*_setting` - Update a setting with persistence
- `delete_*` / `remove_*` - Delete data

### 3. Error Handling Pattern (Rust)

**Location**: `src-tauri/src/error.rs`

```rust
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum ErrorCategory {
    Settings,
    Audio,
    Model,
    Transcription,
    Network,
    Validation,
    State,
    Filesystem,
    Permission,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct HandyError {
    pub category: ErrorCategory,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
    pub suggestion: Option<String>,
}

impl HandyError {
    pub fn audio(message: &str) -> Self {
        Self {
            category: ErrorCategory::Audio,
            message: message.to_string(),
            details: None,
            recoverable: false,
            suggestion: None,
        }
    }

    pub fn with_details(mut self, details: impl ToString) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestion = Some(suggestion.to_string());
        self
    }
}

// Usage example
pub fn load_model(&self, model_id: &str) -> Result<(), HandyError> {
    let model_path = self.get_model_path(model_id)
        .ok_or_else(|| HandyError::model("Model not found")
            .with_details(format!("Model ID: {}", model_id))
            .with_suggestion("Download the model first"))?;

    std::fs::read(&model_path)
        .map_err(|e| HandyError::filesystem("Failed to read model file")
            .with_details(e.to_string())
            .recoverable())?;

    Ok(())
}
```

### 4. Settings Pattern (Rust)

**Location**: `src-tauri/src/settings/mod.rs`

```rust
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const SETTINGS_STORE_PATH: &str = "settings_store.json";

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type)]
pub struct AppSettings {
    #[serde(default = "default_push_to_talk")]
    pub push_to_talk: bool,

    #[serde(default = "default_audio_feedback")]
    pub audio_feedback: bool,

    #[serde(default)]
    pub selected_microphone: Option<String>,

    // ... more fields
}

fn default_push_to_talk() -> bool { false }
fn default_audio_feedback() -> bool { true }

pub fn get_default_settings() -> AppSettings {
    AppSettings {
        push_to_talk: default_push_to_talk(),
        audio_feedback: default_audio_feedback(),
        selected_microphone: None,
        // ...
    }
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    let store = app.store(SETTINGS_STORE_PATH).unwrap();
    store.get("settings")
        .and_then(|v| serde_json::from_value::<AppSettings>(v).ok())
        .unwrap_or_else(get_default_settings)
}

pub fn write_settings(app: &AppHandle, settings: AppSettings) {
    let store = app.store(SETTINGS_STORE_PATH).unwrap();
    store.set("settings", serde_json::to_value(&settings).unwrap());
}
```

### 5. State Management Pattern (Rust)

```rust
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use crate::utils::SafeLock;

pub struct TranscriptionManager {
    // Exclusive access (write-heavy)
    engine: Arc<Mutex<Option<LoadedEngine>>>,

    // Read-heavy, occasional writes
    settings: Arc<RwLock<TranscriptionSettings>>,

    // Lock-free atomics for signals
    shutdown_signal: Arc<AtomicBool>,
    last_activity: Arc<AtomicU64>,
}

impl TranscriptionManager {
    pub fn is_model_loaded(&self) -> bool {
        // Use SafeLock trait for poison recovery
        self.engine.safe_lock()
            .map(|e| e.is_some())
            .unwrap_or(false)
    }

    pub fn shutdown(&self) {
        // Atomic signal - no lock needed
        self.shutdown_signal.store(true, Ordering::Relaxed);
    }
}
```

### 6. Event Pattern (Rust → Frontend)

```rust
// Backend: Emit events
use tauri::{AppHandle, Emitter};

#[derive(Clone, serde::Serialize, specta::Type)]
struct ModelStateEvent {
    event_type: String,
    model_id: Option<String>,
    error: Option<String>,
}

fn emit_model_loaded(app: &AppHandle, model_id: &str) {
    let _ = app.emit("model-state-changed", ModelStateEvent {
        event_type: "loaded".to_string(),
        model_id: Some(model_id.to_string()),
        error: None,
    });
}
```

```typescript
// Frontend: Listen for events
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

interface ModelStateEvent {
  event_type: string;
  model_id: string | null;
  error: string | null;
}

function useModelEvents() {
  useEffect(() => {
    const unlisten = listen<ModelStateEvent>("model-state-changed", (event) => {
      console.log("Model state:", event.payload);
      if (event.payload.event_type === "loaded") {
        // Handle model loaded
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);
}
```

**Events in this project:**

| Event                      | Payload                           | Purpose                 |
| -------------------------- | --------------------------------- | ----------------------- |
| `model-state-changed`      | `{ event_type, model_id, error }` | Model loaded/unloaded   |
| `model-download-progress`  | `{ model_id, downloaded, total }` | Download progress       |
| `audio-level`              | `number`                          | Recording visualization |
| `transcription-complete`   | `{ text, model_id }`              | Transcription finished  |
| `active-listening-insight` | `{ segment_id, insight }`         | AI insight generated    |
| `ask-ai-response`          | `{ response, turn_id }`           | Conversation response   |

### 7. Component Pattern (React)

**Location**: `src/components/settings/`

```tsx
// src/components/settings/MicrophoneSelector.tsx
import React from "react";
import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/useSettings";
import { SettingContainer } from "@/components/ui/SettingContainer";
import { Dropdown } from "@/components/ui/Dropdown";
import { ResetButton } from "@/components/ui/ResetButton";

interface MicrophoneSelectorProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const MicrophoneSelector: React.FC<MicrophoneSelectorProps> = React.memo(
  ({ descriptionMode = "tooltip", grouped = false }) => {
    const { t } = useTranslation();
    const {
      getSetting,
      updateSetting,
      resetSetting,
      isUpdating,
      audioDevices,
    } = useSettings();

    const selectedMicrophone = getSetting("selected_microphone") || "Default";
    const isLoading = isUpdating("selected_microphone");

    const handleSelect = async (deviceName: string) => {
      await updateSetting("selected_microphone", deviceName);
    };

    const handleReset = async () => {
      await resetSetting("selected_microphone");
    };

    return (
      <SettingContainer
        title={t("settings.sound.microphone.title")}
        description={t("settings.sound.microphone.description")}
        descriptionMode={descriptionMode}
        grouped={grouped}
      >
        <div className="flex items-center space-x-1">
          <Dropdown
            options={audioDevices.map((d) => ({
              value: d.name,
              label: d.name,
            }))}
            value={selectedMicrophone}
            onChange={handleSelect}
            disabled={isLoading}
          />
          <ResetButton onReset={handleReset} disabled={isLoading} />
        </div>
      </SettingContainer>
    );
  },
);
```

### 8. Store Pattern (Zustand)

**Location**: `src/stores/settingsStore.ts`

```typescript
import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { commands, type Settings } from "@/bindings";

interface SettingsStore {
  settings: Settings | null;
  isLoading: boolean;
  isUpdating: Record<string, boolean>;

  initialize: () => Promise<void>;
  updateSetting: <K extends keyof Settings>(
    key: K,
    value: Settings[K],
  ) => Promise<void>;
}

export const useSettingsStore = create<SettingsStore>()(
  subscribeWithSelector((set, get) => ({
    settings: null,
    isLoading: true,
    isUpdating: {},

    initialize: async () => {
      const result = await commands.getAppSettings();
      if (result.status === "ok") {
        set({ settings: result.data, isLoading: false });
      }
    },

    updateSetting: async (key, value) => {
      const prev = get().settings;

      // Optimistic update
      set((state) => ({
        settings: state.settings ? { ...state.settings, [key]: value } : null,
        isUpdating: { ...state.isUpdating, [key]: true },
      }));

      try {
        // Call backend handler
        const handler = settingUpdaters[key];
        if (handler) {
          await handler(value);
        }
      } catch (error) {
        // Rollback on failure
        set({ settings: prev });
        throw error;
      } finally {
        set((state) => ({
          isUpdating: { ...state.isUpdating, [key]: false },
        }));
      }
    },
  })),
);
```

### 9. Hook Pattern (React)

**Location**: `src/hooks/useSettings.ts`

```typescript
import { useCallback } from "react";
import { useSettingsStore } from "@/stores/settingsStore";
import type { Settings } from "@/bindings";

export function useSettings() {
  const store = useSettingsStore();

  const getSetting = useCallback(
    <K extends keyof Settings>(key: K) => {
      return store.settings?.[key];
    },
    [store.settings],
  );

  const updateSetting = useCallback(
    async <K extends keyof Settings>(key: K, value: Settings[K]) => {
      await store.updateSetting(key, value);
    },
    [store.updateSetting],
  );

  const isUpdating = useCallback(
    (key: string) => {
      return store.isUpdating[key] ?? false;
    },
    [store.isUpdating],
  );

  return {
    settings: store.settings,
    isLoading: store.isLoading,
    getSetting,
    updateSetting,
    isUpdating,
    refreshSettings: store.initialize,
  };
}
```

### 10. Tauri Integration Pattern

**Location**: `src/bindings.ts` (auto-generated)

```typescript
import { commands } from "@/bindings";

// Pattern: Always check result status
async function loadModels() {
  const result = await commands.getAvailableModels();

  if (result.status === "ok") {
    // TypeScript knows result.data is ModelInfo[]
    return result.data;
  } else {
    // TypeScript knows result.error is string
    console.error("Failed to load models:", result.error);
    throw new Error(result.error);
  }
}

// Pattern: With error store
import { useErrorStore } from "@/stores/errorStore";

function useModelDownload() {
  const { addError } = useErrorStore();

  const downloadModel = async (modelId: string) => {
    const result = await commands.downloadModel(modelId);
    if (result.status === "error") {
      addError(result.error, "Model download failed");
    }
  };

  return { downloadModel };
}
```

---

## Fork-Exclusive Features

This fork includes features not present in upstream `cjpais/Handy`:

### Active Listening

Continuous transcription with AI-generated insights via Ollama.

**Files:**

- `src-tauri/src/managers/active_listening.rs` (~1,367 lines)
- `src-tauri/src/commands/active_listening.rs`
- `src-tauri/src/settings/active_listening.rs`
- `src/components/settings/active-listening/`

**Key Commands:**

- `start_active_listening_session` - Begin continuous transcription
- `stop_active_listening_session` - Stop session, generate summary
- `get_active_listening_state` - Get current session state
- `check_ollama_connection` - Verify Ollama is running
- `fetch_ollama_models` - List available Ollama models

### Ask AI

Multi-turn voice conversations with local LLM.

**Files:**

- `src-tauri/src/managers/ask_ai.rs` (~583 lines)
- `src-tauri/src/managers/ask_ai_history.rs` (~545 lines)
- `src-tauri/src/commands/ask_ai.rs`
- `src/components/settings/ask-ai/`

**Key Commands:**

- `get_ask_ai_state` - Current conversation state
- `can_start_ask_ai_recording` - Check if ready for input
- `start_new_ask_ai_conversation` - Begin new conversation
- `list_ask_ai_conversations` - Get conversation history
- `delete_ask_ai_conversation_from_history` - Remove conversation

### RAG Knowledge Base

Vector embeddings and semantic search for contextual responses.

**Files:**

- `src-tauri/src/managers/rag.rs` (~590 lines)
- `src-tauri/src/commands/rag.rs`
- `src-tauri/src/settings/knowledge_base.rs`
- `src/components/settings/knowledge-base/`

**Key Commands:**

- `rag_add_document` - Index a document
- `rag_search` - Semantic search
- `rag_delete_document` - Remove document
- `rag_list_documents` - List all indexed documents
- `rag_get_stats` - Get index statistics

### Suggestion Engine

Context-aware quick responses and AI suggestions.

**Files:**

- `src-tauri/src/managers/suggestion_engine.rs` (~539 lines)
- `src-tauri/src/commands/suggestions.rs`
- `src-tauri/src/settings/suggestions.rs`

**Key Commands:**

- `get_suggestions_settings` - Get suggestion config
- `get_quick_responses` - List quick responses
- `add_quick_response` - Add new quick response
- `toggle_quick_response` - Enable/disable response

### Ollama Integration

Streaming LLM client for all AI features.

**File:** `src-tauri/src/ollama_client.rs` (~533 lines)

```rust
pub struct OllamaClient {
    client: reqwest::Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Result<Self, String>;

    pub async fn stream_generate(
        &self,
        model: &str,
        prompt: &str,
        tx: mpsc::Sender<String>,
    ) -> Result<(), String>;

    pub async fn get_embeddings(
        &self,
        model: &str,
        prompt: &str,
    ) -> Result<Vec<f32>, String>;
}
```

---

## Internationalization

All user-facing strings use i18next.

**Adding a translation:**

1. Add key to `src/i18n/locales/en/translation.json`
2. Use in component: `const { t } = useTranslation(); t('key.path')`

**Supported locales (17):**

```
ar, cs, de, en, es, fr, it, ja, ko, pl, pt, ru, tr, uk, vi, zh, zh-TW
```

ESLint enforces no hardcoded strings via `eslint-plugin-i18next`.

---

## CLI Parameters

| Flag                     | Description                           |
| ------------------------ | ------------------------------------- |
| `--toggle-transcription` | Toggle recording on/off               |
| `--toggle-post-process`  | Toggle recording with post-processing |
| `--cancel`               | Cancel current operation              |
| `--start-hidden`         | Launch without main window            |
| `--no-tray`              | Launch without tray icon              |
| `--debug`                | Enable verbose logging                |

---

## Code Style

### Rust

- Run `cargo fmt` and `cargo clippy` before committing
- Use `HandyError` with categories, not plain strings
- Use `SafeLock`/`SafeRwLock` instead of `.unwrap()` on locks
- Document public APIs with `///` comments

### TypeScript/React

- Strict TypeScript, avoid `any`
- Functional components with hooks
- Wrap settings components with `React.memo()`
- Path aliases: `@/` maps to `./src/`
- Import order: external → tauri → stores/hooks → components → utils

### Commits

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation
- `refactor:` code refactoring
- `test:` tests
- `chore:` maintenance

---

## Debugging

**Debug Mode:** `Cmd+Shift+D` (macOS) / `Ctrl+Shift+D` (Windows/Linux)

**Rust Logging:**

```bash
RUST_LOG=debug bun run tauri dev
RUST_LOG=handy=trace bun run tauri dev
```

---

## Platform Notes

- **macOS**: Metal acceleration, accessibility permissions required
- **Windows**: Vulkan acceleration, code signing
- **Linux**: OpenBLAS + Vulkan, Wayland support via wtype/dotool/ydotool

---

## Adding New Features

### Adding a New Manager

1. Create `src-tauri/src/managers/my_feature.rs`
2. Add `pub mod my_feature;` to `managers/mod.rs`
3. Initialize in `lib.rs`:
   ```rust
   let my_manager = Arc::new(MyFeatureManager::new(app_handle)?);
   app_handle.manage(my_manager);
   ```

### Adding a New Command

1. Create function in `src-tauri/src/commands/my_feature.rs`
2. Add `pub mod my_feature;` to `commands/mod.rs`
3. Register in `lib.rs` specta_builder
4. Rebuild to regenerate `src/bindings.ts`

### Adding a New Setting

1. Add field to `AppSettings` with `#[serde(default = "default_fn")]`
2. Create default function
3. Add command handler in `shortcut/mod.rs`
4. Register command in `lib.rs`

### Adding a Frontend Component

1. Create `src/components/settings/MySetting.tsx`
2. Export from `src/components/settings/index.ts`
3. Add to settings group
4. Add translation keys

---

_See also: [CONTRIBUTING.md](./CONTRIBUTING.md), [ARCHITECTURE.md](./ARCHITECTURE.md), [FEATURES.md](./FEATURES.md)_
