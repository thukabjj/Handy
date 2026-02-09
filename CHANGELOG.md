# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/).

## [0.7.2] - 2026-02-08

### Added

- Models settings page with filtering and management UI (#478)
- Playwright E2E testing infrastructure (#673)
- Layer-shell Wayland overlay support via gtk-layer-shell (#680)
- KDE Plasma 6 Wayland support via kwtype (#676)
- N-gram matching for multi-word custom word correction (#711)
- Korean translation (#721)
- RTL support and language direction handling (#698)
- Homebrew cask install instructions (#705)
- Optional hotkey for post-processing request (#355)

### Fixed

- Rust tests after gtk-layer-shell introduction (#719)
- Missing Nix flake dependencies (#708)
- Missing Linux build dependencies in BUILD.md (#728)

## [0.7.1] - 2026-02-01

### Added

- Arabic localization support (#693)
- Paste delay setting (#694)
- Nix flake for NixOS support (#561)
- ARM64 Linux builds (#629)
- Rust tests in CI (#671)

### Fixed

- First launch on macOS (#690)
- History functionality (#691)
- Post-processing for Chinese text (#628)
- Zero-width character issue (#627)

## [0.7.0] - 2026-01-20

### Added

- Experimental features UI with reorganized settings (#620)
- Automatic filler word removal from transcriptions (#589)
- Copy last transcript action in tray menu (#598)
- Turkish language support (#582)

### Changed

- Refactored state management to use Immer for immutable updates (#527)

### Fixed

- Audio playback in History tab on Linux (#617)

## [0.6.11] - 2026-01-14

### Added

- Moonshine Base speech recognition model (#556)
- Czech translation (#568)
- Portuguese language support (#540)
- Ukrainian language support (#536)
- ydotool support for Linux text input (#557)
- Permission step to onboarding flow (#545)
- Reusable Tooltip component for settings UI (#538)

### Fixed

- Keybinding changes failing silently due to incorrect key ordering (#524)
- Apple Intelligence UI improvements and reusable alert component (#517)
- Race condition when toggling transcription via SIGUSR2 (#560)
- Text selection cursor appearing on UI elements (#541)

## [0.6.10] - 2026-01-04

### Added

- Russian language (#474)

### Fixed

- Crash on macOS 26.x beta during startup (#473)
- Direct typing mode (#493)
- Model unload on cancel when immediate unload enabled (#498)
- Replaced async-openai library with direct HTTP requests (#480)

## [0.6.9] - 2025-12-23

### Added

- Italian translation (#459)
- Polish translation (#453)
- Tray menu localization using system locale (#446)
- Dev indicator in tray menu version for debug builds (#470)
- Overlay transcription display and stored language settings (#465)

### Fixed

- Possible deadlocks when pressing Escape to cancel recordings (#408)
- History limit not saving to settings (#463)

## [0.6.8] - 2025-12-12

### Added

- German, Japanese, and Chinese translations (#444)

### Fixed

- Model auto-switch interrupting active transcription (#443)
- Build configuration (#447)

## [0.6.7] - 2025-12-12

### Added

- Internationalization (i18n) support with language selection (#437)

### Changed

- Refactored enigo input handling (#441)
- Updated CI to macOS 26 (#440)

## [0.6.6] - 2025-12-11

### Added

- Ctrl+Shift+V paste option on Windows and Linux (#430)
- Apple Intelligence integration for Intel Macs (#391)

### Changed

- Refactored history storage to SQLite (#412)
- Set WEBKIT_DISABLE_DMABUF_RENDERER=1 in environment by default (#427)

## [0.6.5] - 2025-12-08

### Added

- Option to append trailing space after pasted text (#405)

### Changed

- Prioritized F32 sample format for better audio quality (#393)

## [0.6.4] - 2025-11-29

### Fixed

- Disabled cancel shortcut on Linux to prevent issues (#392)

## [0.6.3] - 2025-11-28

### Added

- Cancel hotkey and configuration (#224)
- Wayland paste support using wtype or dotool (#376)
- tauri-specta for auto-generated TypeScript bindings (#322)
- Disabled option for pasting (#364)
- Async test sound playback (#375)
- Recording overlay in macOS fullscreen apps and when switching spaces (#361)

### Changed

- Switch from enable to disable update checking (#362)

### Fixed

- Re-assert always-on-top for overlay on Windows (#383)

## [0.6.2] - 2025-11-19

### Added

- Conversion between Simplified and Traditional Chinese (#356)
- SIGUSR2 handling for recording toggles (#354)

### Changed

- Improved audio feedback timing using real sound duration and trimmed WAV files (#349)

### Fixed

- Laptop detection logic made more reliable (#348)

## [0.6.1] - 2025-11-17

### Added

- File-based debug logging (#347)
- More options for deleting recordings automatically with folder access button (#330)
- Fallback microphone for clamshell/desktop mode (#329)
- Windows ARM64 builds in CI (#340)
- Post-processing rule to keep original language (#316)

### Fixed

- Unstable mute implementation on Windows/Linux (#341)

## [0.6.0] - 2025-11-05

### Added

- **LLM-based post-processing** for transcript enhancement (#222)

## [0.5.5] - 2025-11-03

### Added

- Parakeet V2 support (#116)
- Mute while recording setting (#257)
- SIGUSR2 handling for external recording toggles (#281)

### Changed

- Windows paste uses Shift+Insert instead of Ctrl+V (#236)
- Moved paste method and clipboard handling to advanced settings
- Moved app data directory to about section

### Fixed

- Missing defaults option in plugin-store load() call (#288)
- Tooltip overflow (#269)
- Disabled Translate to English setting for Whisper Turbo model (#259)
- Overlay UI flickering (#231)

## [0.5.4] - 2025-10-17

### Added

- Custom recording sounds with volume control (#214)
- Copy to clipboard setting (#220)
- Display download size in model selector (#221)

### Changed

- Startup optimizations (#182)

## [0.5.3] - 2025-10-13

### Added

- Autostart toggle in advanced settings (#177)
- Display overlay on active monitor (#175)
- History limit setting (#150)
- Paste method selection (#187)
- Source code link in about section

### Changed

- Default to no overlay on Linux
- Default to ALSA on Linux (#190)
- Parallel model loading (#181)
- Moved always-on-mic to debug settings

### Fixed

- Whisper crashing on Linux (#212)
- Translate-to-English functionality (#173)
- Modifier key release delay on paste (#165)

## [0.5.1] - 2025-09-28

### Added

- Delete individual audio entries from history (#119)

### Fixed

- Wayland clipboard failure
- Dropped libsamplerate helper feature (#144)

## [0.5.0] - 2025-09-09

### Added

- **Parakeet V3** model with automatic language detection (#111)

### Changed

- Updated onboarding flow (#112)
- Tweaked Whisper sampling strategy to reduce hallucination (#109)

## [0.4.2] - 2025-09-06

### Added

- **Start hidden** option (#105)

## [0.4.1] - 2025-09-05

### Fixed

- Linux history bug

## [0.4.0] - 2025-09-05

### Added

- **UI v2** complete redesign (#103)
- **Transcription history** with browsing and review (#104)

## [0.3.9] - 2025-09-03

### Added

- VRAM inactivity timer for automatic model unloading (#101)
- Pink tray icon option (#100)
- Sponsor section

## [0.3.8] - 2025-08-19

### Changed

- Downgraded whisper-rs to 0.13.2 for older macOS support (#88)
- Unified CI workflow
- Separate Ubuntu 22.04 and 24.04 builds
- Updated default correction threshold to 0.15

### Fixed

- AppImage libwayland stripping (#90)
- Various CI and build issues

## [0.3.7] - 2025-08-11

### Added

- **Custom word correction** with phonetic matching (#75)
- Shared Input/Button UI components

### Fixed

- Settings: default device labels and empty device lists
- Blank settings window when dev server not running
- UI: reduced window height and tightened layout

## [0.3.6] - 2025-08-07

### Changed

- Overlay position selector replaces show/hide toggle
- Updated recording overlay design with pink palette
- Refined overlay positioning (separate top/bottom offsets)

## [0.3.5] - 2025-08-02

### Added

- Partial/resumable model downloads (#70)
- Cross-platform keybinding handling improvements (#71)
- Invalid binding validation in frontend

## [0.3.4] - 2025-08-02

### Added

- Theme-aware tray icons on Windows (#69)
- Single instance enforcement (#68)

### Changed

- Refined settings window dimensions (#62)

## [0.3.3] - 2025-08-01

### Added

- **Recording overlay** with audio visualizer (#59)
- Cancel recording from overlay (#60)
- Debug mode toggle (#61)
- Setting to show/hide overlay
- Cancel recording/transcribing actions in tray menu (#49)

### Fixed

- Cross-layout paste using virtual key codes (#50)
- Shortcut stuck pressed causing missed transcriptions

## [0.3.2] - 2025-07-28

### Added

- Language selection in settings (#54)

### Fixed

- Tray icons marked as template for macOS dark mode (#51)
- Check for updates window focus (#53)

## [0.3.1] - 2025-07-26

### Added

- Transcribing state and updated tray icons (#44)
- CHANGELOG.md
- GitHub Sponsors (FUNDING.yml)

### Fixed

- Vulkan support
- Ubuntu 24.04 compatibility
- u8 sample format support
- Fedora display issues

## [0.3.0] - 2025-07-11

### Added

- **Translate to English** setting: automatic translation of speech to English
- Settings refactored into React hooks for better state management
- Audio device switching capability
- Hysteresis to VAD (Voice Activity Detection) for more stable recording

### Changed

- Major audio backend refactor for improved performance and reliability
- Moved audio toolkit into src-tauri directory for better permissions handling
- Model files no longer need to be downloaded separately for releases
- Updated settings components and transcription logic

### Fixed

- Audio toolkit permissions issues
- Various stability improvements

## [0.2.3] - 2025-07-03

### Fixed

- Keycode bug that was causing input issues
- Whisper model optimization: switched to unquantized Whisper Turbo, updated Whisper Medium quantization to 4_1

## [0.2.2] - 2025-07-02

### Fixed

- Removed 50ms delay feature flag for Windows (now applies to all platforms for consistency)

## [0.2.1] - 2025-07-01

### Added

- Ctrl+Space key binding for Windows platform

### Fixed

- Windows crash issue
- Model loading on startup when available
- Windows paste functionality bug

## [0.2.0] - 2025-06-30

### Added

- **Microphone activation on demand**: More efficient resource usage
- Less permissive VAD settings for better accuracy

### Changed

- Improved microphone management and activation system

## [0.1.6] - 2025-06-30

### Added

- **Multiple models support**: Users can now select from different transcription models
- Model selection onboarding flow
- Cleanup and refactoring of model management

### Changed

- Enhanced user experience with model selection interface
- Better language and UI tweaks

## [0.1.5] - 2025-06-27

### Added

- **Different start and stop recording sounds**: Enhanced audio feedback
- Recording sound samples for better user experience

## [0.1.4] - 2025-06-27

### Fixed

- Build issues
- Auto-update functionality improvements

## [0.1.3] - 2025-06-26

### Fixed

- Paste functionality using enigo library for better cross-platform compatibility

## [0.1.2] - 2025-06-26

### Added

- **Auto-update functionality**: Application can now automatically update itself
- Footer displaying current version
- Improved menu system

### Changed

- Better user interface for version management
- Enhanced update workflow

## [0.1.1] - 2025-06-25

### Added

- **Comprehensive build system**: Support for Windows, macOS, and Linux
- Windows code signing for trusted installation
- Ubuntu/Linux build support with Vulkan
- Model file download and packaging for releases
- GitHub Actions CI/CD workflow

### Changed

- Improved build process and release workflow
- Better cross-platform compatibility

### Fixed

- Various build-related issues across platforms

## [0.1.0] - 2025-05-16

### Added

- **Initial release** of Handy
- Basic speech-to-text transcription functionality
- Voice Activity Detection (VAD) for automatic recording
- Cross-platform support (macOS, Windows, Linux)
- **Tauri-based desktop application** with React frontend
- **Global keyboard shortcuts** for activation
- **Clipboard integration** for automatic text insertion
- **LLM integration** for enhanced transcription processing
- **Configurable settings** including:
  - Custom key bindings
  - Audio device selection
  - Microphone settings
  - Push-to-talk functionality
- **System tray integration** with recording indicators
- **Accessibility permissions** handling for macOS
- **Settings persistence** with unified settings store
- **Background operation** capability
- **Multiple audio format support** with on-the-fly resampling
- **Whisper model integration** for high-quality transcription
- **MIT License** for open-source distribution
