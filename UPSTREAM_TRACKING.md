# Upstream Tracking

This document tracks synchronization between this fork and [cjpais/Handy](https://github.com/cjpais/Handy).

---

## Sync Status

| Property                 | Value                                      |
| ------------------------ | ------------------------------------------ |
| **Last Synced**          | 2026-03-01                                 |
| **Upstream Repo**        | https://github.com/cjpais/Handy            |
| **Last Upstream Commit** | `998449d2adad4729df2c47f920399c846698ced0` |
| **Upstream Version**     | v0.7.9                                     |
| **Fork Branch**          | `main`                                     |

---

## Open PRs by Category

### Ready to Merge

- [#807](https://github.com/cjpais/Handy/pull/807) Portable mode NSIS installer
- [#851](https://github.com/cjpais/Handy/pull/851) Per-entry post-process button
- [#814](https://github.com/cjpais/Handy/pull/814) Store API keys in OS keychain
- [#768](https://github.com/cjpais/Handy/pull/768) Custom audio feedback sounds
- [#747](https://github.com/cjpais/Handy/pull/747) Lazy stream close for Bluetooth
- [#548](https://github.com/cjpais/Handy/pull/548) Flatpak packaging
- [#477](https://github.com/cjpais/Handy/pull/477) Don't crash if no mic

### Features

- [#930](https://github.com/cjpais/Handy/pull/930) Transcription hook
- [#928](https://github.com/cjpais/Handy/pull/928) HuggingFace custom models
- [#927](https://github.com/cjpais/Handy/pull/927) Apple Intelligence detection
- [#874](https://github.com/cjpais/Handy/pull/874) Custom recordings directory
- [#832](https://github.com/cjpais/Handy/pull/832) Live transcription **(may conflict with Active Listening)**
- [#784](https://github.com/cjpais/Handy/pull/784) Custom Storybook
- [#770](https://github.com/cjpais/Handy/pull/770) Hidden OCR template
- [#734](https://github.com/cjpais/Handy/pull/734) Whisper GPU fallback
- [#704](https://github.com/cjpais/Handy/pull/704) More LLM variables
- [#633](https://github.com/cjpais/Handy/pull/633) LLM base URL env support
- [#618](https://github.com/cjpais/Handy/pull/618) Wake-word detection (draft) **(may complement Active Listening)**
- [#559](https://github.com/cjpais/Handy/pull/559) Follow OS input language
- [#552](https://github.com/cjpais/Handy/pull/552) Symmetric visualization bars
- [#509](https://github.com/cjpais/Handy/pull/509) Local API server **(may share infrastructure with Ask AI)**
- [#455](https://github.com/cjpais/Handy/pull/455) Text replacements
- [#381](https://github.com/cjpais/Handy/pull/381) Local file transcription
- [#369](https://github.com/cjpais/Handy/pull/369) Double-click tray icon

### Platform Support

- [#872](https://github.com/cjpais/Handy/pull/872) macOS minimum 10.15
- [#572](https://github.com/cjpais/Handy/pull/572) Wayland GNOME shortcuts
- [#689](https://github.com/cjpais/Handy/pull/689) Wayland remote desktop

---

## Open Issues by Severity

### Critical (Crashes)

- [#924](https://github.com/cjpais/Handy/issues/924) Kubuntu crash
- [#880](https://github.com/cjpais/Handy/issues/880) Ubuntu 24.04 crash
- [#831](https://github.com/cjpais/Handy/issues/831) Segfault on Mint
- [#806](https://github.com/cjpais/Handy/issues/806) Pop!\_OS mic issue
- [#867](https://github.com/cjpais/Handy/issues/867) CUDA out of memory

### Input/Keyboard

- [#917](https://github.com/cjpais/Handy/issues/917) Super key Windows
- [#912](https://github.com/cjpais/Handy/issues/912) Numpad hotkeys
- [#906](https://github.com/cjpais/Handy/issues/906) Korean shortcuts
- [#865](https://github.com/cjpais/Handy/issues/865) Key4 binding issue
- [#714](https://github.com/cjpais/Handy/issues/714) Hyper key support
- [#705](https://github.com/cjpais/Handy/issues/705) Command key macOS

### Audio Issues

- [#907](https://github.com/cjpais/Handy/issues/907) Audio routing macOS
- [#903](https://github.com/cjpais/Handy/issues/903) Mic not detected
- [#897](https://github.com/cjpais/Handy/issues/897) External mic switch
- [#793](https://github.com/cjpais/Handy/issues/793) Bluetooth device issues
- [#745](https://github.com/cjpais/Handy/issues/745) Stereo input issues

### UI/UX

- [#920](https://github.com/cjpais/Handy/issues/920) Linux tray overlap
- [#911](https://github.com/cjpais/Handy/issues/911) Window focus loss
- [#905](https://github.com/cjpais/Handy/issues/905) Overlay position reset
- [#901](https://github.com/cjpais/Handy/issues/901) Settings search broken
- [#879](https://github.com/cjpais/Handy/issues/879) Dark mode inconsistent
- [#864](https://github.com/cjpais/Handy/issues/864) Tray icon visibility

### Feature Requests

- [#926](https://github.com/cjpais/Handy/issues/926) Real-time transcription
- [#923](https://github.com/cjpais/Handy/issues/923) Custom model paths
- [#919](https://github.com/cjpais/Handy/issues/919) Export history formats
- [#909](https://github.com/cjpais/Handy/issues/909) Multi-language support
- [#866](https://github.com/cjpais/Handy/issues/866) Speaker diarization
- [#846](https://github.com/cjpais/Handy/issues/846) Timestamp output

---

## Fork-Exclusive Features

These features exist only in this fork:

| Feature                 | Backend                                   | Frontend                     | Lines  |
| ----------------------- | ----------------------------------------- | ---------------------------- | ------ |
| **Active Listening**    | `managers/active_listening.rs`            | `settings/active-listening/` | ~1,367 |
| **Ask AI**              | `managers/ask_ai.rs`, `ask_ai_history.rs` | `settings/ask-ai/`           | ~1,128 |
| **RAG Knowledge Base**  | `managers/rag.rs`                         | `settings/knowledge-base/`   | ~590   |
| **Suggestion Engine**   | `managers/suggestion_engine.rs`           | `settings/suggestions/`      | ~539   |
| **Ollama Client**       | `ollama_client.rs`                        | -                            | ~533   |
| **Audio Loopback**      | `audio_toolkit/audio/loopback.rs`         | -                            | ~200   |
| **Audio Mixer**         | `audio_toolkit/audio/mixer.rs`            | -                            | ~150   |
| **Speaker Diarization** | `audio_toolkit/diarization/`              | -                            | ~300   |

---

## Potential Conflicts

These upstream PRs may conflict with fork features:

| PR                                               | Feature            | Conflict         | Resolution                                   |
| ------------------------------------------------ | ------------------ | ---------------- | -------------------------------------------- |
| [#832](https://github.com/cjpais/Handy/pull/832) | Live transcription | Active Listening | Compare implementations, may deprecate local |
| [#618](https://github.com/cjpais/Handy/pull/618) | Wake-word          | Active Listening | Could complement Ollama-based listening      |
| [#509](https://github.com/cjpais/Handy/pull/509) | Local API server   | Ask AI / RAG     | Could share LLM infrastructure               |

---

## Sync Procedure

```bash
# 1. Fetch upstream
git fetch upstream

# 2. Create sync branch
git checkout -b feature/upstream-sync-$(date +%Y%m%d)

# 3. Merge upstream
git merge upstream/main

# 4. Resolve conflicts
# Priority: upstream for core, keep local for fork features

# 5. Test
make check && make test

# 6. Merge to main
git checkout main
git merge feature/upstream-sync-$(date +%Y%m%d)
git push origin main
```

### Conflict Resolution

| Area                                                               | Priority                        |
| ------------------------------------------------------------------ | ------------------------------- |
| Core transcription (`audio_toolkit/`, `managers/transcription.rs`) | Upstream                        |
| Settings structure (`settings/mod.rs`)                             | Merge carefully                 |
| UI components (`src/components/`)                                  | Upstream, re-add local features |
| `lib.rs` manager init                                              | Re-add local managers           |
| Dependencies (`Cargo.toml`, `package.json`)                        | Upstream, keep local-only       |

---

## Changelog

### 2026-03-01

- Initial sync with upstream v0.7.9
- Reverted Dictum branding to Handy
- Preserved fork features (Active Listening, Ask AI, RAG, Suggestions)
- Added upstream PR/issue tracking (25 PRs, 30 issues)
- Comprehensive documentation rewrite

---

_Last updated: 2026-03-01_
