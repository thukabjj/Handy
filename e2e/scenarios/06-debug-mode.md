# Scenario: Debug Mode

Verify that the debug keyboard shortcut toggles the Debug section in the sidebar.

## Preconditions

Default mock state. Debug mode is off by default. Debug section is not visible in sidebar.

## Steps

### 1. Navigate and verify no Debug section

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
browser_snapshot()
```

**Verify:**
- Sidebar shows: General, Advanced, Batch Import, Vocabulary, History, About
- "Debug" is NOT in the sidebar

### 2. Press Cmd+Shift+D to enable debug mode

```
browser_press_key({ key: "Meta+Shift+KeyD" })
browser_snapshot()
```

**Verify:**
- "Debug" section now appears in the sidebar
- Sidebar shows: General, Advanced, Batch Import, Vocabulary, History, **Debug**, About

### 3. Click Debug section

```
browser_click({ ref: "<debug-ref>", element: "Debug sidebar item" })
browser_snapshot()
```

**Verify:**
- Debug content area is displayed
- Shows debug-related settings or information

### 4. Press Cmd+Shift+D again to disable

```
browser_press_key({ key: "Meta+Shift+KeyD" })
browser_snapshot()
```

**Verify:**
- "Debug" section is removed from sidebar
- App navigates back to a default section (e.g., General)

### 5. Verify state via evaluate

```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().settings.debug_mode" })
```

**Verify:**
- Returns `false` after toggling off

## Expected Results

- Cmd+Shift+D toggles debug mode on/off
- Debug sidebar section appears/disappears accordingly
- Debug content area renders when section is selected
- Keyboard shortcut works as expected in browser context

## Platform Notes

- macOS: `Meta+Shift+KeyD` (Cmd+Shift+D)
- Windows/Linux: `Control+Shift+KeyD` (Ctrl+Shift+D)
- In browser E2E, use `Meta+Shift+KeyD` since the mock returns `platform() = "macos"`
