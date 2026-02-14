# Scenario: Active Listening Settings

Verify Active Listening configuration UI renders and responds to interactions.

## Preconditions

Default mock state. Active Listening is disabled by default.

## Steps

### 1. Navigate to General settings

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
browser_snapshot()
```

### 2. Enable Active Listening

Look for the Active Listening toggle in the settings and enable it.

```
browser_click({ ref: "<active-listening-toggle-ref>", element: "Active Listening toggle" })
browser_snapshot()
```

**Verify:**
- Active Listening toggle is now enabled
- Additional Active Listening settings may become visible

### 3. Check Active Listening section in sidebar

If Active Listening has its own sidebar section (visible when enabled), click it.

```
browser_snapshot()
```

**Verify:**
- Active Listening related settings are accessible
- Ollama connection status indicator may be visible

### 4. Verify state

```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().settings.active_listening_enabled" })
```

**Verify:**
- Returns `true`

### 5. Configure Active Listening prompt

If a prompt/template field is visible, type into it.

```
browser_type({ ref: "<prompt-field-ref>", text: "Summarize the meeting discussion" })
```

### 6. Disable Active Listening

```
browser_click({ ref: "<active-listening-toggle-ref>", element: "Active Listening toggle" })
```

**Verify:**
- Toggle returns to disabled state
- Additional settings may be hidden

## Expected Results

- Active Listening toggle enables/disables correctly
- Related settings appear/disappear based on toggle state
- Prompt field accepts text input
- Mock state reflects all changes
