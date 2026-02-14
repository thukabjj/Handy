# Scenario: Toggle Settings

Verify that toggle switches in the General settings respond to clicks and update state.

## Preconditions

Default mock state. Push to Talk is off, Audio Feedback is on.

## Steps

### 1. Navigate and wait for General settings

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
browser_snapshot()
```

**Verify:**
- General settings section is visible
- Push to Talk toggle is present
- Audio Feedback toggle is present

### 2. Toggle Push to Talk ON

```
browser_click({ ref: "<ptt-toggle-ref>", element: "Push to Talk toggle" })
browser_snapshot()
```

**Verify:**
- Push to Talk toggle appears enabled/checked
- Mock state updated: `window.__E2E_MOCK__.getState().settings.push_to_talk === true`

### 3. Verify state via evaluate

```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().settings.push_to_talk" })
```

**Verify:**
- Returns `true`

### 4. Toggle Push to Talk OFF

```
browser_click({ ref: "<ptt-toggle-ref>", element: "Push to Talk toggle" })
browser_snapshot()
```

**Verify:**
- Push to Talk toggle appears disabled/unchecked

### 5. Toggle Audio Feedback OFF

```
browser_click({ ref: "<audio-feedback-ref>", element: "Audio Feedback toggle" })
```

**Verify state:**
```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().settings.audio_feedback" })
```

- Returns `false`

### 6. Toggle Autostart

Find and click the Autostart toggle (under Advanced or General depending on layout).

```
browser_click({ ref: "<autostart-ref>", element: "Autostart toggle" })
```

## Expected Results

- Toggle switches respond to click events
- Visual state updates immediately
- Mock state reflects the toggled values
- Toggles can be switched back and forth
