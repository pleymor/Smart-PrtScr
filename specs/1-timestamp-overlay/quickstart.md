# Quickstart: Timestamp on Selection Rectangle

**Feature Branch**: `1-timestamp-overlay`
**Date**: 2024-12-14

## Prerequisites

- Node.js (for Tauri CLI)
- Rust 1.92+ (as specified in Cargo.toml)
- Windows OS (application target platform)

## Setup

```bash
# Clone and checkout feature branch
git checkout 1-timestamp-overlay

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

## Testing the Feature

### Manual Test: Timestamp Preview on Selection

1. Launch the application (`npm run tauri:dev`)
2. Open main window from system tray
3. Enable timestamp in "Horodatage" section
4. Configure timestamp options (position, color, font size, etc.)
5. Press `PrintScreen` or `Win+Shift+PrintScreen`
6. Draw a selection rectangle
7. **Verify**: Timestamp preview appears on the selection rectangle
8. Resize/move the selection
9. **Verify**: Timestamp preview updates in real-time
10. Press `Enter` to confirm capture
11. **Verify**: Save dialog opens without preview thumbnail

### Manual Test: Fullscreen Capture with Timestamp

1. Configure timestamp options in main window
2. Press `Win+Shift+PrintScreen` for fullscreen capture
3. **Verify**: Selection window shows fullscreen rectangle with timestamp preview
4. Press `Enter` to confirm
5. **Verify**: Save dialog shows correct filename, no preview thumbnail

### Manual Test: Timestamp Disabled

1. Disable timestamp in main window settings
2. Trigger capture
3. **Verify**: Selection rectangle shows without timestamp overlay
4. Complete capture
5. **Verify**: Saved image has no timestamp

## Key Files

| File | Purpose |
|------|---------|
| `src/selection.html` | Selection UI with timestamp preview |
| `src/filename-dialog.html` | Save dialog (without preview) |
| `src/index.html` | Main settings window |
| `src-tauri/src/lib.rs` | Backend Rust code |

## Build

```bash
# Production build
npm run tauri:build
```

Output: `src-tauri/target/release/smart-prtscr.exe`
