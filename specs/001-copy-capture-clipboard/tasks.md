# Tasks: Copy Capture to Clipboard on Save

**Branch**: `001-copy-capture-clipboard` | **Generated**: 2025-12-15

## Overview

| Metric | Value |
|--------|-------|
| Total Tasks | 14 |
| User Story 1 (P1) Tasks | 4 |
| User Story 2 (P2) Tasks | 4 |
| User Story 3 (P3) Tasks | 3 |
| Setup Tasks | 1 |
| Foundational Tasks | 1 |
| Polish Tasks | 1 |
| Parallel Opportunities | 5 |

## Implementation Strategy

**MVP Scope**: Complete Phase 1 (Setup) + Phase 2 (Foundational) + Phase 3 (User Story 1)
- This delivers the core clipboard copy functionality
- Users can save and immediately paste screenshots
- Setting toggle (User Story 2) can be added incrementally

**Incremental Delivery**:
1. Setup → Foundational → US1 = Fully functional clipboard copy (enabled by default)
2. + US2 = User can toggle the feature on/off
3. + US3 = Copy to clipboard only button (no file save)
4. + Polish = Error notifications for edge cases

---

## Phase 1: Setup

**Goal**: Add required dependency for clipboard operations.

- [x] T001 Add arboard dependency to `src-tauri/Cargo.toml`

---

## Phase 2: Foundational

**Goal**: Create the core clipboard copy helper function that both user stories depend on.

**Blocking**: Must complete before User Story phases.

- [x] T002 Implement `copy_image_to_clipboard()` helper function in `src-tauri/src/lib.rs`

---

## Phase 3: User Story 1 - Automatic Clipboard Copy on Save (P1)

**Story Goal**: As a user taking a screenshot, I want the captured image to be automatically copied to my clipboard when I save it, so that I can immediately paste it into another application.

**Independent Test**: Capture a screenshot, save it, then immediately paste (Ctrl+V) into Paint/Word/Teams and verify the image appears correctly matching the saved file.

**Acceptance Criteria**:
- Screenshot saved to disk AND copied to clipboard
- Pasted image matches saved file exactly (resolution, timestamp)
- File save succeeds even if clipboard copy fails

### Tasks

- [x] T003 [US1] Add `clipboardCopyEnabled` field to `AllSettings` struct in `src-tauri/src/lib.rs`
- [x] T004 [US1] Extend `save_screenshot()` to call clipboard copy after successful file write in `src-tauri/src/lib.rs`
- [x] T005 [US1] Define `ClipboardErrorPayload` struct and emit `clipboard-copy-failed` event on error in `src-tauri/src/lib.rs`
- [x] T006 [P] [US1] Add listener for `clipboard-copy-failed` event and show toast notification in `src/index.html`

---

## Phase 4: User Story 2 - User Control Over Clipboard Behavior (P2)

**Story Goal**: As a user, I want to control whether screenshots are automatically copied to clipboard, so that I can choose to preserve my existing clipboard content when I only need to save to disk.

**Independent Test**: Disable the setting, save a screenshot, verify previous clipboard content remains unchanged. Re-enable, save again, verify clipboard now contains screenshot.

**Acceptance Criteria**:
- Toggle visible in settings (enabled by default)
- When disabled, clipboard not modified on save
- Setting persists across application restarts

### Tasks

- [x] T007 [US2] Implement `get_clipboard_copy_enabled()` Tauri command in `src-tauri/src/lib.rs`
- [x] T008 [US2] Implement `set_clipboard_copy_enabled()` Tauri command in `src-tauri/src/lib.rs`
- [x] T009 [P] [US2] Add clipboard toggle checkbox to settings section in `src/index.html`
- [x] T010 [P] [US2] Wire up JavaScript to load/save clipboard setting via Tauri invoke in `src/index.html`

---

## Phase 5: User Story 3 - Copy to Clipboard Only Button (P3)

**Story Goal**: As a user, I want a dedicated button to copy my screenshot to the clipboard without saving it to disk, so that I can quickly share a capture via paste without creating a file.

**Independent Test**: Capture a screenshot, click "Copy to clipboard" button, paste into an application, verify the image appears and no file was created on disk.

**Acceptance Criteria**:
- "Copy to clipboard" button visible in capture dialog
- Clicking copies image (with timestamp if enabled) to clipboard without saving
- Dialog closes on success, stays open on failure with error notification

### Tasks

- [x] T012 [US3] Implement `copy_to_clipboard_only()` Tauri command in `src-tauri/src/lib.rs`
- [x] T013 [P] [US3] Add "Copy to clipboard" button to capture dialog in `src/filename-dialog.html`
- [x] T014 [P] [US3] Wire up JavaScript to call `copy_to_clipboard_only` and handle success/failure in `src/filename-dialog.html`

---

## Phase 6: Polish & Cross-Cutting Concerns

**Goal**: Final integration verification and edge case handling.

- [x] T011 Verify clipboard copy works with large screenshots (4K+) and update error messages if needed in `src-tauri/src/lib.rs`

---

## Dependencies

```
T001 (Setup)
  └─► T002 (Foundational - clipboard helper)
        └─► T003, T004, T005 (US1 - backend)
              └─► T006 (US1 - frontend toast) [can parallel with T007-T010]
        └─► T007, T008 (US2 - backend commands)
              └─► T009, T010 (US2 - frontend toggle) [can parallel with T006]
        └─► T012 (US3 - backend command)
              └─► T013, T014 (US3 - frontend button) [can parallel]

T011 (Polish) ─► after all US tasks complete
```

## Parallel Execution Opportunities

### Within User Story 1

After T005 completes:
- T006 (frontend toast) can run in parallel with US2 backend tasks

### Within User Story 2

After T007-T008 complete:
- T009 (checkbox HTML) and T010 (JavaScript wiring) can run in parallel

### Cross-Story Parallelism

After T002 (foundational) completes:
- US1 backend tasks (T003-T005) can partially overlap with US2 backend tasks (T007-T008)
- Both modify lib.rs but different sections

## Task Details

### T001: Add arboard dependency

**File**: `src-tauri/Cargo.toml`

Add to `[dependencies]` section:
```toml
arboard = "3.4"
```

### T002: Implement clipboard helper function

**File**: `src-tauri/src/lib.rs`

Create function:
```rust
fn copy_image_to_clipboard(image_bytes: &[u8]) -> Result<(), String> {
    // 1. Decode image_bytes using image::load_from_memory()
    // 2. Convert to RGBA8 pixels
    // 3. Create arboard::ImageData { width, height, bytes }
    // 4. Call Clipboard::new()?.set_image(image_data)
    // 5. Return Ok(()) or Err with message
}
```

### T003: Add clipboardCopyEnabled to AllSettings

**File**: `src-tauri/src/lib.rs`

Extend `AllSettings` struct (~line 186):
```rust
pub struct AllSettings {
    // ... existing fields
    clipboardCopyEnabled: bool,  // Add this
}
```

Update `get_all_settings()` to include the new field with default `true`.

### T004: Extend save_screenshot with clipboard copy

**File**: `src-tauri/src/lib.rs`

After `fs::write(&full_path, &final_image)` succeeds (~line 642):
1. Load `clipboardCopyEnabled` from settings
2. If enabled, call `copy_image_to_clipboard(&final_image)`
3. Handle result (success continues, error emits event)

### T005: Define error payload and emit event

**File**: `src-tauri/src/lib.rs`

Add struct:
```rust
#[derive(Clone, Serialize)]
struct ClipboardErrorPayload {
    message: String,
}
```

In save_screenshot after clipboard error:
```rust
app.emit("clipboard-copy-failed", ClipboardErrorPayload {
    message: "Screenshot saved but clipboard copy failed".to_string(),
})?;
```

### T006: Add frontend toast listener

**File**: `src/index.html`

In JavaScript section, add event listener:
```javascript
const { listen } = window.__TAURI__.event;
listen('clipboard-copy-failed', (event) => {
    showNotification(event.payload.message, 'warning');
});
```

### T007: Implement get_clipboard_copy_enabled command

**File**: `src-tauri/src/lib.rs`

Follow `get_image_format` pattern:
```rust
#[tauri::command]
async fn get_clipboard_copy_enabled(app: AppHandle) -> Result<bool, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(store.get("clipboardCopyEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true))  // Default: enabled
}
```

Register in `.invoke_handler()`.

### T008: Implement set_clipboard_copy_enabled command

**File**: `src-tauri/src/lib.rs`

Follow `set_image_format` pattern:
```rust
#[tauri::command]
async fn set_clipboard_copy_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("clipboardCopyEnabled", serde_json::json!(enabled));
    store.save().map_err(|e| e.to_string())?;
    Ok(enabled)
}
```

Register in `.invoke_handler()`.

### T009: Add clipboard toggle to settings UI

**File**: `src/index.html`

In settings section (after image format or similar), add:
```html
<div class="settings-item">
    <label for="clipboard-copy-toggle">Copy to clipboard on save</label>
    <input type="checkbox" id="clipboard-copy-toggle" checked>
</div>
```

### T010: Wire up JavaScript for setting

**File**: `src/index.html`

Add to initialization:
```javascript
// Load setting
const clipboardEnabled = await invoke('get_clipboard_copy_enabled');
document.getElementById('clipboard-copy-toggle').checked = clipboardEnabled;

// Save on change
document.getElementById('clipboard-copy-toggle').addEventListener('change', async (e) => {
    await invoke('set_clipboard_copy_enabled', { enabled: e.target.checked });
});
```

### T011: Verify large screenshot handling

**File**: `src-tauri/src/lib.rs`

Manual testing task:
1. Capture 4K or multi-monitor screenshot
2. Verify clipboard copy succeeds or fails gracefully
3. Ensure error message is user-friendly if memory constraints hit
4. Verify file save always succeeds regardless

### T012: Implement copy_to_clipboard_only command

**File**: `src-tauri/src/lib.rs`

Create a new Tauri command that copies to clipboard without saving:
```rust
#[tauri::command]
async fn copy_to_clipboard_only(
    app: AppHandle,
    state: State<'_, AppState>,
    timestamp_options: TimestampOptions,
    image_format: String,
) -> Result<(), String> {
    // 1. Get pending screenshot from state
    // 2. Apply timestamp if enabled (using add_timestamp_to_image)
    // 3. Call copy_image_to_clipboard()
    // 4. On success: clear pending screenshot, return Ok
    // 5. On failure: return Err with message (don't clear pending)
}
```

Register in `.invoke_handler()`.

### T013: Add "Copy to clipboard" button to capture dialog

**File**: `src/filename-dialog.html`

Add a button next to the Save button:
```html
<button id="copyOnlyBtn" onclick="copyToClipboardOnly()">
    Copier dans le presse-papiers
</button>
```

Style consistently with existing buttons.

### T014: Wire up JavaScript for copy only button

**File**: `src/filename-dialog.html`

Implement the handler:
```javascript
async function copyToClipboardOnly() {
    try {
        const timestampOptions = getTimestampOptions();
        const imageFormat = document.getElementById('format').value;

        await invoke('copy_to_clipboard_only', {
            timestampOptions,
            imageFormat
        });

        // Success: close windows
        await invoke('close_window', { label: 'filename-dialog' });
        await invoke('close_window', { label: 'selection' });
    } catch (error) {
        // Failure: show notification, keep dialog open
        showNotification('Échec de la copie: ' + error, 'error');
    }
}
```
