# Handy

[![Discord](https://img.shields.io/badge/Discord-%235865F2.svg?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/WVBeWsNXK4)

**A free, open source, and extensible speech-to-text application that works completely offline.**

Handy is a cross-platform desktop application that provides simple, privacy-focused speech transcription. Press a shortcut, speak, and have your words appear in any text field. This happens on your own computer without sending any information to the cloud.

> This fork adds AI-powered features: **Active Listening**, **Ask AI**, **RAG Knowledge Base**, and **Suggestion Engine** — all running locally via [Ollama](https://ollama.ai).

---

## Features

### Core Features

- **Speech-to-Text** — Local transcription using Whisper, Parakeet, Moonshine, or SenseVoice
- **Push-to-Talk & Auto Mode** — Hold shortcut or auto-detect speech end
- **Voice Activity Detection** — Silero VAD filters silence automatically
- **Post-Processing** — Optional LLM cleanup via OpenAI, Anthropic, or Ollama
- **History** — Searchable transcription history
- **Multi-Platform** — macOS, Windows, Linux

### Fork-Exclusive Features

| Feature | Description | Requires |
|---------|-------------|----------|
| **Active Listening** | Continuous transcription with AI-generated insights | Ollama |
| **Ask AI** | Multi-turn voice conversations with local LLM | Ollama |
| **RAG Knowledge Base** | Vector search for context-aware responses | Ollama |
| **Suggestion Engine** | Context-aware quick responses | Ollama |
| **System Audio Loopback** | Capture both sides of calls/meetings | - |
| **Audio Mixer** | Mix microphone + system audio | - |

See [FEATURES.md](FEATURES.md) for detailed documentation.

---

## Quick Start

### Installation

1. Download the latest release from the [releases page](https://github.com/cjpais/Handy/releases) or the [website](https://handy.computer)
   - **macOS**: Also available via [Homebrew cask](https://formulae.brew.sh/cask/handy): `brew install --cask handy`
   - **Windows**: Also available via [winget](https://github.com/microsoft/winget-pkgs): `winget install cjpais.Handy` \
     **Note:** The Homebrew cask and winget package are not maintained by the Handy developers.
2. Install the application
3. Launch Handy and grant necessary system permissions (microphone, accessibility)
4. Configure your preferred keyboard shortcuts in Settings
5. Start transcribing!

### Development Setup

For detailed build instructions including platform-specific requirements, see [BUILD.md](BUILD.md).

## Integrations

<a href="https://www.raycast.com/mattiacolombomc/handy" title="Install Handy Raycast Extension"><img src="https://www.raycast.com/mattiacolombomc/handy/install_button@2x.png?v=1.1" height="64" style="height: 64px;" alt="Install handy Raycast Extension" /></a>

Control Handy from [Raycast](https://www.raycast.com) — start/stop recording, browse transcript history, manage dictionary, switch models and languages.

[Source](https://github.com/mattiacolombomc/raycast-handy) · by [@mattiacolombomc](https://github.com/mattiacolombomc)

## Architecture

Handy is built as a Tauri application combining:

- **Frontend**: React + TypeScript with Tailwind CSS for the settings UI
- **Backend**: Rust for system integration, audio processing, and ML inference
- **Core Libraries**:
  - `whisper-rs`: Local speech recognition with Whisper models
  - `transcription-rs`: CPU-optimized speech recognition with Parakeet models
  - `cpal`: Cross-platform audio I/O
  - `vad-rs`: Voice Activity Detection
  - `rdev`: Global keyboard shortcuts and system events
  - `rubato`: Audio resampling

### Debug Mode

Handy includes an advanced debug mode for development and troubleshooting. Access it by pressing:

- **macOS**: `Cmd+Shift+D`
- **Windows/Linux**: `Ctrl+Shift+D`

### CLI Parameters

Handy supports command-line flags for controlling a running instance and customizing startup behavior. These work on all platforms (macOS, Windows, Linux).

**Remote control flags** (sent to an already-running instance via the single-instance plugin):

```bash
# Install Ollama
brew install ollama          # macOS
# or curl -fsSL https://ollama.com/install.sh | sh  # Linux

# Start Ollama server
ollama serve

# Pull a model
ollama pull llama3.2
```

Then enable features in Settings > Active Listening / Ask AI / Knowledge Base.

---

## How It Works

```
Press Shortcut → Speak → VAD Filters Silence → Local Transcription → Paste to App
```

**Supported Models:**

| Model | Size | Speed | Accuracy |
|-------|------|-------|----------|
| Whisper Tiny | 75MB | Fastest | Good |
| Whisper Base | 142MB | Fast | Better |
| Whisper Small | 466MB | Medium | Good |
| Whisper Medium | 1.5GB | Slow | Great |
| Whisper Large | 2.9GB | Slowest | Best |
| Parakeet V3 | 478MB | Fast | Good (English) |

---

## Documentation

| Document | Description |
|----------|-------------|
| [FEATURES.md](FEATURES.md) | Feature documentation and usage |
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design and data flow |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |
| [CLAUDE.md](CLAUDE.md) | Developer reference and patterns |
| [DEV_DOCS.md](DEV_DOCS.md) | Technical reference |
| [BUILD.md](BUILD.md) | Build instructions |
| [UPSTREAM_TRACKING.md](UPSTREAM_TRACKING.md) | Fork sync status |

---

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| **macOS** (Intel & Apple Silicon) | Full | Metal GPU acceleration |
| **Windows** (x64) | Full | Vulkan/DirectML acceleration |
| **Linux** (x64) | Full | Wayland requires wtype/dotool |

### Linux Notes

**Text Input Tools:**

| Display Server | Tool | Install |
|----------------|------|---------|
| X11 | xdotool | `sudo apt install xdotool` |
| Wayland | wtype | `sudo apt install wtype` |
| Both | dotool | `sudo apt install dotool` |

**Wayland Shortcuts:**

Use CLI flags with your window manager:

```bash
# GNOME: Settings > Keyboard > Custom Shortcuts
handy --toggle-transcription

# Sway/i3
bindsym $mod+o exec handy --toggle-transcription

# Or via Unix signals
bindsym $mod+o exec pkill -USR2 -n handy
```

---

## CLI Parameters

```bash
# Remote control (sent to running instance)
handy --toggle-transcription    # Toggle recording
handy --toggle-post-process     # Toggle with post-processing
handy --cancel                  # Cancel current operation

# Startup flags
handy --start-hidden            # Start without window
handy --no-tray                 # No system tray icon
handy --debug                   # Enable debug logging
```

---

## Debug Mode

Access advanced diagnostics: `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux)

---

## Troubleshooting

### Manual Model Installation

If behind a proxy, download models manually:

**Whisper Models:**
- Small: `https://blob.handy.computer/ggml-small.bin`
- Medium: `https://blob.handy.computer/whisper-medium-q4_1.bin`
- Turbo: `https://blob.handy.computer/ggml-large-v3-turbo.bin`
- Large: `https://blob.handy.computer/ggml-large-v3-q5_0.bin`

**Parakeet Models:**
- V2: `https://blob.handy.computer/parakeet-v2-int8.tar.gz`
- V3: `https://blob.handy.computer/parakeet-v3-int8.tar.gz`

Place in `~/.config/handy/models/` (Linux), `~/Library/Application Support/com.pais.handy/models/` (macOS), or `%APPDATA%\com.pais.handy\models\` (Windows).

### Common Issues

**"Ollama not connected":**
```bash
ollama serve  # Start Ollama server
curl http://localhost:11434/api/tags  # Verify
```

**Linux crashes:**
```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 handy
```

**Missing library (libgtk-layer-shell.so.0):**
```bash
sudo apt install libgtk-layer-shell0  # Ubuntu/Debian
sudo dnf install gtk-layer-shell      # Fedora
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Code patterns (Manager, Command, Settings, Components)
- Testing guidelines
- PR process

**Quick Start:**
```bash
git clone git@github.com:YOUR_USERNAME/Handy.git
cd Handy
make install
make dev
```

---

## Upstream Sync

This fork tracks [cjpais/Handy](https://github.com/cjpais/Handy). See [UPSTREAM_TRACKING.md](UPSTREAM_TRACKING.md) for:
- Open PRs and issues
- Sync status
- Conflict resolution guidelines

---

## Sponsors

<div align="center">
  <a href="https://wordcab.com">
    <img src="sponsor-images/wordcab.png" alt="Wordcab" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://github.com/epicenter-so/epicenter">
    <img src="sponsor-images/epicenter.png" alt="Epicenter" width="120" height="120">
  </a>
  &nbsp;&nbsp;&nbsp;&nbsp;
  <a href="https://boltai.com?utm_source=handy">
    <img src="sponsor-images/boltai.jpg" alt="Bolt AI" width="120" height="120">
  </a>
</div>

---

## Related Projects

- [Handy CLI](https://github.com/cjpais/handy-cli) — Original Python CLI version
- [handy.computer](https://handy.computer) — Project website

---

## License

MIT License — see [LICENSE](LICENSE)

---

## Acknowledgments

- **Whisper** by OpenAI
- **whisper.cpp and ggml** for cross-platform inference
- **Silero** for VAD
- **Tauri** for the app framework
- **Ollama** for local LLM inference
- **Community contributors**

---

_"Your search for the right speech-to-text tool can end here—not because Handy is perfect, but because you can make it perfect for you."_
