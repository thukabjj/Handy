# Scenario: Model Selection

Verify the model selector in the footer and model-related UI.

## Preconditions

Default mock state. Active model is "whisper-tiny" (downloaded). Two other models available (whisper-base downloaded, whisper-small not downloaded).

## Steps

### 1. Navigate and verify model selector

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "Whisper Tiny" })
browser_snapshot()
```

**Verify:**
- Footer shows model selector button with "Whisper Tiny"
- Model status indicator is present

### 2. Click the model selector

```
browser_click({ ref: "<model-selector-ref>", element: "Model selector button" })
browser_snapshot()
```

**Verify:**
- Model dropdown opens
- Shows available models (Whisper Tiny, Whisper Base, Whisper Small)
- Downloaded models have appropriate indicators
- Non-downloaded models show download option

### 3. Select a different model

```
browser_click({ ref: "<whisper-base-ref>", element: "Whisper Base option" })
browser_snapshot()
```

**Verify:**
- Model selector now shows "Whisper Base"
- Mock state updated: active model changed

### 4. Verify via evaluate

```
browser_evaluate({ function: "() => window.__E2E_MOCK__.getState().currentModel" })
```

**Verify:**
- Returns the whisper-base model name

## Expected Results

- Model selector displays current model name
- Dropdown shows all available models
- Selecting a model updates the display and mock state
- Downloaded vs non-downloaded models are visually distinct
