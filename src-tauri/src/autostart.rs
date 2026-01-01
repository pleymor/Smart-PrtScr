//! Windows autostart management module
//!
//! Replaces tauri-plugin-autostart with a more reliable implementation
//! using the auto-launch crate. This fixes the Windows 10 startup compatibility
//! issue where the registry entry was being removed after the first boot.

#[cfg(target_os = "windows")]
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
#[cfg(target_os = "windows")]
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
#[cfg(target_os = "windows")]
pub fn is_enabled() -> Result<bool, String> {
    let auto_launch = get_auto_launch()?;
    auto_launch
        .is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
}

#[cfg(not(target_os = "windows"))]
pub fn is_enabled() -> Result<bool, String> {
    Ok(false)
}

/// Enable autostart
#[cfg(target_os = "windows")]
pub fn enable() -> Result<(), String> {
    let auto_launch = get_auto_launch()?;

    // Log the operation
    let exe_path = get_exe_path()?;
    println!("[AUTOSTART] Enabling autostart for: {}", exe_path);

    auto_launch
        .enable()
        .map_err(|e| format!("[AUTOSTART] Failed to enable autostart: {}", e))?;

    println!("[AUTOSTART] Autostart enabled successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn enable() -> Result<(), String> {
    println!("[AUTOSTART] Autostart not supported on this platform");
    Ok(())
}

/// Disable autostart
#[cfg(target_os = "windows")]
pub fn disable() -> Result<(), String> {
    let auto_launch = get_auto_launch()?;

    println!("[AUTOSTART] Disabling autostart");

    auto_launch
        .disable()
        .map_err(|e| format!("[AUTOSTART] Failed to disable autostart: {}", e))?;

    println!("[AUTOSTART] Autostart disabled successfully");
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn disable() -> Result<(), String> {
    println!("[AUTOSTART] Autostart not supported on this platform");
    Ok(())
}

/// Verify autostart configuration and repair if needed
/// Returns Ok(true) if configuration matches expected state (no repair needed)
/// Returns Ok(false) if repair was performed
/// Returns Err if verification or repair failed
pub fn verify_and_repair(expected_enabled: bool) -> Result<bool, String> {
    let actual_enabled = is_enabled()?;

    if expected_enabled && !actual_enabled {
        println!("[AUTOSTART] Registry entry missing, attempting repair...");
        enable()?;
        println!("[AUTOSTART] Repair successful - autostart re-enabled");
        return Ok(false); // Repair was performed
    }

    if !expected_enabled && actual_enabled {
        println!("[AUTOSTART] Unexpected registry entry found, removing...");
        disable()?;
        println!("[AUTOSTART] Repair successful - autostart disabled");
        return Ok(false); // Repair was performed
    }

    println!("[AUTOSTART] Configuration verified - no repair needed");
    Ok(true) // No repair needed
}
