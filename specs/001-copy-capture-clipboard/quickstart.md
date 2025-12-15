# Quickstart: Copy Capture to Clipboard on Save

**Branch**: `001-copy-capture-clipboard` | **Date**: 2025-12-15

## Overview

Add automatic clipboard copy when saving screenshots. Users can toggle this feature on/off in settings.

## Prerequisites

- Rust toolchain installed
- Node.js for Tauri CLI
- Windows development environment

## Setup

```bash
# Checkout feature branch
git checkout 001-copy-capture-clipboard

# Install dependencies
cd src-tauri
cargo build
```

## Key Files

| File | Purpose |
|------|---------|
| `src-tauri/src/lib.rs` | Backend: clipboard commands, save logic |
| `src-tauri/Cargo.toml` | Add arboard dependency |
| `src/index.html` | Frontend: settings toggle |

## Implementation Steps

### 1. Add Dependency

```toml
# src-tauri/Cargo.toml
[dependencies]
arboard = "3.4"
```

### 2. Add Settings Commands

```rust
// lib.rs - new commands
#[tauri::command]
async fn get_clipboard_copy_enabled(app: AppHandle) -> Result<bool, String>

#[tauri::command]
async fn set_clipboard_copy_enabled(app: AppHandle, enabled: bool) -> Result<bool, String>
```

### 3. Extend Save Logic

```rust
// lib.rs - in save_screenshot() after fs::write succeeds
if clipboard_enabled {
    match copy_to_clipboard(&final_image, width, height) {
        Ok(_) => { /* success */ }
        Err(e) => {
            app.emit("clipboard-copy-failed", payload)?;
        }
    }
}
```

### 4. Add UI Toggle

```html
<!-- index.html - in settings section -->
<div class="setting-item">
    <label for="clipboard-copy">Copy to clipboard on save</label>
    <input type="checkbox" id="clipboard-copy" checked>
</div>
```

## Testing

1. Take a screenshot
2. Save it
3. Paste into Paint/Word - should show screenshot
4. Disable setting, repeat - clipboard should not change
5. Test with clipboard locked (open Word's clipboard panel)

## Commands Reference

| Command | Description |
|---------|-------------|
| `get_clipboard_copy_enabled` | Get current setting value |
| `set_clipboard_copy_enabled` | Update setting value |
| `save_screenshot` | Extended with clipboard copy logic |

## Event Reference

| Event | Direction | Purpose |
|-------|-----------|---------|
| `clipboard-copy-failed` | Backend → Frontend | Notify user of clipboard error |
