# Scenario: History Viewer

Verify the transcription history section displays entries and supports interactions.

## Preconditions

Default mock state. Two mock history entries are pre-populated.

## Steps

### 1. Navigate and go to History

```
browser_navigate({ url: "http://localhost:1420" })
browser_wait_for({ text: "General" })
browser_click({ ref: "<history-ref>", element: "History sidebar item" })
browser_snapshot()
```

**Verify:**
- History section is displayed
- Two transcription entries are visible

### 2. Verify entry content

```
browser_snapshot()
```

**Verify:**
- Entry 1 shows transcription text (e.g., "This is a test transcription...")
- Entry 2 shows transcription text (e.g., "Another transcription entry...")
- Each entry shows:
  - Transcription text
  - Timestamp or date
  - Action buttons (copy, delete)

### 3. Click copy button on first entry

```
browser_click({ ref: "<copy-btn-ref>", element: "Copy button for first entry" })
```

**Verify:**
- Copy action triggers (clipboard mock updated)
- Visual feedback (button state change or toast notification)

### 4. Click delete button on an entry

```
browser_click({ ref: "<delete-btn-ref>", element: "Delete button for entry" })
browser_snapshot()
```

**Verify:**
- Entry is removed from the list
- Remaining entries still display correctly

### 5. Verify empty state (after deleting all)

Delete the remaining entry.

```
browser_click({ ref: "<delete-btn-ref>", element: "Delete button for remaining entry" })
browser_snapshot()
```

**Verify:**
- Empty state message is shown (e.g., "No transcriptions yet")

## Expected Results

- History section lists all mock entries
- Each entry displays text, timestamp, and action buttons
- Copy button triggers clipboard write
- Delete button removes entry from list
- Empty state renders when all entries are deleted
