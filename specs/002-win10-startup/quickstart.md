# Quickstart: Windows 10 Startup Compatibility Fix

**Feature**: 002-win10-startup
**Date**: 2026-01-01

## Prerequisites

- Rust 1.92+ installed
- Windows 10 or Windows 11 development machine
- Project cloned and dependencies installed

## Quick Implementation Guide

### Step 1: Add Dependency

Edit `src-tauri/Cargo.toml`:

```toml
[target.'cfg(windows)'.dependencies]
auto-launch = "0.5"
```

Run:
```bash
cd src-tauri
cargo build
```

### Step 2: Create Autostart Module

Create `src-tauri/src/autostart.rs`:

```rust
//! Windows autostart management module
//!
//! Replaces tauri-plugin-autostart with a more reliable implementation
//! using the auto-launch crate.

use auto_launch::AutoLaunchBuilder;
use std::env;

const APP_NAME: &str = "SmartPrtScr";

/// Get the current executable path
fn get_exe_path() -> Result<String, String> {
    env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to get executable path: {}", e))
}

/// Create an AutoLaunch instance for this application
fn get_auto_launch() -> Result<auto_launch::AutoLaunch, String> {
    let exe_path = get_exe_path()?;

    AutoLaunchBuilder::new()
        .set_app_name(APP_NAME)
        .set_app_path(&exe_path)
        .set_args(&["--hidden"])
        .build()
        .map_err(|e| format!("Failed to create AutoLaunch: {}", e))
}

/// Check if autostart is currently enabled
pub fn is_enabled() -> Result<bool, String> {
    let auto_launch = get_auto_launch()?;
    auto_launch.is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
}

/// Enable autostart
pub fn enable() -> Result<(), String> {
    let auto_launch = get_auto_launch()?;

    // Log the operation
    let exe_path = get_exe_path()?;
    println!("[AUTOSTART] Enabling autostart for: {}", exe_path);

    auto_launch.enable()
        .map_err(|e| format!("Failed to enable autostart: {}", e))?;

    println!("[AUTOSTART] Autostart enabled successfully");
    Ok(())
}

/// Disable autostart
pub fn disable() -> Result<(), String> {
    let auto_launch = get_auto_launch()?;

    println!("[AUTOSTART] Disabling autostart");

    auto_launch.disable()
        .map_err(|e| format!("Failed to disable autostart: {}", e))?;

    println!("[AUTOSTART] Autostart disabled successfully");
    Ok(())
}

/// Verify autostart configuration and repair if needed
/// Returns true if autostart is properly configured, false otherwise
pub fn verify_and_repair(expected_enabled: bool) -> Result<bool, String> {
    let actual_enabled = is_enabled()?;

    if expected_enabled && !actual_enabled {
        println!("[AUTOSTART] Registry entry missing, attempting repair...");
        enable()?;
        return Ok(true);
    }

    if !expected_enabled && actual_enabled {
        println!("[AUTOSTART] Unexpected registry entry, removing...");
        disable()?;
        return Ok(true);
    }

    Ok(actual_enabled == expected_enabled)
}
```

### Step 3: Update lib.rs

Modify `src-tauri/src/lib.rs`:

```rust
// Add module declaration at the top
mod autostart;

// Replace existing autostart commands:

#[tauri::command]
async fn get_auto_start() -> Result<bool, String> {
    autostart::is_enabled()
}

#[tauri::command]
async fn set_auto_start(enabled: bool) -> Result<bool, String> {
    if enabled {
        autostart::enable()?;
    } else {
        autostart::disable()?;
    }
    Ok(enabled)
}
```

### Step 4: Add Startup Verification

In the app setup (within `lib.rs` setup function):

```rust
// After settings are loaded, verify autostart
if let Some(auto_start_enabled) = settings.get("autoStart") {
    if let Some(enabled) = auto_start_enabled.as_bool() {
        match autostart::verify_and_repair(enabled) {
            Ok(true) => println!("[AUTOSTART] Configuration verified"),
            Ok(false) => println!("[AUTOSTART] Configuration repaired"),
            Err(e) => {
                println!("[AUTOSTART] Verification failed: {}", e);
                // TODO: Show notification to user
            }
        }
    }
}
```

### Step 5: Remove Old Plugin (Optional)

If removing tauri-plugin-autostart entirely:

1. Remove from `Cargo.toml`:
   ```toml
   # Remove this line
   tauri-plugin-autostart = "2"
   ```

2. Remove plugin initialization from `lib.rs`:
   ```rust
   // Remove this block
   .plugin(tauri_plugin_autostart::init(
       MacosLauncher::LaunchAgent,
       Some(vec!["--hidden"]),
   ))
   ```

3. Remove unused imports

## Testing Checklist

### Windows 10 Testing

- [ ] Enable autostart in settings
- [ ] Verify registry entry created at `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
- [ ] Reboot Windows 10
- [ ] Verify app launches automatically
- [ ] Reboot again (test persistence)
- [ ] Verify app launches automatically on second reboot
- [ ] Disable autostart in settings
- [ ] Verify registry entry removed
- [ ] Reboot and verify app does NOT launch

### Windows 11 Testing

- [ ] Repeat all Windows 10 tests on Windows 11
- [ ] Verify no regression from current working behavior

### Edge Case Testing

- [ ] Disable startup via Task Manager → Startup tab
- [ ] Launch app → check if notification appears
- [ ] Enable autostart again → verify it re-enables

## Troubleshooting

### Registry Not Being Created

1. Check Windows Event Viewer for permission errors
2. Verify the executable path is correct
3. Check if antivirus is blocking registry writes

### App Not Starting After Reboot

1. Open Registry Editor (regedit)
2. Navigate to `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
3. Check if "SmartPrtScr" entry exists
4. Verify the path in the entry is correct

### Autostart Disabled by Windows

1. Open Task Manager → Startup tab
2. Find "SmartPrtScr"
3. Check if it's disabled
4. Right-click → Enable

## Build and Test

```bash
# Build the application
cd src-tauri
cargo build

# Run in development mode
cd ..
npm run tauri dev

# Build for release
npm run tauri build
```
