# Research: Copy Capture to Clipboard on Save

**Branch**: `001-copy-capture-clipboard` | **Date**: 2025-12-15

## Research Questions Resolved

### 1. Clipboard Crate Selection

**Decision**: Use `arboard` crate (v3.4)

**Rationale**:
- Cross-platform support (future-proofs for macOS/Linux)
- Clean API with `ImageData` struct for image operations
- Actively maintained by 1Password team
- Handles Windows DIB format conversion automatically
- Aligns with Tauri's cross-platform philosophy

**Alternatives Considered**:
- `clipboard-win`: Windows-only, lower-level API, requires manual DIB handling
- `clipboard-rs`: Less commonly used, similar features to arboard

### 2. Integration Point in Save Flow

**Decision**: Add clipboard copy after `fs::write()` succeeds in `save_screenshot()` (lib.rs ~line 642)

**Rationale**:
- FR-006 requires clipboard copy only after successful file save
- FR-005 requires file save to succeed even if clipboard fails
- Existing error handling pattern allows graceful degradation

**Flow**:
1. File written to disk (existing logic)
2. Decode `final_image` bytes to RGBA pixel data using existing `image` crate
3. Call `arboard::Clipboard::set_image()` with ImageData
4. Handle errors via event emission (don't propagate)
5. Continue with explorer open (existing logic)

### 3. Settings System Pattern

**Decision**: Follow existing `imageFormat` pattern with tauri-plugin-store

**Rationale**:
- Consistent with existing settings architecture
- Uses proven `store.get()`/`store.set()` pattern
- Automatic persistence across restarts
- No new plugins required

**Implementation Pattern**:
- Add `clipboardCopyEnabled: bool` to `AllSettings` struct
- Create `get_clipboard_copy_enabled()` command (default: `true`)
- Create `set_clipboard_copy_enabled()` command
- Include in `get_all_settings()` response

### 4. Toast Notification Approach

**Decision**: Use existing Tauri event system with custom frontend toast

**Rationale**:
- No new plugin dependency required
- Event infrastructure already in place (`app.emit()`)
- Frontend already has notification/toast styling and `showNotification()` function
- Allows custom messaging ("Screenshot saved but clipboard copy failed")

**Implementation**:
- Backend: `app.emit("clipboard-copy-failed", { message: "..." })`
- Frontend: Listen for event, call existing `showNotification()` function
- Auto-dismiss after 3 seconds (existing timeout)

### 5. Clipboard Image Format

**Decision**: Let arboard handle format automatically (DIB on Windows)

**Rationale**:
- Arboard converts RGBA to DIB format automatically
- DIB format has maximum compatibility with Windows apps
- No manual format conversion needed
- Existing `image` crate (v0.25) handles PNG/JPEG decoding to RGBA

**Data Flow**:
- `final_image: Vec<u8>` (PNG or JPEG bytes)
- Decode with `image::load_from_memory()` to DynamicImage
- Convert to RGBA8 bytes
- Pass to arboard with width/height

## Dependencies to Add

```toml
# Cargo.toml
arboard = "3.4"
```

No other dependencies needed - `image` crate already present.

## Files to Modify

| File | Changes |
|------|---------|
| `src-tauri/Cargo.toml` | Add arboard dependency |
| `src-tauri/src/lib.rs` | Add clipboard logic, new commands, event emission |
| `src/index.html` | Add clipboard toggle to settings section |

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Clipboard locked by other app | Graceful error handling, toast notification |
| Large images (4K+) | Arboard handles; fallback is file-save success |
| Thread safety | Arboard handles Windows clipboard locking |
