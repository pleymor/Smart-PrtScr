# Implementation Plan: Windows 10 Startup Compatibility Fix

**Branch**: `002-win10-startup` | **Date**: 2026-01-01 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-win10-startup/spec.md`

## Summary

Fix the Windows 10 auto-start functionality by replacing the buggy `tauri-plugin-autostart` with a custom implementation using the `auto-launch` crate. The solution includes proper error handling, diagnostic logging, and non-blocking user notifications when startup configuration fails.

## Technical Context

**Language/Version**: Rust 1.92+ (Edition 2021)
**Primary Dependencies**: Tauri 2.x, auto-launch 0.5.0, winreg 0.55
**Storage**: Windows Registry (HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run)
**Testing**: Manual testing on Windows 10/11 VMs (no existing test framework)
**Target Platform**: Windows 10 build 1809+, Windows 11
**Project Type**: Single desktop application (Tauri)
**Performance Goals**: Startup within 30 seconds of Windows login
**Constraints**: No admin privileges required, NSIS installer only
**Scale/Scope**: Single-user desktop application

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution is not yet customized. Proceeding with standard best practices:

- [x] Changes are minimal and focused on the bug fix
- [x] No unnecessary abstractions introduced
- [x] Existing code patterns followed
- [x] Error handling added for new functionality
- [x] Logging added for diagnostic purposes

## Project Structure

### Documentation (this feature)

```text
specs/002-win10-startup/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 research findings
├── data-model.md        # Phase 1: Data entities
├── quickstart.md        # Phase 1: Implementation guide
├── contracts/           # Phase 1: API contracts (N/A - internal feature)
├── checklists/
│   └── requirements.md  # Spec validation checklist
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── lib.rs           # Main application logic (modify autostart functions)
│   ├── autostart.rs     # NEW: Dedicated autostart module
│   └── main.rs          # Entry point (no changes)
└── Cargo.toml           # Add auto-launch dependency
```

**Structure Decision**: Minimal changes to existing structure. New `autostart.rs` module encapsulates all autostart logic, keeping `lib.rs` clean. No new directories required.

## Complexity Tracking

No constitution violations. The implementation is straightforward:

| Aspect | Complexity | Justification |
|--------|------------|---------------|
| New module | Low | Single-purpose module for autostart |
| New dependency | Low | Well-maintained crate (auto-launch) |
| Registry access | Low | Project already uses winreg |

## Implementation Overview

### Phase 1: Core Fix

1. **Add auto-launch dependency** to Cargo.toml
2. **Create autostart.rs module** with:
   - `enable_autostart()` - Enable auto-start via registry
   - `disable_autostart()` - Disable auto-start via registry
   - `is_autostart_enabled()` - Check current state
   - `verify_autostart()` - Verify registry entry exists and is valid
3. **Update lib.rs** to use new module instead of tauri-plugin-autostart
4. **Add logging** for all autostart operations

### Phase 2: Error Handling & Notifications

1. **Implement notification system** for autostart failures
2. **Add startup verification** - check on app launch if autostart is configured but not working
3. **Add detailed error logging** with timestamps and error codes

### Phase 3: Testing & Validation

1. **Test on Windows 10 VMs** (builds 1809, 21H2, 22H2)
2. **Test on Windows 11 VMs** (builds 22H2, 23H2)
3. **Verify persistence** across multiple reboots
4. **Test edge cases** (Task Manager disable, permissions)

## Dependencies

### To Add

```toml
# Cargo.toml
[target.'cfg(windows)'.dependencies]
auto-launch = "0.5"
```

### To Remove/Deprecate

```toml
# Remove from Cargo.toml (or keep as fallback)
tauri-plugin-autostart = "2"  # Buggy on Windows
```

### To Keep (already present)

```toml
winreg = "0.55"  # For fallback registry operations
tauri-plugin-log = "2"  # For logging
```

## API Changes

### Tauri Commands (existing, to modify)

```rust
// Current implementation (buggy)
#[tauri::command]
async fn get_auto_start(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

// New implementation
#[tauri::command]
async fn get_auto_start() -> Result<bool, String> {
    autostart::is_enabled().map_err(|e| e.to_string())
}
```

### Frontend API (no changes required)

The existing JavaScript calls to `invoke('get_auto_start')` and `invoke('set_auto_start')` will continue to work with the same interface.

## Success Criteria Mapping

| Requirement | Implementation |
|-------------|----------------|
| FR-001: Auto-start on Windows 10 | auto-launch crate with proper registry handling |
| FR-002: Auto-start on Windows 11 | Same implementation, verified on Windows 11 |
| FR-007: Graceful failure handling | try/catch in Rust with Result types |
| FR-008: Log startup errors | tauri-plugin-log integration |
| FR-009: Non-blocking notification | Tauri notification or dialog API |
| SC-003: Under 30 seconds | No changes to startup time |
| SC-005: Zero crashes | Comprehensive error handling |
