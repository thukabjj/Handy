# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

**Prerequisites:** [Rust](https://rustup.rs/) (latest stable via `rust-toolchain.toml`), [Bun](https://bun.sh/)

```bash
# Install all dependencies (frontend + VAD model)
make install

# Run in development mode
make dev
# Or directly:
bun run tauri dev
# If cmake error on macOS:
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# Build for production
bun run tauri build

# Frontend only development
bun run dev        # Start Vite dev server
bun run build      # Build frontend (TypeScript + Vite)
bun run preview    # Preview built frontend
```

**Code Quality:**

```bash
bun run lint              # ESLint for frontend
bun run lint:fix          # ESLint with auto-fix
bun run format            # Prettier + cargo fmt
bun run format:check      # Check formatting without changes
make check                # Full check (Rust + TypeScript)
make lint                 # Run all linters (ESLint + clippy)
```

**Testing:**

```bash
bun run test              # Frontend tests (Vitest)
bun run test:watch        # Frontend tests in watch mode
make test-rust            # Rust tests
make test                 # All tests (Rust + frontend)
```

**Model Setup (Required for Development):**

```bash
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx
```

## Architecture Overview

Handy is a cross-platform desktop speech-to-text application built with Tauri 2.x (Rust backend + React/TypeScript frontend). It supports offline transcription, voice-powered AI conversations, and real-time active listening with local LLM integration via Ollama.

### Backend Structure (src-tauri/src/)

- `lib.rs` - Main entry point, Tauri setup, manager initialization
- `error.rs` - Structured error handling (`HandyError`, `ErrorCategory`)
- `ollama_client.rs` - Streaming Ollama LLM client for Active Listening, Ask AI, and RAG
- `llm_client.rs` - LLM client abstraction
- `managers/` - Core business logic:
  - `audio.rs` - Audio recording and device management
  - `model.rs` - Model downloading and management
  - `transcription.rs` - Speech-to-text processing pipeline
  - `history.rs` - Transcription history storage (SQLite)
  - `active_listening.rs` - Continuous transcription with AI-generated insights
  - `ask_ai.rs` - Multi-turn voice conversations with local LLM
  - `ask_ai_history.rs` - Conversation persistence (SQLite)
  - `rag.rs` - Retrieval-Augmented Generation knowledge base
  - `suggestion_engine.rs` - Context-aware quick responses and suggestions
- `audio_toolkit/` - Low-level audio processing:
  - `audio/` - Device enumeration, recording, resampling
  - `audio/loopback.rs` - System audio capture (loopback recording)
  - `audio/mixer.rs` - Multi-source audio mixing
  - `vad/` - Voice Activity Detection (Silero VAD)
  - `diarization/` - Speaker diarization via energy-based RMS analysis
- `commands/` - Tauri command handlers:
  - `audio.rs`, `models.rs`, `transcription.rs`, `history.rs`
  - `active_listening.rs`, `ask_ai.rs`, `rag.rs`, `suggestions.rs`
- `shortcut/` - Global keyboard shortcut handling (modular: handler, tauri_impl, handy_keys)
- `settings/` - Application settings (general, active_listening, ask_ai, knowledge_base, suggestions)
- `utils/` - Shared utilities (SafeLock/SafeRwLock, clipboard, overlay, tray helpers)
- `tray.rs` - System tray icon management with theme-aware icons
- `actions.rs` - Core transcription action orchestration
- `input.rs` - Cross-platform text input (xdotool, wtype, dotool, enigo)

### Frontend Structure (src/)

- `App.tsx` - Main component with onboarding flow
- `components/settings/` - Settings UI (45+ files organized by feature):
  - `general/`, `advanced/`, `models/`, `history/`
  - `active-listening/` - Active Listening session and settings UI
  - `ask-ai/` - Ask AI conversation history and settings
  - `knowledge-base/` - RAG knowledge base configuration
  - `post-processing/` - LLM post-processing settings
  - `debug/`, `about/`
- `components/model-selector/` - Model management interface
- `components/onboarding/` - First-run experience
- `hooks/useSettings.ts`, `useModels.ts` - State management hooks
- `stores/settingsStore.ts` - Zustand store for settings
- `stores/errorStore.ts` - Global error state
- `stores/modelStore.ts` - Model state management
- `lib/errors/` - Frontend error handling utilities
- `bindings.ts` - Auto-generated Tauri type bindings (via tauri-specta)
- `overlay/` - Recording overlay window code
- `i18n/` - Internationalization (16 languages)

### Key Patterns

**Manager Pattern:** Core functionality organized into managers (Audio, Model, Transcription, ActiveListening, AskAi, AskAiHistory, Rag, SuggestionEngine) initialized at startup and managed via Tauri state.

**Command-Event Architecture:** Frontend communicates with backend via Tauri commands; backend sends updates via events.

**Pipeline Processing:** Audio -> VAD -> Whisper/Parakeet/Moonshine/SenseVoice -> Text output -> Clipboard/Paste

**State Flow:** Zustand -> Tauri Command -> Rust State -> Persistence (tauri-plugin-store)

**Error Handling:** `HandyError` with `ErrorCategory` enum provides structured, user-friendly errors with recovery suggestions. `SafeLock` and `SafeRwLock` traits wrap mutex/rwlock access to avoid panics on poisoned locks.

**AI Integration:** `OllamaClient` provides streaming LLM inference for Active Listening insights, Ask AI conversations, RAG queries, and the Suggestion Engine.

**Application Flow:**

1. App starts minimized to tray, loads settings, initializes managers
2. First-run downloads preferred model (Whisper/Parakeet/Moonshine)
3. Global shortcut triggers audio recording with VAD filtering
4. Audio sent to transcription engine for processing
5. Text pasted to active application via system clipboard
6. (Optional) Active Listening runs continuous transcription with Ollama-powered insights

### Single Instance Architecture

The app enforces single instance behavior -- launching when already running brings the settings window to front rather than creating a new process.

## Internationalization (i18n)

All user-facing strings must use i18next translations. ESLint enforces this via `eslint-plugin-i18next` (no hardcoded strings in JSX).

**Adding new text:**

1. Add key to `src/i18n/locales/en/translation.json`
2. Use in component: `const { t } = useTranslation(); t('key.path')`

**Supported locales (16):**

```
src/i18n/locales/
  ar/ cs/ de/ en/ es/ fr/ it/ ja/ ko/ pl/ pt/ ru/ tr/ uk/ vi/ zh/
```

## Code Style

**Rust:**

- Run `cargo fmt` and `cargo clippy` before committing
- Use `HandyError` over plain strings for error returns
- Use `SafeLock`/`SafeRwLock` instead of `.unwrap()` on mutexes
- Use descriptive names, add doc comments for public APIs

**TypeScript/React:**

- Strict TypeScript, avoid `any` types
- Functional components with hooks
- Zod schemas for runtime type validation
- `useCallback` for stable function references
- Container component pattern for layout
- Tailwind CSS for styling
- Path aliases: `@/` -> `./src/`
- Group imports: external libs, internal modules, relative imports

## Commit Guidelines

Use conventional commits:

- `feat:` new features
- `fix:` bug fixes
- `docs:` documentation
- `refactor:` code refactoring
- `test:` test additions/changes
- `chore:` maintenance

## Debug Mode

Access debug features: `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux)

## Platform Notes

- **macOS**: Metal acceleration, accessibility permissions required
- **Windows**: Vulkan acceleration, code signing
- **Linux**: OpenBLAS + Vulkan, Wayland support via wtype/dotool/ydotool/kwtype, gtk-layer-shell for overlay, SIGUSR2 signal for external shortcut control
