# Contributing to Handy

Thank you for your interest in contributing to Handy! This guide will help you get started with contributing to this open source speech-to-text application.

## Philosophy

Handy aims to be the most forkable speech-to-text app. The goal is to create both a useful tool and a foundation for others to build upon -- a well-patterned, simple codebase that serves the community. We prioritize:

- **Simplicity**: Clear, maintainable code over clever solutions
- **Extensibility**: Make it easy for others to fork and customize
- **Privacy**: Keep everything local and offline
- **Accessibility**: Free tooling that belongs in everyone's hands

---

## Quick Start

```bash
# 1. Fork and clone
git clone git@github.com:YOUR_USERNAME/Handy.git
cd Handy
git remote add upstream git@github.com:cjpais/Handy.git

# 2. Install dependencies
make install

# 3. Download VAD model (required)
mkdir -p src-tauri/resources/models
curl -o src-tauri/resources/models/silero_vad_v4.onnx https://blob.handy.computer/silero_vad_v4.onnx

# 4. Run in development mode
make dev
```

---

## Prerequisites

| Tool | Version | Installation |
|------|---------|--------------|
| **Rust** | Latest stable | [rustup.rs](https://rustup.rs/) |
| **Bun** | Latest | [bun.sh](https://bun.sh/) |
| **CMake** | 3.5+ | `brew install cmake` (macOS) |
| **Platform tools** | - | See [BUILD.md](BUILD.md) |

### Platform-Specific Requirements

**macOS:**
```bash
xcode-select --install  # Xcode Command Line Tools
```

**Linux (Debian/Ubuntu):**
```bash
sudo apt install build-essential pkg-config libssl-dev libasound2-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

**Windows:**
- Visual Studio Build Tools with C++ workload
- WebView2 Runtime

---

## Development Workflow

### Daily Development

```bash
# Start development server
make dev

# Or with CMake version workaround (macOS)
CMAKE_POLICY_VERSION_MINIMUM=3.5 bun run tauri dev

# Frontend only (faster iteration)
bun run dev
```

### Code Quality Checks

```bash
# Run before every commit
make check              # Full check (Rust + TypeScript)

# Individual checks
bun run lint            # ESLint for frontend
bun run lint:fix        # ESLint with auto-fix
bun run format          # Prettier + cargo fmt
cargo clippy            # Rust linter
cargo fmt               # Rust formatter
```

### Testing

```bash
# All tests
make test

# Frontend tests (Vitest)
bun run test
bun run test:watch      # Watch mode

# Rust tests
make test-rust
cargo test              # Direct cargo test
```

### Building

```bash
# Production build
bun run tauri build

# macOS DMG
make build-dmg
```

---

## Code Patterns

This section provides copy-paste ready examples for adding new functionality. Each pattern follows the established codebase conventions.

### Adding a New Manager (Backend)

Managers encapsulate business logic and state. They're initialized at startup and accessed via Tauri state.

**Step 1: Create the manager file**

```rust
// src-tauri/src/managers/my_feature.rs

use crate::error::{HandyError, HandyResult};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;

/// Manages my feature state and operations.
pub struct MyFeatureManager {
    app_handle: AppHandle,
    is_enabled: AtomicBool,
    data: RwLock<Vec<String>>,
}

impl MyFeatureManager {
    /// Creates a new MyFeatureManager.
    ///
    /// # Arguments
    /// * `app_handle` - Tauri application handle for events and state
    ///
    /// # Returns
    /// * `HandyResult<Self>` - The manager instance or an error
    pub fn new(app_handle: AppHandle) -> HandyResult<Self> {
        Ok(Self {
            app_handle,
            is_enabled: AtomicBool::new(false),
            data: RwLock::new(Vec::new()),
        })
    }

    /// Enables the feature.
    pub fn enable(&self) {
        self.is_enabled.store(true, Ordering::SeqCst);
        // Emit event to frontend
        let _ = self.app_handle.emit("my-feature-enabled", ());
    }

    /// Disables the feature.
    pub fn disable(&self) {
        self.is_enabled.store(false, Ordering::SeqCst);
        let _ = self.app_handle.emit("my-feature-disabled", ());
    }

    /// Checks if the feature is enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled.load(Ordering::SeqCst)
    }

    /// Adds data to the manager.
    pub async fn add_data(&self, item: String) -> HandyResult<()> {
        if !self.is_enabled() {
            return Err(HandyError::state("Feature is not enabled")
                .with_suggestion("Enable the feature first"));
        }

        let mut data = self.data.write().await;
        data.push(item.clone());

        // Emit event with payload
        let _ = self.app_handle.emit("my-feature-data-added", &item);
        Ok(())
    }

    /// Gets all data.
    pub async fn get_data(&self) -> Vec<String> {
        self.data.read().await.clone()
    }
}
```

**Step 2: Register in managers/mod.rs**

```rust
// src-tauri/src/managers/mod.rs

// Add to existing exports
pub mod my_feature;
pub use my_feature::MyFeatureManager;
```

**Step 3: Initialize in lib.rs**

```rust
// src-tauri/src/lib.rs

use crate::managers::MyFeatureManager;

// In run() function, after other manager initializations:
let my_feature_manager = Arc::new(
    MyFeatureManager::new(app.handle().clone())
        .expect("Failed to initialize MyFeatureManager")
);
app.manage(my_feature_manager);
```

---

### Adding a New Command (Backend)

Commands expose Rust functions to the frontend via Tauri IPC.

**Step 1: Create the commands file**

```rust
// src-tauri/src/commands/my_feature.rs

use crate::error::HandyResult;
use crate::managers::MyFeatureManager;
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Enables my feature.
#[tauri::command]
#[specta::specta]
pub async fn enable_my_feature(
    _app: AppHandle,
    manager: State<'_, Arc<MyFeatureManager>>,
) -> Result<(), String> {
    manager.enable();
    Ok(())
}

/// Disables my feature.
#[tauri::command]
#[specta::specta]
pub async fn disable_my_feature(
    _app: AppHandle,
    manager: State<'_, Arc<MyFeatureManager>>,
) -> Result<(), String> {
    manager.disable();
    Ok(())
}

/// Gets the current feature status.
#[tauri::command]
#[specta::specta]
pub async fn get_my_feature_status(
    _app: AppHandle,
    manager: State<'_, Arc<MyFeatureManager>>,
) -> Result<bool, String> {
    Ok(manager.is_enabled())
}

/// Adds data to my feature.
#[tauri::command]
#[specta::specta]
pub async fn add_my_feature_data(
    _app: AppHandle,
    manager: State<'_, Arc<MyFeatureManager>>,
    item: String,
) -> Result<(), String> {
    manager.add_data(item).await.map_err(|e| e.to_string())
}

/// Gets all data from my feature.
#[tauri::command]
#[specta::specta]
pub async fn get_my_feature_data(
    _app: AppHandle,
    manager: State<'_, Arc<MyFeatureManager>>,
) -> Result<Vec<String>, String> {
    Ok(manager.get_data().await)
}
```

**Step 2: Register in commands/mod.rs**

```rust
// src-tauri/src/commands/mod.rs

// Add to existing exports
pub mod my_feature;
pub use my_feature::*;
```

**Step 3: Register commands in lib.rs**

```rust
// src-tauri/src/lib.rs

// In the tauri::Builder chain, add to invoke_handler:
.invoke_handler(tauri::generate_handler![
    // ... existing commands ...
    commands::enable_my_feature,
    commands::disable_my_feature,
    commands::get_my_feature_status,
    commands::add_my_feature_data,
    commands::get_my_feature_data,
])

// In collect_types!() for specta:
collect_types![
    // ... existing types ...
    commands::enable_my_feature,
    commands::disable_my_feature,
    commands::get_my_feature_status,
    commands::add_my_feature_data,
    commands::get_my_feature_data,
]
```

**Step 4: Regenerate TypeScript bindings**

```bash
make dev
# bindings.ts is auto-regenerated on startup
```

---

### Adding a New Setting (Backend)

Settings are persisted via `tauri-plugin-store` with serde serialization.

**Step 1: Create settings module**

```rust
// src-tauri/src/settings/my_feature.rs

use serde::{Deserialize, Serialize};

/// Settings for my feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MyFeatureSettings {
    /// Whether the feature is enabled by default.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Maximum number of items to store.
    #[serde(default = "default_max_items")]
    pub max_items: u32,

    /// Custom label for the feature.
    #[serde(default)]
    pub label: Option<String>,
}

fn default_enabled() -> bool {
    false
}

fn default_max_items() -> u32 {
    100
}

impl Default for MyFeatureSettings {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_items: default_max_items(),
            label: None,
        }
    }
}
```

**Step 2: Register in settings/mod.rs**

```rust
// src-tauri/src/settings/mod.rs

pub mod my_feature;
pub use my_feature::MyFeatureSettings;
```

**Step 3: Add to main Settings struct**

```rust
// src-tauri/src/settings/mod.rs (in Settings struct)

#[derive(Debug, Clone, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    // ... existing fields ...

    #[serde(default)]
    pub my_feature: MyFeatureSettings,
}
```

---

### Adding a Frontend Setting Component

Settings UI follows the `SettingContainer` pattern with i18n support.

**Step 1: Create the component**

```tsx
// src/components/settings/my-feature/MyFeatureToggle.tsx

import React, { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { SettingContainer } from "../SettingContainer";
import { Toggle } from "../../ui/Toggle";
import { useSettings } from "../../../hooks/useSettings";

export const MyFeatureToggle: React.FC = React.memo(() => {
  const { t } = useTranslation();
  const { getSetting, updateSetting, loading } = useSettings();

  const enabled = getSetting("myFeature.enabled") ?? false;

  const handleToggle = useCallback(
    async (value: boolean) => {
      await updateSetting("myFeature.enabled", value);
    },
    [updateSetting]
  );

  return (
    <SettingContainer
      title={t("settings.myFeature.toggle.title")}
      description={t("settings.myFeature.toggle.description")}
    >
      <Toggle
        checked={enabled}
        onCheckedChange={handleToggle}
        disabled={loading}
        aria-label={t("settings.myFeature.toggle.title")}
      />
    </SettingContainer>
  );
});

MyFeatureToggle.displayName = "MyFeatureToggle";
```

**Step 2: Create the settings page**

```tsx
// src/components/settings/my-feature/MyFeatureSettings.tsx

import React from "react";
import { useTranslation } from "react-i18next";
import { SettingPage } from "../SettingPage";
import { MyFeatureToggle } from "./MyFeatureToggle";
import { MyFeatureMaxItems } from "./MyFeatureMaxItems";

export const MyFeatureSettings: React.FC = React.memo(() => {
  const { t } = useTranslation();

  return (
    <SettingPage
      title={t("settings.myFeature.title")}
      description={t("settings.myFeature.description")}
    >
      <MyFeatureToggle />
      <MyFeatureMaxItems />
    </SettingPage>
  );
});

MyFeatureSettings.displayName = "MyFeatureSettings";
```

**Step 3: Create the index export**

```tsx
// src/components/settings/my-feature/index.tsx

export { MyFeatureSettings } from "./MyFeatureSettings";
export { MyFeatureToggle } from "./MyFeatureToggle";
```

**Step 4: Add i18n translations**

```json
// src/i18n/locales/en/translation.json

{
  "settings": {
    "myFeature": {
      "title": "My Feature",
      "description": "Configure my feature settings",
      "toggle": {
        "title": "Enable My Feature",
        "description": "Turn on to activate my feature functionality"
      },
      "maxItems": {
        "title": "Maximum Items",
        "description": "Set the maximum number of items to store"
      }
    }
  }
}
```

**Step 5: Register in settings navigation**

```tsx
// src/components/settings/SettingsNavigation.tsx

import { MyFeatureSettings } from "./my-feature";

// Add to navigation items:
{
  id: "my-feature",
  label: t("settings.myFeature.title"),
  icon: <MyFeatureIcon />,
  component: <MyFeatureSettings />,
}
```

---

### Adding a Frontend Hook

Hooks encapsulate reusable logic with proper TypeScript types.

```tsx
// src/hooks/useMyFeature.ts

import { useCallback, useEffect, useState } from "react";
import { commands } from "../bindings";
import { listen } from "@tauri-apps/api/event";

interface MyFeatureState {
  enabled: boolean;
  data: string[];
  loading: boolean;
  error: string | null;
}

interface UseMyFeatureReturn extends MyFeatureState {
  enable: () => Promise<void>;
  disable: () => Promise<void>;
  addData: (item: string) => Promise<void>;
  refresh: () => Promise<void>;
}

export function useMyFeature(): UseMyFeatureReturn {
  const [state, setState] = useState<MyFeatureState>({
    enabled: false,
    data: [],
    loading: true,
    error: null,
  });

  // Fetch initial state
  const refresh = useCallback(async () => {
    setState((prev) => ({ ...prev, loading: true, error: null }));
    try {
      const [statusResult, dataResult] = await Promise.all([
        commands.getMyFeatureStatus(),
        commands.getMyFeatureData(),
      ]);

      if (statusResult.status === "ok" && dataResult.status === "ok") {
        setState({
          enabled: statusResult.data,
          data: dataResult.data,
          loading: false,
          error: null,
        });
      } else {
        throw new Error(statusResult.error || dataResult.error);
      }
    } catch (err) {
      setState((prev) => ({
        ...prev,
        loading: false,
        error: err instanceof Error ? err.message : "Unknown error",
      }));
    }
  }, []);

  // Enable feature
  const enable = useCallback(async () => {
    const result = await commands.enableMyFeature();
    if (result.status === "error") {
      setState((prev) => ({ ...prev, error: result.error }));
    }
  }, []);

  // Disable feature
  const disable = useCallback(async () => {
    const result = await commands.disableMyFeature();
    if (result.status === "error") {
      setState((prev) => ({ ...prev, error: result.error }));
    }
  }, []);

  // Add data
  const addData = useCallback(async (item: string) => {
    const result = await commands.addMyFeatureData(item);
    if (result.status === "error") {
      setState((prev) => ({ ...prev, error: result.error }));
    }
  }, []);

  // Listen for events
  useEffect(() => {
    const unlisteners: (() => void)[] = [];

    const setup = async () => {
      unlisteners.push(
        await listen("my-feature-enabled", () => {
          setState((prev) => ({ ...prev, enabled: true }));
        })
      );

      unlisteners.push(
        await listen("my-feature-disabled", () => {
          setState((prev) => ({ ...prev, enabled: false }));
        })
      );

      unlisteners.push(
        await listen<string>("my-feature-data-added", (event) => {
          setState((prev) => ({
            ...prev,
            data: [...prev.data, event.payload],
          }));
        })
      );
    };

    setup();
    refresh();

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, [refresh]);

  return {
    ...state,
    enable,
    disable,
    addData,
    refresh,
  };
}
```

---

### Adding a Zustand Store

For complex state that needs to be shared across many components.

```tsx
// src/stores/myFeatureStore.ts

import { create } from "zustand";
import { subscribeWithSelector } from "zustand/middleware";
import { commands } from "../bindings";

interface MyFeatureState {
  // State
  items: string[];
  selectedItem: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  fetchItems: () => Promise<void>;
  addItem: (item: string) => Promise<void>;
  selectItem: (item: string | null) => void;
  clearError: () => void;
}

export const useMyFeatureStore = create<MyFeatureState>()(
  subscribeWithSelector((set, get) => ({
    // Initial state
    items: [],
    selectedItem: null,
    isLoading: false,
    error: null,

    // Fetch items from backend
    fetchItems: async () => {
      set({ isLoading: true, error: null });
      try {
        const result = await commands.getMyFeatureData();
        if (result.status === "ok") {
          set({ items: result.data, isLoading: false });
        } else {
          set({ error: result.error, isLoading: false });
        }
      } catch (err) {
        set({
          error: err instanceof Error ? err.message : "Failed to fetch items",
          isLoading: false,
        });
      }
    },

    // Add item with optimistic update
    addItem: async (item: string) => {
      const previousItems = get().items;
      // Optimistic update
      set({ items: [...previousItems, item] });

      try {
        const result = await commands.addMyFeatureData(item);
        if (result.status === "error") {
          // Rollback on error
          set({ items: previousItems, error: result.error });
        }
      } catch (err) {
        // Rollback on error
        set({
          items: previousItems,
          error: err instanceof Error ? err.message : "Failed to add item",
        });
      }
    },

    // Select item
    selectItem: (item: string | null) => {
      set({ selectedItem: item });
    },

    // Clear error
    clearError: () => {
      set({ error: null });
    },
  }))
);

// Selector hooks for performance
export const useMyFeatureItems = () =>
  useMyFeatureStore((state) => state.items);

export const useMyFeatureLoading = () =>
  useMyFeatureStore((state) => state.isLoading);

export const useMyFeatureError = () =>
  useMyFeatureStore((state) => state.error);
```

---

## Error Handling Patterns

### Backend Errors

Always use `HandyError` with appropriate categories:

```rust
use crate::error::{HandyError, HandyResult};

fn my_function() -> HandyResult<String> {
    // Validation error
    if input.is_empty() {
        return Err(HandyError::validation("Input cannot be empty")
            .with_suggestion("Provide a non-empty value"));
    }

    // State error
    if !self.is_initialized() {
        return Err(HandyError::state("Manager not initialized")
            .recoverable()
            .with_suggestion("Call initialize() first"));
    }

    // Network error
    let response = reqwest::get(url)
        .await
        .map_err(|e| HandyError::network(format!("Request failed: {}", e)))?;

    // Audio error
    if device.is_none() {
        return Err(HandyError::audio("No audio device found")
            .with_details("No input devices detected")
            .with_suggestion("Connect a microphone and try again"));
    }

    Ok("Success".to_string())
}
```

### Frontend Errors

Use the error store for global error handling:

```tsx
import { useErrorStore } from "../stores/errorStore";

function MyComponent() {
  const setError = useErrorStore((state) => state.setError);

  const handleAction = async () => {
    try {
      const result = await commands.myCommand();
      if (result.status === "error") {
        setError({
          message: result.error,
          recoverable: true,
          suggestion: "Try again later",
        });
      }
    } catch (err) {
      setError({
        message: err instanceof Error ? err.message : "Unknown error",
        recoverable: false,
      });
    }
  };
}
```

---

## Internationalization (i18n)

All user-facing strings must use i18next. ESLint enforces this.

### Adding Translations

**1. Add to English locale (required):**

```json
// src/i18n/locales/en/translation.json
{
  "myFeature": {
    "title": "My Feature",
    "button": {
      "save": "Save",
      "cancel": "Cancel"
    },
    "message": {
      "success": "Operation completed successfully",
      "error": "An error occurred: {{message}}"
    }
  }
}
```

**2. Use in components:**

```tsx
import { useTranslation } from "react-i18next";

function MyComponent() {
  const { t } = useTranslation();

  return (
    <div>
      <h1>{t("myFeature.title")}</h1>
      <button>{t("myFeature.button.save")}</button>
      {error && <p>{t("myFeature.message.error", { message: error })}</p>}
    </div>
  );
}
```

**3. Supported locales (17):**

```
ar, cs, de, en, es, fr, it, ja, ko, pl, pt, ru, tr, uk, vi, zh, zh-TW
```

---

## Testing Patterns

### Frontend Tests (Vitest)

```tsx
// src/components/__tests__/MyComponent.test.tsx

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MyComponent } from "../MyComponent";

// Mock Tauri commands
vi.mock("../../bindings", () => ({
  commands: {
    myCommand: vi.fn(),
  },
}));

describe("MyComponent", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders correctly", () => {
    render(<MyComponent />);
    expect(screen.getByText("My Feature")).toBeInTheDocument();
  });

  it("calls command on button click", async () => {
    const { commands } = await import("../../bindings");
    (commands.myCommand as ReturnType<typeof vi.fn>).mockResolvedValue({
      status: "ok",
      data: "success",
    });

    render(<MyComponent />);
    fireEvent.click(screen.getByRole("button", { name: /save/i }));

    expect(commands.myCommand).toHaveBeenCalled();
  });
});
```

### Rust Tests

```rust
// src-tauri/src/managers/my_feature.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enable_disable() {
        // Note: AppHandle requires a full Tauri setup, so use mock or skip
        // For unit tests, test pure logic functions
    }

    #[tokio::test]
    async fn test_data_operations() {
        let data: RwLock<Vec<String>> = RwLock::new(Vec::new());

        {
            let mut guard = data.write().await;
            guard.push("test".to_string());
        }

        let guard = data.read().await;
        assert_eq!(guard.len(), 1);
        assert_eq!(guard[0], "test");
    }
}
```

---

## Commit Guidelines

Use conventional commits for clear history:

| Prefix | Use Case | Example |
|--------|----------|---------|
| `feat:` | New features | `feat: add voice command support` |
| `fix:` | Bug fixes | `fix: resolve audio device selection issue` |
| `docs:` | Documentation | `docs: update contributing guide` |
| `refactor:` | Code refactoring | `refactor: simplify manager initialization` |
| `test:` | Test additions/changes | `test: add unit tests for AudioManager` |
| `chore:` | Maintenance | `chore: update dependencies` |
| `perf:` | Performance | `perf: optimize transcription pipeline` |

**Examples:**

```bash
git commit -m "feat: add active listening session management"
git commit -m "fix: handle missing microphone gracefully"
git commit -m "docs: document manager pattern in CONTRIBUTING.md"
git commit -m "refactor: extract audio utilities to separate module"
```

---

## Pull Request Process

### Before Submitting

1. **Search existing issues and PRs** - Check both open AND closed
2. **Run all checks**: `make check && make test`
3. **Update documentation** if adding features
4. **Add translations** for any new user-facing strings

### PR Template

```markdown
## Summary

Brief description of changes.

## Changes

- Added X
- Fixed Y
- Updated Z

## Testing

- [ ] Tested on macOS
- [ ] Tested on Windows
- [ ] Tested on Linux
- [ ] Added unit tests
- [ ] Manual testing completed

## Screenshots/Videos

(if applicable)

## AI Disclosure

- AI assisted: Yes/No
- Tools used: (e.g., Claude Code, GitHub Copilot)
- Extent: (e.g., boilerplate, debugging, implementation)
```

### Review Process

1. Maintainers review for code quality and patterns
2. CI must pass (lint, format, tests)
3. At least one approval required
4. Squash merge to main

---

## Upstream Sync (For Fork Contributors)

This fork tracks [cjpais/Handy](https://github.com/cjpais/Handy). To sync:

```bash
# Fetch upstream changes
git fetch upstream

# Create sync branch
git checkout -b feature/upstream-sync-$(date +%Y%m%d)

# Merge upstream (prefer upstream for conflicts in core, keep local for features)
git merge upstream/main

# Resolve conflicts, then test
make check && make test

# Merge to main
git checkout main
git merge feature/upstream-sync-$(date +%Y%m%d)
git push origin main
```

### Conflict Resolution

| Area | Priority |
|------|----------|
| Core transcription | Accept upstream |
| Settings structure | Merge carefully |
| UI components | Accept upstream, re-add local features |
| `lib.rs` | Re-add local manager initializations |
| Dependencies | Prefer upstream, keep local-only deps |

See [UPSTREAM_TRACKING.md](UPSTREAM_TRACKING.md) for PR status and sync history.

---

## Getting Help

- **Discord**: [discord.com/invite/WVBeWsNXK4](https://discord.com/invite/WVBeWsNXK4)
- **Discussions**: [github.com/cjpais/Handy/discussions](https://github.com/cjpais/Handy/discussions)
- **Debug Mode**: `Cmd+Shift+D` (macOS) or `Ctrl+Shift+D` (Windows/Linux)

---

## License

By contributing, you agree that your contributions will be licensed under the MIT License. See [LICENSE](LICENSE).

---

**Thank you for contributing to Handy!**
