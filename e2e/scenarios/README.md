# E2E Scenario Documentation

Step-by-step guides for testing Dictum's frontend using Claude Code's Playwright MCP tools.

## How It Works

The E2E build replaces all `@tauri-apps/*` imports with browser-compatible mocks via Vite aliases. The full React frontend renders in a standard browser at `http://localhost:1420`, and Claude Code's Playwright MCP interacts with it natively.

## Setup

```bash
# Start the E2E dev server (background)
bun run dev:e2e &

# Wait for Vite to start, then navigate
browser_navigate({ url: "http://localhost:1420" })
browser_snapshot()
```

## Available Playwright MCP Tools

| Tool | Purpose |
|------|---------|
| `browser_navigate` | Go to a URL |
| `browser_snapshot` | Get accessibility tree with element refs |
| `browser_click` | Click an element by ref |
| `browser_type` | Type text into an input |
| `browser_press_key` | Press keyboard shortcuts |
| `browser_evaluate` | Run JS (e.g., manipulate mock state) |
| `browser_take_screenshot` | Capture a visual screenshot |
| `browser_fill_form` | Fill multiple form fields |
| `browser_select_option` | Select dropdown option |

## Mock State API

All mocks share centralized state accessible via `window.__E2E_MOCK__`:

```javascript
// Get current state
window.__E2E_MOCK__.getState()

// Reset to defaults (returning user, models downloaded)
window.__E2E_MOCK__.reset()

// Patch settings
window.__E2E_MOCK__.updateSettings({ push_to_talk: true })

// Patch any state
window.__E2E_MOCK__.update({ hasAnyModels: false })
```

After changing state, reload the page for the app to reinitialize:

```javascript
window.__E2E_MOCK__.update({ hasAnyModels: false })
location.reload()
```

## Reset Between Tests

```javascript
browser_evaluate({ function: "() => { window.__E2E_MOCK__.reset(); location.reload(); }" })
```

## Scenarios

1. [Settings Navigation](01-settings-navigation.md) - Sidebar navigation between sections
2. [Toggle Settings](02-toggle-settings.md) - Toggle switches for various settings
3. [Model Selection](03-model-selection.md) - Model dropdown and selection
4. [Active Listening](04-active-listening.md) - Active Listening settings
5. [Onboarding Flow](05-onboarding-flow.md) - First-run experience (no models)
6. [Debug Mode](06-debug-mode.md) - Debug section via keyboard shortcut
7. [History Viewer](07-history-viewer.md) - Transcription history entries
8. [Sound Detection](08-sound-detection.md) - Sound Detection settings

## What This Tests

- Full React frontend rendering and routing
- State management (Zustand stores)
- Conditional rendering and onboarding logic
- i18n translations
- Keyboard shortcuts
- Settings toggles and persistence (via mocks)
- Sidebar navigation and section filtering

## What This Does NOT Test

- Rust backend (audio recording, model inference, transcription)
- Native window behaviors (tray icon, overlay, minimize)
- Real file system or clipboard operations
- Platform-specific behaviors beyond mock simulation
- IPC latency and real backend error handling
