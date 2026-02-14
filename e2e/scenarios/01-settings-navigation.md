# Scenario: Settings Navigation

Verify that the sidebar renders all expected sections and clicking each one switches the content area.

## Preconditions

Default mock state (returning user, models downloaded).

## Steps

### 1. Navigate to app

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
```

### 2. Take accessibility snapshot

```
browser_snapshot()
```

**Verify:**
- Sidebar shows sections: General, Advanced, Batch Import, Vocabulary, History, About
- General section is active/selected by default
- Main content shows General settings

### 3. Click "History" in sidebar

```
browser_click({ ref: "<history-ref>", element: "History sidebar item" })
browser_snapshot()
```

**Verify:**
- History section content is displayed
- History sidebar item appears active/selected

### 4. Click "Advanced" in sidebar

```
browser_click({ ref: "<advanced-ref>", element: "Advanced sidebar item" })
browser_snapshot()
```

**Verify:**
- Advanced settings content is displayed
- Advanced sidebar item appears active/selected

### 5. Click "About" in sidebar

```
browser_click({ ref: "<about-ref>", element: "About sidebar item" })
browser_snapshot()
```

**Verify:**
- About section content is displayed
- Version info shown (v0.8.0)

### 6. Click "Batch Import" in sidebar

```
browser_click({ ref: "<batch-ref>", element: "Batch Import sidebar item" })
browser_snapshot()
```

**Verify:**
- Batch Import content is displayed

### 7. Click "Vocabulary" in sidebar

```
browser_click({ ref: "<vocab-ref>", element: "Vocabulary sidebar item" })
browser_snapshot()
```

**Verify:**
- Vocabulary content is displayed

## Expected Results

- All 6 sidebar sections render and are clickable
- Clicking a section updates the main content area
- Only one section is active at a time
- No console errors during navigation
