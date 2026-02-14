# End-to-End Tests

This directory contains E2E test infrastructure for the Handy application.

## Important: Tauri App Limitations

**Handy is a Tauri application** - the frontend depends on `@tauri-apps/api` which only works inside the Tauri webview runtime. This means:

- The frontend **cannot run standalone** in a regular browser
- Browser-based E2E tools (Playwright, Cypress) won't work directly
- True E2E testing requires the native Tauri runtime

## Testing Options

### 1. Browser E2E with Playwright MCP (All Platforms - Recommended for Frontend)

A browser-compatible E2E build that replaces all `@tauri-apps/*` imports with mock modules via Vite aliases. The full React frontend renders in a standard browser, testable with Claude Code's Playwright MCP or any browser automation tool.

```bash
# Start the E2E dev server
bun run dev:e2e

# App available at http://localhost:1420
```

**How it works:**
- `e2e/vite.config.e2e.ts` aliases 16 Tauri imports to browser-compatible mocks
- `e2e/browser-mocks/` contains mock implementations for all Tauri APIs
- `e2e/browser-mocks/mock-state.ts` provides centralized state with `window.__E2E_MOCK__` API
- Zero production code changes — all mocking happens at build time

**What it tests:**
- Full React frontend rendering and routing
- Sidebar navigation and section filtering
- Settings toggles and state management
- Onboarding flow (configurable via mock state)
- Keyboard shortcuts (Cmd+Shift+D for debug mode)
- Model selector UI
- History viewer with mock entries
- i18n translations

**Mock state control (via browser console or Playwright evaluate):**
```javascript
window.__E2E_MOCK__.getState()                          // Get current state
window.__E2E_MOCK__.reset()                              // Reset to defaults
window.__E2E_MOCK__.updateSettings({ push_to_talk: true }) // Patch settings
window.__E2E_MOCK__.update({ hasAnyModels: false })      // Patch any state
```

See `e2e/scenarios/` for step-by-step test guides.

### 2. Unit & Component Tests (Recommended - All Platforms)

The existing Vitest test suite mocks all Tauri APIs and provides comprehensive coverage:

```bash
# Run all frontend unit tests (132 tests)
bun run test

# Run with UI
bun run test:ui

# Run with coverage
bun run test:coverage
```

**Coverage includes:**
- UI components (Button, Dropdown, ToggleSwitch, Slider, Input, Skeleton)
- Settings store (Zustand state management)
- All Tauri API calls are mocked in `src/test/setup.ts`

### 2. Rust Backend Tests (All Platforms)

```bash
# Run all Rust tests (72 tests)
bun run test:rust

# Or directly with cargo
cd src-tauri && cargo test
```

**Coverage includes:**
- Settings management
- Active listening configuration
- Text processing (custom words)
- Ollama client

### 3. WebdriverIO + tauri-driver (Linux/Windows Only)

For true E2E testing of the native app:

```bash
# Install tauri-driver (one-time setup)
cargo install tauri-driver

# Build the release app
bun run e2e:build

# Run E2E tests
bun run e2e:wdio
```

**Note**: Not supported on macOS (WKWebView lacks WebDriver).

### 4. Manual Testing with Tauri Dev

```bash
# Run the full Tauri app in dev mode
bun run tauri dev

# Then manually test features:
# - Settings navigation
# - Toggle switches
# - Keyboard shortcuts (Cmd+Shift+D for debug)
# - Audio recording (requires permissions)
```

## Directory Structure

```
e2e/
├── README.md                  # This file
├── vite.config.e2e.ts         # Vite config with Tauri mock aliases
├── browser-mocks/             # Browser-compatible Tauri API mocks
│   ├── mock-state.ts          # Centralized state + window.__E2E_MOCK__
│   ├── tauri-api-core.ts      # invoke() handler for all 90+ commands
│   ├── tauri-api-event.ts     # listen(), emit(), once()
│   ├── tauri-api-window.ts    # getCurrentWindow() stub
│   ├── tauri-api-webview-window.ts
│   ├── tauri-api-app.ts       # getVersion()
│   ├── tauri-plugin-os.ts     # platform(), type(), locale()
│   ├── tauri-plugin-opener.ts
│   ├── tauri-plugin-clipboard.ts
│   ├── tauri-plugin-store.ts
│   ├── tauri-plugin-autostart.ts
│   ├── tauri-plugin-dialog.ts
│   ├── tauri-plugin-updater.ts
│   ├── tauri-plugin-process.ts
│   ├── tauri-plugin-fs.ts
│   ├── tauri-plugin-global-shortcut.ts
│   └── tauri-plugin-macos-perms.ts
├── scenarios/                 # Step-by-step Playwright MCP test guides
│   ├── README.md
│   ├── 01-settings-navigation.md
│   ├── 02-toggle-settings.md
│   ├── 03-model-selection.md
│   ├── 04-active-listening.md
│   ├── 05-onboarding-flow.md
│   ├── 06-debug-mode.md
│   ├── 07-history-viewer.md
│   └── 08-sound-detection.md
├── playwright.config.js       # Playwright config (reference only)
├── wdio.conf.js              # WebdriverIO config (Linux/Windows)
├── playwright/               # Playwright specs (reference only)
│   ├── app.spec.js
│   └── settings.spec.js
└── test/                     # WebdriverIO specs
    └── specs/
        ├── app.e2e.js
        ├── settings.e2e.js
        ├── onboarding.e2e.js
        └── keyboard.e2e.js
```

## CI/CD Recommendations

### For macOS CI
Use unit tests only:
```yaml
- name: Run tests
  run: |
    bun run test
    bun run test:rust
```

### For Linux CI
Full E2E testing available:
```yaml
- name: Install tauri-driver
  run: cargo install tauri-driver

- name: Build app
  run: bun run e2e:build

- name: Run unit tests
  run: bun run test && bun run test:rust

- name: Run E2E tests
  run: xvfb-run bun run e2e:wdio
```

## Test Coverage Summary

| Type | Tests | Platform |
|------|-------|----------|
| Frontend Unit (Vitest) | 151 | All |
| Backend Unit (Rust) | 72 | All |
| Browser E2E (Playwright MCP) | 8 scenarios | All |
| E2E (WebdriverIO) | ~30 | Linux/Windows |

**Total: 223+ automated tests + 8 interactive scenarios**

## Why `bun run dev` Alone Won't Work for E2E

When you run `bun run dev`, it starts the Vite dev server but the frontend immediately calls Tauri APIs (`@tauri-apps/api/event`, `@tauri-apps/plugin-store`, etc.) which require the native Tauri runtime. Without it, the app shows a blank page.

**Solution:** Use `bun run dev:e2e` which aliases all Tauri imports to browser-compatible mocks at build time, allowing the full frontend to render in any browser.
