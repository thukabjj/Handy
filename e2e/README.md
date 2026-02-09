# End-to-End Tests

This directory contains E2E test infrastructure for the Handy application.

## Important: Tauri App Limitations

**Handy is a Tauri application** - the frontend depends on `@tauri-apps/api` which only works inside the Tauri webview runtime. This means:

- The frontend **cannot run standalone** in a regular browser
- Browser-based E2E tools (Playwright, Cypress) won't work directly
- True E2E testing requires the native Tauri runtime

## Testing Options

### 1. Unit & Component Tests (Recommended - All Platforms)

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
| Frontend Unit (Vitest) | 132 | All |
| Backend Unit (Rust) | 72 | All |
| E2E (WebdriverIO) | ~30 | Linux/Windows |

**Total: 204+ automated tests**

## Why Playwright Tests Won't Work

When you run `bun run dev`, it only starts the Vite development server for the frontend. The frontend JavaScript immediately tries to call Tauri APIs like:

- `@tauri-apps/api/event` - Event system
- `@tauri-apps/plugin-store` - Settings persistence
- `@tauri-apps/plugin-os` - Platform detection

These APIs require the Tauri Rust backend running. Without it, the app shows a blank page with console errors like:
```
TypeError: Cannot read properties of undefined (reading 'invoke')
```

This is why we:
1. Use **mocked Tauri APIs** in unit tests (see `src/test/setup.ts`)
2. Use **tauri-driver** for E2E tests on supported platforms
3. Rely on **manual testing** via `bun run tauri dev` on macOS
