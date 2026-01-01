# Research: Windows 10 Startup Compatibility

**Feature**: 002-win10-startup
**Date**: 2026-01-01

## Executive Summary

The Windows 10 startup issue is caused by a known bug in `tauri-plugin-autostart` where the Windows registry entry is removed after the first boot. The recommended fix is to either upgrade to the latest plugin version with proper error handling, or implement a custom solution using the `auto-launch` crate directly.

## Research Findings

### 1. Root Cause Analysis

**Decision**: The issue is a known bug in tauri-plugin-autostart on Windows

**Rationale**:
- GitHub Issue #771 documents that the registry entry in `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` is mysteriously removed after the app launches
- GitHub Issue #24 confirms autostart works on macOS but fails on Windows
- The plugin's `MacosLauncher` parameter is ignored on non-macOS platforms
- Windows 10 and Windows 11 may have different behaviors due to registry/UAC differences

**Alternatives considered**:
- WebView2 runtime issues: Ruled out - the app runs fine when manually launched
- Permission issues: Partially ruled out - the registry entry IS created, just gets removed
- MSIX vs NSIS differences: Not the primary cause since we're only targeting NSIS

**Sources**:
- [Issue #771: autostart on Windows is removed after one boot](https://github.com/tauri-apps/plugins-workspace/issues/771)
- [Issue #24: Auto Start not working on Windows platform](https://github.com/tauri-apps/plugins-workspace/issues/24)

### 2. Solution Approach

**Decision**: Replace tauri-plugin-autostart with custom implementation using `auto-launch` crate

**Rationale**:
- The `auto-launch` crate (v0.5.0) handles both registry locations:
  - `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
  - `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`
- It can detect if startup was disabled via Task Manager and re-enable it
- Direct control over registry operations allows better error handling and logging
- The project already uses `winreg 0.55` for registry operations

**Alternatives considered**:
- Keep tauri-plugin-autostart and add workaround: Rejected because the plugin has an unresolved bug and workarounds are unreliable
- Use WiX template to set registry during installation: Rejected because users need runtime control over the setting

**Sources**:
- [auto-launch crate documentation](https://docs.rs/auto-launch)
- [auto-launch on crates.io](https://crates.io/crates/auto-launch)

### 3. Implementation Strategy

**Decision**: Implement a hybrid approach with fallback mechanism

**Rationale**:
- Primary: Use `auto-launch` crate for cross-platform compatibility
- Fallback: Direct registry manipulation if auto-launch fails
- Detection: Check both registry locations on startup to verify auto-start is properly configured
- Logging: Add comprehensive logging for all autostart operations
- Notification: Use Tauri's notification API for non-blocking user feedback

**Implementation steps**:
1. Add `auto-launch` dependency to Cargo.toml
2. Create new autostart module with enable/disable/check functions
3. Add startup verification on app launch
4. Implement non-blocking notification for failures
5. Add debug logging for all autostart operations
6. Remove or deprecate tauri-plugin-autostart

### 4. Testing Strategy

**Decision**: Manual testing on Windows 10 and Windows 11 VMs

**Rationale**:
- The project has no existing test infrastructure
- Autostart functionality requires system-level testing (reboot cycles)
- Automated testing of registry changes and startup behavior is complex
- Focus on creating a testing checklist for manual verification

**Test scenarios**:
1. Enable autostart → reboot → verify app launches
2. Disable autostart → reboot → verify app does NOT launch
3. Enable autostart → disable via Task Manager → reboot → verify behavior
4. Multiple reboots to verify registry persistence
5. Test on Windows 10 builds: 1809, 21H2, 22H2
6. Test on Windows 11 builds: 22H2, 23H2

### 5. Notification Implementation

**Decision**: Use Windows toast notifications via existing Tauri capabilities

**Rationale**:
- Tauri v2 supports native notifications
- Non-blocking as required by FR-009
- Consistent with Windows UX patterns
- Can include actionable information (e.g., "Open Settings")

**Dependencies**:
- May need to add `tauri-plugin-notification` if not already available
- Alternative: Use Tauri's dialog plugin for simpler implementation

## Technical Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| auto-launch | 0.5.0 | Cross-platform autostart management |
| winreg | 0.55 (existing) | Direct registry access for fallback |
| tauri-plugin-notification | 2.x | User notifications (if not present) |

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| auto-launch has same bug | Low | High | Fallback to direct registry manipulation |
| Windows Defender flags registry changes | Medium | Medium | Sign the application, document for users |
| UAC blocks registry writes | Low | Medium | Use HKCU (no admin required) |
| Task Manager disables startup | Medium | Low | Detect and notify user |

## Resolved Clarifications

All technical clarifications have been resolved through research. No additional user input required.
