# Scenario: Onboarding Flow

Verify the first-run onboarding experience when no models are downloaded.

## Preconditions

Must modify `mock-state.ts` defaults OR use mock API before app initialization. The default state has `hasAnyModels: true` which shows the main settings UI. To trigger onboarding, the state must have `hasAnyModels: false` before the app's `checkOnboardingStatus()` runs.

**Option A: Modify mock-state.ts temporarily**
Change `hasAnyModels: true` to `hasAnyModels: false` in `e2e/browser-mocks/mock-state.ts`.

**Option B: Race condition approach**
Use `browser_evaluate` immediately after navigate (may not work reliably since the app initializes quickly).

**Option C (Recommended): Pre-configure via mock-state.ts**
Edit `mock-state.ts` to set `hasAnyModels: false`, reload the page, test onboarding, then restore.

## Steps

### 1. Configure mock state for new user

Edit `e2e/browser-mocks/mock-state.ts`:
- Set `hasAnyModels: false`
- Set all model `is_downloaded: false`

### 2. Navigate to app

```
browser_navigate({ url: "http://localhost:1420" })
browser_snapshot()
```

**Verify:**
- Onboarding screen is displayed (not the main settings UI)
- Welcome message or model selection prompt visible

### 3. Verify onboarding content

```
browser_snapshot()
```

**Verify:**
- Model cards are displayed for available models
- Each model shows name, description, size
- A "Download" or "Get Started" button is present

### 4. Click on a model card to select/download

```
browser_click({ ref: "<model-card-ref>", element: "Whisper Tiny model card" })
browser_snapshot()
```

**Verify:**
- Model selection is acknowledged
- Progress indicator or next step appears

### 5. Restore default state

Edit `e2e/browser-mocks/mock-state.ts` back to `hasAnyModels: true`.

```
browser_navigate({ url: "http://localhost:1420" })
browser_snapshot()
```

**Verify:**
- Main settings UI is displayed (onboarding skipped)

## Expected Results

- With no models: onboarding flow renders
- With models: main settings UI renders
- Model cards display correctly in onboarding
- Onboarding provides clear path to download first model

## Notes

The onboarding check in `App.tsx` calls `commands.hasAnyModelsAvailable()` on mount. The mock returns `mockState.hasAnyModels`. Since mock state resets on page reload (module re-initialization), changing state via `window.__E2E_MOCK__.update()` then reloading won't persist. The mock-state.ts defaults must be changed directly.
