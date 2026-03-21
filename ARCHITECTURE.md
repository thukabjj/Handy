# Architecture

This document describes the system architecture of Handy, a cross-platform desktop speech-to-text application built with Tauri 2.x.

---

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Handy Application                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Frontend (React/TypeScript)                   │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │   Settings  │  │   Models    │  │   History   │  │   Overlay   │ │   │
│  │  │     UI      │  │  Selector   │  │   Viewer    │  │   Window    │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Zustand Stores                             │ │   │
│  │  │  settingsStore  │  modelStore  │  errorStore                    │ │   │
│  │  └─────────────────────────────────────────────────────────────────┘ │   │
│  └──────────────────────────────┬──────────────────────────────────────┘   │
│                                 │ Tauri IPC                                │
│                                 ▼                                          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Backend (Rust/Tauri)                         │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │   Audio     │  │   Model     │  │Transcription│  │   History   │ │   │
│  │  │  Manager    │  │  Manager    │  │  Manager    │  │   Manager   │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │   │
│  │  │  Active     │  │   Ask AI    │  │    RAG      │  │ Suggestion  │ │   │
│  │  │ Listening   │  │  Manager    │  │  Manager    │  │   Engine    │ │   │
│  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘ │   │
│  │  ┌─────────────────────────────────────────────────────────────────┐ │   │
│  │  │                      Audio Toolkit                              │ │   │
│  │  │  Recorder  │  VAD  │  Diarization  │  Loopback  │  Mixer       │ │   │
│  │  └─────────────────────────────────────────────────────────────────┘ │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                            External Services                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│  │   Ollama    │  │  OpenAI/    │  │  Clipboard  │  │  System Audio   │   │
│  │  (Local)    │  │  Anthropic  │  │   System    │  │    Devices      │   │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Technology Stack

| Layer            | Technology            | Purpose                           |
| ---------------- | --------------------- | --------------------------------- |
| **Frontend**     | React 18 + TypeScript | UI components, state management   |
| **UI Framework** | Tailwind CSS          | Styling, responsive design        |
| **State**        | Zustand               | Global state management           |
| **Build**        | Vite                  | Fast frontend bundling            |
| **Desktop**      | Tauri 2.x             | Native desktop shell, IPC         |
| **Backend**      | Rust                  | Core business logic, performance  |
| **Audio**        | CPAL + Whisper        | Recording, transcription          |
| **VAD**          | Silero VAD (ONNX)     | Voice activity detection          |
| **Database**     | SQLite                | History, conversation storage     |
| **LLM**          | Ollama                | Local AI inference                |
| **i18n**         | i18next               | Internationalization (17 locales) |

---

## Data Flow

### Transcription Pipeline

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│Microphone│ -> │ Audio    │ -> │   VAD    │ -> │Transcri- │ -> │ Clipboard│
│  Input   │    │ Recorder │    │ (Silero) │    │  ption   │    │  /Paste  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
                     │                              │
                     │                              ▼
                     │              ┌─────────────────────────────┐
                     │              │   Post-Processing (LLM)     │
                     │              │   - Grammar correction      │
                     │              │   - Formatting              │
                     │              └─────────────────────────────┘
                     │                              │
                     ▼                              ▼
              ┌──────────┐                  ┌──────────┐
              │ History  │ <--------------- │  Output  │
              │  Storage │                  │   Text   │
              └──────────┘                  └──────────┘
```

**Detailed Steps:**

1. **Audio Capture**: `AudioRecorder` captures PCM audio from selected input device
2. **VAD Filtering**: Silero VAD model detects speech segments, filters silence
3. **Resampling**: Audio resampled to 16kHz mono for transcription models
4. **Transcription**: Whisper/Parakeet/Moonshine/SenseVoice converts speech to text
5. **Post-Processing**: Optional LLM cleanup via OpenAI/Anthropic/Ollama
6. **Output**: Text sent to clipboard and/or typed into active application
7. **Storage**: Transcription saved to SQLite history database

### Active Listening Pipeline

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         Active Listening Mode                            │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐                  │
│  │  Microphone │ -> │   Audio     │ -> │   Speech    │                  │
│  │    Input    │    │   Buffer    │    │  Detection  │                  │
│  └─────────────┘    └─────────────┘    └─────────────┘                  │
│         +                                    │                           │
│  ┌─────────────┐                             ▼                           │
│  │   System    │ -> ┌─────────────┐    ┌─────────────┐                  │
│  │   Audio     │    │   Mixer     │    │Transcription│                  │
│  │  Loopback   │    │             │    │  (Whisper)  │                  │
│  └─────────────┘    └─────────────┘    └─────────────┘                  │
│                                              │                           │
│                                              ▼                           │
│                     ┌────────────────────────────────────┐              │
│                     │      Transcript Buffer             │              │
│                     │  (Rolling window of speech)        │              │
│                     └────────────────────────────────────┘              │
│                                              │                           │
│                                              ▼                           │
│                     ┌────────────────────────────────────┐              │
│                     │         Ollama LLM                 │              │
│                     │  - Insight generation              │              │
│                     │  - Context analysis                │              │
│                     │  - Summary creation                │              │
│                     └────────────────────────────────────┘              │
│                                              │                           │
│                                              ▼                           │
│                     ┌────────────────────────────────────┐              │
│                     │         Insight Event              │              │
│                     │  -> Frontend notification          │              │
│                     │  -> History storage                │              │
│                     └────────────────────────────────────┘              │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Ask AI Conversation Flow

```
┌─────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  User   │ -> │    Audio    │ -> │Transcription│ -> │   Ollama    │
│  Speaks │    │  Recording  │    │             │    │    LLM      │
└─────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                            │
                                                            ▼
                                                     ┌─────────────┐
                                                     │    RAG      │
                                                     │   Context   │
                                                     │   (Optional)│
                                                     └─────────────┘
                                                            │
                                                            ▼
┌─────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  User   │ <- │     TTS     │ <- │  Response   │ <- │  AI Reply   │
│  Hears  │    │  (Optional) │    │   Display   │    │  Generation │
└─────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

---

## Component Architecture

### Backend Components

#### Manager Layer

Managers encapsulate business logic and maintain state:

| Manager                  | Responsibility                      | Key State                        |
| ------------------------ | ----------------------------------- | -------------------------------- |
| `AudioRecordingManager`  | Device selection, recording control | Active device, recording state   |
| `ModelManager`           | Model downloads, lifecycle          | Downloaded models, active model  |
| `TranscriptionManager`   | Speech-to-text processing           | Loaded engine, processing queue  |
| `HistoryManager`         | Transcription persistence           | SQLite connection                |
| `ActiveListeningManager` | Continuous transcription + insights | Session state, transcript buffer |
| `AskAiManager`           | Voice conversation control          | Conversation state               |
| `AskAiHistoryManager`    | Conversation persistence            | SQLite connection                |
| `RagManager`             | Document indexing, vector search    | Index state, embeddings          |
| `SuggestionEngine`       | Context-aware suggestions           | Active suggestions               |

#### Command Layer

Commands expose manager functionality to frontend:

```
Frontend                        Backend
   │                               │
   │  commands.startRecording()    │
   │  ─────────────────────────►   │
   │                               ├──► AudioRecordingManager.start()
   │                               │
   │  ◄───────────────────────────┤
   │       Result<(), String>      │
   │                               │
   │  ◄─ Event: "audio-level"      │
   │  ◄─ Event: "transcription-complete"
```

#### Audio Toolkit

Low-level audio processing components:

```
audio_toolkit/
├── audio/
│   ├── device.rs       # Device enumeration
│   ├── recorder.rs     # Audio capture
│   ├── resampler.rs    # Sample rate conversion
│   ├── loopback.rs     # System audio capture (fork)
│   └── mixer.rs        # Multi-source mixing (fork)
├── vad/
│   └── silero.rs       # Voice activity detection
└── diarization/
    └── mod.rs          # Speaker identification
```

### Frontend Components

#### Component Hierarchy

```
App.tsx
├── OnboardingFlow/
│   ├── WelcomeStep
│   ├── PermissionsStep
│   └── ModelDownloadStep
└── SettingsWindow/
    ├── SettingsNavigation
    └── SettingsContent/
        ├── general/
        │   ├── LanguageSelector
        │   ├── ThemeSelector
        │   └── ShortcutSettings
        ├── models/
        │   ├── ModelSelector
        │   └── ModelDownloader
        ├── active-listening/    (fork)
        │   ├── SessionControls
        │   └── InsightsList
        ├── ask-ai/             (fork)
        │   ├── ConversationView
        │   └── HistoryPanel
        ├── knowledge-base/     (fork)
        │   ├── DocumentList
        │   └── SearchInterface
        └── history/
            └── TranscriptionList
```

#### State Management

```
┌─────────────────────────────────────────────────────────────────┐
│                        Zustand Stores                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   settingsStore                          │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │   │
│  │  │  settings   │  │   loading   │  │  updateSetting  │  │   │
│  │  │   object    │  │    state    │  │    function     │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │   │
│  │         │                                                │   │
│  │         ▼ Sync                                           │   │
│  │  ┌─────────────────────────────────────────────────────┐│   │
│  │  │              Tauri Backend State                    ││   │
│  │  │         (tauri-plugin-store persistence)            ││   │
│  │  └─────────────────────────────────────────────────────┘│   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐             │
│  │     modelStore      │  │     errorStore      │             │
│  │  - availableModels  │  │  - currentError     │             │
│  │  - downloadProgress │  │  - errorHistory     │             │
│  │  - activeModel      │  │  - setError         │             │
│  └─────────────────────┘  └─────────────────────┘             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Event System

### Backend to Frontend Events

Events flow from Rust to React via Tauri's event system:

| Event                      | Payload                                  | Trigger              |
| -------------------------- | ---------------------------------------- | -------------------- |
| `audio-level`              | `{ level: number }`                      | During recording     |
| `transcription-progress`   | `{ progress: number }`                   | During processing    |
| `transcription-complete`   | `{ text: string, duration: number }`     | After transcription  |
| `model-state-changed`      | `{ modelId: string, state: string }`     | Model download/load  |
| `active-listening-insight` | `{ insight: string, timestamp: number }` | AI generates insight |
| `ask-ai-response`          | `{ text: string, streaming: boolean }`   | LLM response chunk   |
| `suggestions-updated`      | `Suggestion[]`                           | Context changes      |
| `error`                    | `{ message: string, category: string }`  | Error occurred       |

### Frontend Event Handling

```typescript
// Hook-based event listening
useEffect(() => {
  const unlisten = listen<TranscriptionPayload>(
    "transcription-complete",
    (event) => {
      setTranscription(event.payload.text);
    },
  );

  return () => {
    unlisten.then((fn) => fn());
  };
}, []);
```

---

## Persistence Layer

### SQLite Databases

**history.db** - Transcription history

```sql
CREATE TABLE transcriptions (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    duration_ms INTEGER,
    model_id TEXT,
    created_at TEXT NOT NULL,
    metadata TEXT  -- JSON for extensibility
);

CREATE INDEX idx_transcriptions_created_at ON transcriptions(created_at DESC);
```

**ask_ai_history.db** - Conversation history

```sql
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    title TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    role TEXT NOT NULL,  -- 'user' | 'assistant'
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id)
);

CREATE INDEX idx_messages_conversation ON messages(conversation_id);
```

### Settings Storage

Settings persisted via `tauri-plugin-store`:

```
~/.config/handy/settings.json
├── language: string
├── theme: "light" | "dark" | "system"
├── shortcut: { modifiers: string[], key: string }
├── inputDevice: string | null
├── outputDevice: string | null
├── activeModel: string
├── postProcessing: PostProcessingSettings
├── activeListening: ActiveListeningSettings
├── askAi: AskAiSettings
├── knowledgeBase: KnowledgeBaseSettings
└── suggestions: SuggestionSettings
```

---

## Security Architecture

### Permission Model

| Permission    | Platform | Purpose                |
| ------------- | -------- | ---------------------- |
| Microphone    | All      | Audio recording        |
| Accessibility | macOS    | Text paste automation  |
| Notification  | All      | Transcription alerts   |
| File System   | All      | Model storage, history |

### Data Privacy

- **Local Processing**: All transcription happens on-device
- **No Network Required**: Core features work offline
- **Optional Cloud**: Post-processing can use OpenAI/Anthropic (opt-in)
- **Local LLM**: Ollama integration keeps AI features private

### API Key Storage

```
                          ┌─────────────────────┐
                          │   Settings Store    │
                          │  (encrypted JSON)   │
                          └─────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────┐
            │  OpenAI   │   │ Anthropic │   │  Ollama   │
            │  API Key  │   │  API Key  │   │   URL     │
            └───────────┘   └───────────┘   └───────────┘
```

---

## Platform-Specific Architecture

### macOS

```
┌────────────────────────────────────────┐
│              Handy.app                 │
├────────────────────────────────────────┤
│  WebView (WKWebView)                   │
│    └── React Frontend                  │
├────────────────────────────────────────┤
│  Native Layer                          │
│  ├── Metal Acceleration (Whisper)      │
│  ├── Core Audio (Recording)            │
│  ├── Accessibility API (Paste)         │
│  └── Menu Bar Integration              │
└────────────────────────────────────────┘
```

### Windows

```
┌────────────────────────────────────────┐
│              Handy.exe                 │
├────────────────────────────────────────┤
│  WebView2 (Chromium)                   │
│    └── React Frontend                  │
├────────────────────────────────────────┤
│  Native Layer                          │
│  ├── Vulkan/DirectML (Whisper)         │
│  ├── WASAPI (Recording)                │
│  ├── SendInput (Paste)                 │
│  └── System Tray Integration           │
└────────────────────────────────────────┘
```

### Linux

```
┌────────────────────────────────────────┐
│              handy                     │
├────────────────────────────────────────┤
│  WebKitGTK                             │
│    └── React Frontend                  │
├────────────────────────────────────────┤
│  Native Layer                          │
│  ├── OpenBLAS/Vulkan (Whisper)         │
│  ├── ALSA/PulseAudio (Recording)       │
│  ├── wtype/dotool/ydotool (Paste)      │
│  └── gtk-layer-shell (Overlay)         │
└────────────────────────────────────────┘
```

---

## Error Handling Architecture

### Error Categories

```rust
pub enum ErrorCategory {
    Settings,      // Configuration errors
    Audio,         // Recording/playback issues
    Model,         // Model loading/inference
    Transcription, // Processing failures
    Network,       // API/download errors
    Validation,    // Input validation
    State,         // Invalid state transitions
    Filesystem,    // File I/O errors
    Permission,    // Missing permissions
    Unknown,       // Unexpected errors
}
```

### Error Flow

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Manager   │ -> │ HandyError  │ -> │  Command    │ -> │  Frontend   │
│   Operation │    │  Creation   │    │  Response   │    │  Display    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                          │
                          ▼
                   ┌─────────────┐
                   │  Structured │
                   │    Error    │
                   │  - message  │
                   │  - category │
                   │  - details  │
                   │  - suggestion│
                   │  - recoverable│
                   └─────────────┘
```

---

## Single Instance Architecture

Handy enforces single-instance behavior:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Launch                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   First Launch                     Subsequent Launch            │
│   ┌─────────────┐                  ┌─────────────┐             │
│   │   Check     │                  │   Check     │             │
│   │   Lockfile  │                  │   Lockfile  │             │
│   └──────┬──────┘                  └──────┬──────┘             │
│          │ Not exists                      │ Exists            │
│          ▼                                 ▼                   │
│   ┌─────────────┐                  ┌─────────────┐             │
│   │   Create    │                  │   Send IPC  │             │
│   │   Lockfile  │                  │   to First  │             │
│   └──────┬──────┘                  └──────┬──────┘             │
│          ▼                                 ▼                   │
│   ┌─────────────┐                  ┌─────────────┐             │
│   │   Start     │                  │   Exit      │             │
│   │   Normally  │                  │             │             │
│   └──────┬──────┘                  └─────────────┘             │
│          ▼                                                      │
│   ┌─────────────┐     IPC      ┌─────────────┐                 │
│   │   Listen    │ <─────────── │  Focus      │                 │
│   │   for IPC   │              │  Window     │                 │
│   └─────────────┘              └─────────────┘                 │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Build Pipeline

```
┌─────────────────────────────────────────────────────────────────┐
│                        Build Process                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   Source Code                                                   │
│   ┌─────────────┐  ┌─────────────┐                             │
│   │   src/      │  │ src-tauri/  │                             │
│   │ (React/TS)  │  │   (Rust)    │                             │
│   └──────┬──────┘  └──────┬──────┘                             │
│          │                 │                                    │
│          ▼                 ▼                                    │
│   ┌─────────────┐  ┌─────────────┐                             │
│   │    Vite     │  │    Cargo    │                             │
│   │   Bundle    │  │   Compile   │                             │
│   └──────┬──────┘  └──────┬──────┘                             │
│          │                 │                                    │
│          └────────┬────────┘                                   │
│                   ▼                                             │
│           ┌─────────────┐                                       │
│           │   Tauri     │                                       │
│           │   Bundle    │                                       │
│           └──────┬──────┘                                       │
│                  │                                              │
│    ┌─────────────┼─────────────┐                               │
│    ▼             ▼             ▼                               │
│ ┌───────┐   ┌───────┐   ┌───────┐                             │
│ │ .dmg  │   │ .msi  │   │ .deb  │                             │
│ │(macOS)│   │ (Win) │   │(Linux)│                             │
│ └───────┘   └───────┘   └───────┘                             │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Performance Considerations

### Audio Processing

- **Sample Rate**: Input resampled to 16kHz for models
- **Buffer Size**: 30ms chunks for low latency
- **VAD Threshold**: Configurable silence detection
- **Memory**: Audio buffers managed to prevent leaks

### Transcription

- **Model Loading**: Lazy load, keep in memory once loaded
- **Batch Processing**: VAD segments batched for efficiency
- **GPU Acceleration**: Metal (macOS), Vulkan (Windows/Linux)

### Frontend

- **Code Splitting**: Route-based lazy loading
- **Memoization**: `React.memo` for expensive components
- **Virtual Lists**: Large history lists virtualized
- **Optimistic Updates**: UI updates before backend confirmation

---

## Extension Points

### Adding New Transcription Models

1. Implement model loader in `src-tauri/src/audio_toolkit/`
2. Register in `ModelManager`
3. Add UI in `src/components/model-selector/`

### Adding New Post-Processors

1. Implement processor trait in `src-tauri/src/llm_client.rs`
2. Add settings in `src-tauri/src/settings/`
3. Add UI in `src/components/settings/post-processing/`

### Adding New AI Features

1. Create manager in `src-tauri/src/managers/`
2. Add commands in `src-tauri/src/commands/`
3. Register in `lib.rs`
4. Create frontend hook and components

---

## Related Documentation

- [CLAUDE.md](CLAUDE.md) - Developer reference and code patterns
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [FEATURES.md](FEATURES.md) - Feature documentation
- [DEV_DOCS.md](DEV_DOCS.md) - Technical reference
- [BUILD.md](BUILD.md) - Build instructions

---

_Last updated: 2026-03-01_
