# Implementation Plan: Copy Capture to Clipboard on Save

**Branch**: `001-copy-capture-clipboard` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-copy-capture-clipboard/spec.md`

## Summary

Automatically copy screenshots to the system clipboard when saving, allowing users to immediately paste into other applications. Includes a user-configurable toggle (enabled by default) and graceful error handling with toast notifications for clipboard failures.

## Technical Context

**Language/Version**: Rust 1.92 (backend), JavaScript ES6+ (frontend)
**Primary Dependencies**: Tauri 2.0, arboard 3.4 (new), image 0.25 (existing)
**Storage**: tauri-plugin-store (settings.json) for clipboard toggle preference
**Testing**: Manual testing (no automated test framework in project)
**Target Platform**: Windows 10/11 desktop
**Project Type**: Desktop application (Tauri hybrid)
**Performance Goals**: Clipboard copy completes within 500ms of file save
**Constraints**: File save must succeed even if clipboard fails; clipboard operations must not block UI
**Scale/Scope**: Single-user desktop app, one screenshot at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution is a template with no specific gates defined. Proceeding with standard best practices:
- [x] Feature is self-contained within existing architecture
- [x] Single new dependency required (arboard) - justified for clipboard operations
- [x] Changes follow existing patterns (settings, commands, events)
- [x] No breaking changes to existing functionality
- [x] Error handling follows FR-005 (save succeeds regardless of clipboard)

## Project Structure

### Documentation (this feature)

```text
specs/001-copy-capture-clipboard/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output - technology decisions
├── data-model.md        # Phase 1 output - entity definitions
├── quickstart.md        # Phase 1 output - implementation guide
├── checklists/
│   └── requirements.md  # Quality validation checklist
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   └── lib.rs           # Add clipboard commands and save logic
└── Cargo.toml           # Add arboard dependency

src/
└── index.html           # Add clipboard toggle to settings UI
```

**Structure Decision**: This is a Tauri desktop application with Rust backend and HTML/JS/CSS frontend. Changes required in both backend (Rust: clipboard logic, settings commands) and frontend (JS/HTML: settings toggle, error toast listener).

## Implementation Approach

### Backend Changes (src-tauri/src/lib.rs)

#### 1. Add Dependency

```toml
# Cargo.toml
[dependencies]
arboard = "3.4"
```

#### 2. New Settings Commands

Follow existing pattern from `get_image_format`/`set_image_format`:

- `get_clipboard_copy_enabled()` - Returns bool, default `true`
- `set_clipboard_copy_enabled(enabled: bool)` - Persists to settings.json
- Update `get_all_settings()` to include `clipboardCopyEnabled`
- Update `AllSettings` struct with new field

#### 3. Clipboard Copy Function

New helper function:

```rust
fn copy_image_to_clipboard(image_bytes: &[u8]) -> Result<(), String> {
    // 1. Decode image bytes to RGBA pixels using image crate
    // 2. Create arboard::ImageData with width, height, rgba
    // 3. Set clipboard image
    // 4. Return success or error
}
```

#### 4. Extend save_screenshot() Command

After successful `fs::write()` (around line 642):

1. Check if `clipboardCopyEnabled` is true (from settings or SaveData)
2. Call `copy_image_to_clipboard(&final_image)`
3. On success: continue to explorer open
4. On failure: emit `clipboard-copy-failed` event with message, continue to explorer

#### 5. Error Event

Define event payload and emit on clipboard failure:

```rust
#[derive(Serialize)]
struct ClipboardErrorPayload {
    message: String,
}

// In save_screenshot after clipboard error
app.emit("clipboard-copy-failed", ClipboardErrorPayload {
    message: "Screenshot saved but clipboard copy failed".to_string()
})?;
```

### Frontend Changes (src/index.html)

#### 1. Settings Toggle

Add checkbox in the settings section (follow existing toggle patterns):

```html
<div class="settings-item">
    <label>Copy to clipboard on save</label>
    <input type="checkbox" id="clipboard-copy-toggle" />
</div>
```

#### 2. JavaScript Integration

- Load setting on page load via `invoke('get_clipboard_copy_enabled')`
- Save on toggle change via `invoke('set_clipboard_copy_enabled', { enabled })`
- Listen for `clipboard-copy-failed` event
- Call existing `showNotification()` function for toast display

### Interaction Flow

```
User clicks Save
    ↓
Frontend calls save_screenshot with options
    ↓
Backend: save_screenshot()
    ├─► Write file to disk
    │       ↓
    │   Check clipboardCopyEnabled setting
    │       ↓ (if true)
    │   Decode image to RGBA pixels
    │       ↓
    │   arboard.set_image(ImageData)
    │       ├─► Success: continue
    │       └─► Error: emit "clipboard-copy-failed" event
    ↓
Backend: Open explorer to show file
    ↓
Frontend: Listen for clipboard-copy-failed
    ↓ (if received)
Show toast: "Screenshot saved but clipboard copy failed"
```

## Complexity Tracking

No constitution violations. This is a straightforward feature addition:
- One new dependency (arboard) - necessary for clipboard operations
- Follows existing patterns for settings and commands
- Graceful error handling per requirements

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Clipboard locked | Try once, emit error event, don't retry |
| Large images | Let arboard handle; if fails, user still has file |
| Slow clipboard | Async operation, doesn't block UI or file save |

## Artifacts Generated

- [x] research.md - Technology decisions
- [x] data-model.md - Entity definitions
- [x] quickstart.md - Implementation guide
- [ ] tasks.md - Created by `/speckit.tasks`
