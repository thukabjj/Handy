# Features

This document describes all features available in Handy, including both upstream features and fork-exclusive enhancements.

---

## Table of Contents

- [Core Features](#core-features)
  - [Speech-to-Text](#speech-to-text)
  - [Model Selection](#model-selection)
  - [Post-Processing](#post-processing)
  - [Recording Modes](#recording-modes)
  - [History Management](#history-management)
- [Fork-Exclusive Features](#fork-exclusive-features)
  - [Active Listening](#active-listening)
  - [Ask AI](#ask-ai)
  - [RAG Knowledge Base](#rag-knowledge-base)
  - [Suggestion Engine](#suggestion-engine)
- [Advanced Features](#advanced-features)
  - [Custom Words](#custom-words)
  - [Audio Device Selection](#audio-device-selection)
  - [Debug Mode](#debug-mode)
- [Ollama Setup](#ollama-setup)

---

## Core Features

### Speech-to-Text

**Description:** Convert spoken audio to text using local AI models.

**How It Works:**

1. Press global shortcut (or use push-to-talk)
2. Speak into your microphone
3. Voice Activity Detection (VAD) filters silence
4. Local model transcribes speech to text
5. Text is copied to clipboard and/or pasted into active application

**Supported Models:**

| Model              | Size   | Speed   | Accuracy | Languages |
| ------------------ | ------ | ------- | -------- | --------- |
| **Whisper Tiny**   | ~75MB  | Fastest | Good     | 99        |
| **Whisper Base**   | ~142MB | Fast    | Better   | 99        |
| **Whisper Small**  | ~466MB | Medium  | Good     | 99        |
| **Whisper Medium** | ~1.5GB | Slow    | Great    | 99        |
| **Whisper Large**  | ~2.9GB | Slowest | Best     | 99        |
| **Parakeet**       | ~120MB | Fast    | Good     | English   |
| **Moonshine**      | ~600MB | Medium  | Good     | English   |
| **SenseVoice**     | ~1GB   | Medium  | Good     | Multi     |

**Settings:**

| Setting       | Description                      | Default      |
| ------------- | -------------------------------- | ------------ |
| Model         | Which transcription model to use | Whisper Base |
| Language      | Auto-detect or specify language  | Auto         |
| VAD Threshold | Silence detection sensitivity    | 0.5          |
| Output Mode   | Clipboard, paste, or both        | Both         |

---

### Model Selection

**Description:** Download and manage local transcription models.

**Features:**

- One-click model downloads
- Download progress tracking
- Model size and speed indicators
- Switch between models instantly
- Delete unused models to save space

**Location:** Settings > Models

**Model Storage:** `~/.config/handy/models/`

---

### Post-Processing

**Description:** Enhance transcription quality using LLM post-processing.

**Supported Providers:**

| Provider      | Local | API Key Required |
| ------------- | ----- | ---------------- |
| **Ollama**    | Yes   | No               |
| **OpenAI**    | No    | Yes              |
| **Anthropic** | No    | Yes              |

**Processing Options:**

- Grammar and spelling correction
- Punctuation improvement
- Formatting (markdown, lists)
- Custom prompts for specific use cases

**Configuration:**

```
Settings > Post-Processing
├── Enable Post-Processing
├── Provider Selection
├── API Key (for cloud providers)
├── Model Selection
└── Custom Prompt (optional)
```

---

### Recording Modes

**Description:** Multiple ways to trigger recording.

**Push-to-Talk Mode:**

- Hold shortcut key to record
- Release to stop and transcribe
- Best for: Quick voice notes, commands

**Auto Mode (Press to Start):**

- Press shortcut to start recording
- Voice Activity Detection automatically stops when you're done speaking
- Best for: Longer dictation, hands-free use

**Global Shortcut:**

- Default: `Cmd+Shift+Space` (macOS), `Ctrl+Shift+Space` (Windows/Linux)
- Fully customizable
- Works in any application

---

### History Management

**Description:** View and manage past transcriptions.

**Features:**

- Searchable transcription history
- Timestamp and duration tracking
- Copy previous transcriptions
- Delete individual entries or clear all
- Export functionality

**Location:** Settings > History

**Storage:** SQLite database at `~/.config/handy/history.db`

---

## Fork-Exclusive Features

These features are unique to this fork and require [Ollama](https://ollama.ai) for local LLM inference.

### Active Listening

**Description:** Continuous background transcription with AI-generated insights.

**Use Cases:**

- Meeting notes and summaries
- Interview transcription
- Lecture capture
- Brainstorming sessions

**How It Works:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Active Listening Session                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Audio Input (Mic + System)                                     │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ Continuous VAD  │ → Speech detected → Transcribe            │
│  └─────────────────┘                                           │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ Rolling Buffer  │ ← Recent transcripts accumulated         │
│  └─────────────────┘                                           │
│           │                                                     │
│           ▼ (On trigger or interval)                           │
│  ┌─────────────────┐                                           │
│  │   Ollama LLM    │ → Analyze context → Generate insight      │
│  └─────────────────┘                                           │
│           │                                                     │
│           ▼                                                     │
│  ┌─────────────────┐                                           │
│  │ Insight Display │ → Notify user → Store in history         │
│  └─────────────────┘                                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**

- Microphone input capture
- System audio loopback (hear both sides of calls)
- Audio mixing for combined capture
- Configurable insight generation triggers
- Rolling transcript buffer
- Session management (start/stop/pause)
- Insight history

**Settings:**

| Setting              | Description                | Default    |
| -------------------- | -------------------------- | ---------- |
| Ollama Model         | LLM for insights           | llama3.2   |
| Insight Interval     | Time between insights      | 5 minutes  |
| Buffer Size          | Transcript buffer length   | 10 minutes |
| Include System Audio | Capture system sounds      | On         |
| Insight Prompt       | Custom prompt for analysis | Default    |

**Location:** Settings > Active Listening

**Backend:** `src-tauri/src/managers/active_listening.rs`

**Frontend:** `src/components/settings/active-listening/`

---

### Ask AI

**Description:** Multi-turn voice conversations with local LLM.

**Use Cases:**

- Hands-free AI assistant
- Voice-controlled coding help
- Quick questions while working
- Accessibility assistance

**How It Works:**

```
User speaks → Transcription → Ollama LLM → Response displayed
      │                              │
      │                              └──► (Optional) RAG context
      │
      └──► (Optional) Text-to-Speech response
```

**Features:**

- Natural conversation flow
- Conversation history
- Context retention within sessions
- RAG integration for knowledge-aware responses
- Conversation export
- Custom system prompts

**Settings:**

| Setting             | Description           | Default           |
| ------------------- | --------------------- | ----------------- |
| Ollama Model        | LLM for responses     | llama3.2          |
| System Prompt       | Personality/behavior  | Helpful assistant |
| Enable RAG          | Use knowledge base    | Off               |
| Conversation Memory | Messages to retain    | 10                |
| Auto-clear on Close | Clear history on exit | Off               |

**Location:** Settings > Ask AI

**Backend:**

- `src-tauri/src/managers/ask_ai.rs`
- `src-tauri/src/managers/ask_ai_history.rs`

**Frontend:** `src/components/settings/ask-ai/`

---

### RAG Knowledge Base

**Description:** Retrieval-Augmented Generation for context-aware AI responses.

**Use Cases:**

- Personal knowledge base queries
- Documentation search
- Project-specific context
- Research assistance

**How It Works:**

```
┌─────────────────────────────────────────────────────────────────┐
│                    RAG Knowledge Base                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Documents                                                      │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐                        │
│  │  .txt   │  │  .md    │  │  .pdf   │                        │
│  └────┬────┘  └────┬────┘  └────┬────┘                        │
│       │            │            │                              │
│       └────────────┼────────────┘                              │
│                    ▼                                            │
│           ┌─────────────────┐                                  │
│           │    Chunking     │ → Split into segments            │
│           └─────────────────┘                                  │
│                    │                                            │
│                    ▼                                            │
│           ┌─────────────────┐                                  │
│           │   Embedding     │ → Create vector embeddings       │
│           └─────────────────┘                                  │
│                    │                                            │
│                    ▼                                            │
│           ┌─────────────────┐                                  │
│           │  Vector Index   │ ← Stored for fast retrieval     │
│           └─────────────────┘                                  │
│                                                                 │
│  Query Flow:                                                    │
│  User Query → Embed → Search Index → Top K Results → LLM      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Features:**

- Document import (txt, md, pdf)
- Automatic chunking and embedding
- Fast vector similarity search
- Context injection into Ask AI queries
- Document management (add/remove)
- Index statistics

**Settings:**

| Setting              | Description         | Default    |
| -------------------- | ------------------- | ---------- |
| Chunk Size           | Text segment size   | 500 tokens |
| Overlap              | Chunk overlap       | 50 tokens  |
| Top K                | Results to retrieve | 5          |
| Similarity Threshold | Minimum relevance   | 0.7        |

**Location:** Settings > Knowledge Base

**Backend:** `src-tauri/src/managers/rag.rs`

**Frontend:** `src/components/settings/knowledge-base/`

---

### Suggestion Engine

**Description:** Context-aware quick responses and suggestions.

**Use Cases:**

- Quick replies in messaging apps
- Email response suggestions
- Meeting follow-up actions
- Smart shortcuts

**How It Works:**

```
Context Input → Ollama Analysis → Suggestions Generated → Display
      │                                    │
      ├──► Current transcription           │
      ├──► Active Listening buffer         │
      └──► RAG context                     ▼
                                   ┌─────────────────┐
                                   │ Suggestion List │
                                   │  • Reply 1      │
                                   │  • Reply 2      │
                                   │  • Action item  │
                                   └─────────────────┘
```

**Features:**

- Automatic suggestion generation
- Context from multiple sources
- Quick-paste suggestions
- Custom suggestion prompts
- Suggestion history

**Settings:**

| Setting            | Description      | Default       |
| ------------------ | ---------------- | ------------- |
| Enable Suggestions | Turn on/off      | Off           |
| Suggestion Count   | Number to show   | 3             |
| Context Sources    | What to analyze  | Transcription |
| Custom Prompt      | Suggestion style | Default       |

**Location:** Settings > Suggestions

**Backend:** `src-tauri/src/managers/suggestion_engine.rs`

**Frontend:** `src/components/settings/suggestions/`

---

## Advanced Features

### Custom Words

**Description:** Add custom words and corrections for better transcription accuracy.

**Use Cases:**

- Technical jargon
- Names and proper nouns
- Acronyms
- Domain-specific vocabulary

**How It Works:**

- Add word pairs: (misheard, correct)
- Applied as post-processing step
- Supports regex patterns

**Location:** Settings > Advanced > Custom Words

---

### Audio Device Selection

**Description:** Choose specific input/output devices.

**Features:**

- List all available audio devices
- Select preferred microphone
- Select output for playback
- Real-time device switching
- Device hot-plug detection

**Location:** Settings > Audio

---

### Debug Mode

**Description:** Advanced diagnostics and troubleshooting.

**Access:** `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux)

**Features:**

- Real-time audio level visualization
- VAD trigger visualization
- Model loading status
- Event log viewer
- Settings export/import
- System information

---

## Ollama Setup

Fork-exclusive features (Active Listening, Ask AI, RAG, Suggestions) require Ollama.

### Installation

**macOS:**

```bash
brew install ollama
```

**Linux:**

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

**Windows:**

Download from [ollama.ai](https://ollama.ai)

### Starting Ollama

```bash
# Start Ollama server
ollama serve

# Pull recommended model
ollama pull llama3.2

# For faster responses (smaller model)
ollama pull llama3.2:1b
```

### Recommended Models

| Model         | Size | Use Case                  |
| ------------- | ---- | ------------------------- |
| `llama3.2`    | 2GB  | General purpose (default) |
| `llama3.2:1b` | 1GB  | Faster, lower resource    |
| `mistral`     | 4GB  | Better reasoning          |
| `codellama`   | 4GB  | Code-focused              |
| `phi3`        | 2GB  | Efficient alternative     |

### Configuration in Handy

1. Ensure Ollama is running (`ollama serve`)
2. Go to Settings > Active Listening (or Ask AI)
3. Select your Ollama model
4. Optionally configure Ollama URL (default: `http://localhost:11434`)

### Troubleshooting

**"Ollama not connected":**

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# If not running, start it
ollama serve
```

**"Model not found":**

```bash
# List available models
ollama list

# Pull missing model
ollama pull <model-name>
```

**Slow performance:**

- Use smaller model (`llama3.2:1b`)
- Ensure GPU is being utilized
- Check system resources

---

## Feature Comparison

| Feature                   | Upstream | This Fork |
| ------------------------- | -------- | --------- |
| Speech-to-Text            | Yes      | Yes       |
| Multiple Models           | Yes      | Yes       |
| Post-Processing           | Yes      | Yes       |
| History                   | Yes      | Yes       |
| Custom Words              | Yes      | Yes       |
| **Active Listening**      | No       | Yes       |
| **Ask AI**                | No       | Yes       |
| **RAG Knowledge Base**    | No       | Yes       |
| **Suggestion Engine**     | No       | Yes       |
| **System Audio Loopback** | No       | Yes       |
| **Audio Mixer**           | No       | Yes       |

---

## Related Documentation

- [CLAUDE.md](CLAUDE.md) - Developer reference
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [DEV_DOCS.md](DEV_DOCS.md) - Technical reference

---

_Last updated: 2026-03-01_
