# Upstream PR Tracking

This document tracks the synchronization status between this fork and the upstream [cjpais/Handy](https://github.com/cjpais/Handy) repository.

## Sync Status

| Property | Value |
|----------|-------|
| **Last Synced** | 2026-03-01 |
| **Upstream Repo** | https://github.com/cjpais/Handy |
| **Last Upstream Commit** | `998449d2adad4729df2c47f920399c846698ced0` |
| **Upstream Version** | v0.7.9 |
| **Fork Branch** | `main` |

---

## PRs Ready to Merge

These PRs have all CI checks passing and are ready for integration:

| PR# | Title | CI Status | Notes |
|-----|-------|-----------|-------|
| [#807](https://github.com/cjpais/Handy/pull/807) | Add portable mode to NSIS installer | ✅ All pass | Windows installer enhancement |
| [#851](https://github.com/cjpais/Handy/pull/851) | Per-entry post-process button and version history viewer | ✅ All pass | UI enhancement for post-processing |
| [#814](https://github.com/cjpais/Handy/pull/814) | Store post-processing API keys in OS keychain | ✅ All pass | Security improvement |
| [#768](https://github.com/cjpais/Handy/pull/768) | Configurable custom audio feedback sounds | ✅ All pass | Audio customization |
| [#747](https://github.com/cjpais/Handy/pull/747) | Lazy stream close for bluetooth mic latency | ✅ All pass | Bluetooth audio fix |
| [#548](https://github.com/cjpais/Handy/pull/548) | Add Flatpak packaging support | ✅ All pass | Linux distribution |
| [#477](https://github.com/cjpais/Handy/pull/477) | Don't crash if there's no mic | ✅ All pass | Error handling |

---

## PRs In Progress

These PRs are actively being developed or have failing CI:

| PR# | Title | CI Status | Notes |
|-----|-------|-----------|-------|
| [#930](https://github.com/cjpais/Handy/pull/930) | Add transcription hook | Pending | Extensibility feature |
| [#928](https://github.com/cjpais/Handy/pull/928) | HuggingFace custom model downloads | Pending | Model management |
| [#927](https://github.com/cjpais/Handy/pull/927) | Improve Apple Intelligence detection | Pending | macOS feature |
| [#874](https://github.com/cjpais/Handy/pull/874) | Custom recordings directory management | Pending | Storage management |
| [#872](https://github.com/cjpais/Handy/pull/872) | Bump macOS minimum to 10.15 | Pending | Platform requirement |
| [#832](https://github.com/cjpais/Handy/pull/832) | Live transcription | ❌ Rust tests fail | **May conflict with Active Listening** |
| [#784](https://github.com/cjpais/Handy/pull/784) | Custom Storybook for design system | Pending | Development tooling |
| [#770](https://github.com/cjpais/Handy/pull/770) | Hidden OCR template context | Pending | Post-processing |
| [#734](https://github.com/cjpais/Handy/pull/734) | Whisper GPU fallback and compute mode | Pending | Performance |
| [#704](https://github.com/cjpais/Handy/pull/704) | More LLM post-processing variables | Pending | Post-processing |
| [#633](https://github.com/cjpais/Handy/pull/633) | LLM base URL env support | Pending | Configuration |
| [#559](https://github.com/cjpais/Handy/pull/559) | Follow OS Input Language | ❌ Rust tests fail | Localization |
| [#552](https://github.com/cjpais/Handy/pull/552) | Symmetric recording visualization bars | Pending | UI improvement |
| [#509](https://github.com/cjpais/Handy/pull/509) | OpenAI-style local API server | Pending | API feature |
| [#455](https://github.com/cjpais/Handy/pull/455) | Text replacements feature | Pending | Text processing |
| [#381](https://github.com/cjpais/Handy/pull/381) | Local file transcription (WAV, MP3, M4A) | Pending | File import |
| [#369](https://github.com/cjpais/Handy/pull/369) | Double-click tray icon support | ⚠️ Prettier fail | UX enhancement |

---

## PRs That May Conflict with Local Features

These upstream PRs may have feature overlap with our local implementations:

| PR# | Title | Local Overlap | Action Required |
|-----|-------|---------------|-----------------|
| [#832](https://github.com/cjpais/Handy/pull/832) | Live transcription | **Active Listening** | Compare implementations, may need to deprecate local version |
| [#618](https://github.com/cjpais/Handy/pull/618) | Wake-Word (draft) | **Active Listening** | Wake-word trigger could complement our Ollama-based listening |
| [#509](https://github.com/cjpais/Handy/pull/509) | Local API server | **Ask AI / RAG** | Could potentially share LLM infrastructure |

---

## Platform-Specific PRs

### Linux

| PR# | Title | Status | Notes |
|-----|-------|--------|-------|
| [#572](https://github.com/cjpais/Handy/pull/572) | Wayland GNOME system shortcuts | Pending | Input handling |
| [#548](https://github.com/cjpais/Handy/pull/548) | Flatpak packaging | ✅ Ready | Distribution |
| [#689](https://github.com/cjpais/Handy/pull/689) | Wayland remote desktop direct mode | Pending | Remote access |

### macOS

| PR# | Title | Status | Notes |
|-----|-------|--------|-------|
| [#927](https://github.com/cjpais/Handy/pull/927) | Apple Intelligence detection | Pending | AI integration |
| [#872](https://github.com/cjpais/Handy/pull/872) | Minimum version 10.15 | Pending | Compatibility |

### Windows

| PR# | Title | Status | Notes |
|-----|-------|--------|-------|
| [#807](https://github.com/cjpais/Handy/pull/807) | Portable mode installer | ✅ Ready | NSIS enhancement |

---

## Local Unique Features (Fork-Only)

These features are unique to this fork and not present in upstream:

### Active Listening (`src-tauri/src/managers/active_listening.rs`)
- **Description**: Continuous transcription with AI-generated insights via Ollama
- **Status**: ✅ Maintained
- **Lines**: ~1,367
- **Dependencies**: Ollama LLM, audio loopback/mixer
- **Frontend**: `src/components/settings/active-listening/`

### Ask AI (`src-tauri/src/managers/ask_ai.rs`)
- **Description**: Multi-turn voice conversations with local LLM
- **Status**: ✅ Maintained
- **Lines**: ~583
- **Dependencies**: Ollama LLM
- **Frontend**: `src/components/settings/ask-ai/`
- **History**: `src-tauri/src/managers/ask_ai_history.rs` (~545 lines)

### RAG Knowledge Base (`src-tauri/src/managers/rag.rs`)
- **Description**: Retrieval-Augmented Generation for contextual responses
- **Status**: ✅ Maintained
- **Lines**: ~590
- **Dependencies**: fuse.js (frontend fuzzy search)
- **Frontend**: `src/components/settings/knowledge-base/`

### Suggestion Engine (`src-tauri/src/managers/suggestion_engine.rs`)
- **Description**: Context-aware quick responses and suggestions
- **Status**: ✅ Maintained
- **Lines**: ~539
- **Dependencies**: RAG, Ollama LLM

### Audio Toolkit Extensions
- **Loopback Recording**: `src-tauri/src/audio_toolkit/audio/loopback.rs`
- **Audio Mixer**: `src-tauri/src/audio_toolkit/audio/mixer.rs`
- **Speaker Diarization**: `src-tauri/src/audio_toolkit/diarization/`

### Ollama Client (`src-tauri/src/ollama_client.rs`)
- **Description**: Streaming Ollama LLM client for all AI features
- **Status**: ✅ Maintained
- **Lines**: ~533

---

## Sync Procedure

### Regular Sync (Monthly)

```bash
# 1. Fetch upstream changes
git fetch upstream

# 2. Create sync branch
git checkout -b feature/upstream-sync-$(date +%Y%m%d)

# 3. Merge upstream
git merge upstream/main

# 4. Resolve conflicts (prioritize upstream for core, keep local for unique features)

# 5. Test build
make check && make test

# 6. Merge to main
git checkout main
git merge feature/upstream-sync-$(date +%Y%m%d)
```

### Conflict Resolution Guidelines

1. **Core Transcription**: Accept upstream changes
2. **Settings Structure**: Merge carefully, preserve local settings modules
3. **UI Components**: Accept upstream, re-add local feature UIs
4. **lib.rs**: Re-add local manager initializations after merge
5. **Dependencies**: Prefer upstream versions, keep local-only deps

---

## Contributing Back to Upstream

Features that could be contributed back:

| Feature | Effort | Value | Blocker |
|---------|--------|-------|---------|
| Ask AI conversation UI | Medium | High | Requires Ollama dependency discussion |
| RAG knowledge base | High | High | Large PR, needs community input |
| Suggestion Engine | Medium | Medium | UX design alignment needed |

---

## Changelog

### 2026-03-01
- Initial sync with upstream v0.7.9
- Reverted Dictum branding to Handy
- Preserved all local features (Active Listening, Ask AI, RAG, Suggestions)
- Added 27 upstream PRs to tracking

---

*Last updated: 2026-03-01*
