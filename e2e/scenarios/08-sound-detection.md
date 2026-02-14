# Scenario: Sound Detection Settings

Verify the Sound Detection settings UI renders and responds to interactions.

## Preconditions

Default mock state. Sound Detection is disabled by default.

## Steps

### 1. Navigate to General settings

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
browser_snapshot()
```

**Verify:**
- General settings section is visible
- Sound Detection section is present (may need to scroll)

### 2. Find Sound Detection toggle

```
browser_snapshot()
```

Look for "Sound Detection" or similar label with a toggle switch.

### 3. Enable Sound Detection

```
browser_click({ ref: "<sound-detection-toggle-ref>", element: "Sound Detection toggle" })
browser_snapshot()
```

**Verify:**
- Sound Detection toggle is now enabled
- Additional settings may appear (threshold, alert sound, etc.)

### 4. Verify state

```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().settings.sound_detection_enabled" })
```

**Verify:**
- Returns `true`

### 5. Adjust threshold (if slider is present)

If a threshold slider appears after enabling:

```
browser_click({ ref: "<threshold-slider-ref>", element: "Sound detection threshold slider" })
```

### 6. Disable Sound Detection

```
browser_click({ ref: "<sound-detection-toggle-ref>", element: "Sound Detection toggle" })
```

**Verify:**
- Toggle returns to disabled state
- Additional settings hidden

## Expected Results

- Sound Detection toggle enables/disables correctly
- Related settings appear when enabled
- Threshold controls are interactive
- Mock state reflects all changes
