# Data Model: Windows 10 Startup Compatibility

**Feature**: 002-win10-startup
**Date**: 2026-01-01

## Overview

This feature has minimal data model requirements as it primarily involves system-level configuration (Windows Registry) rather than application data storage.

## Entities

### 1. AutostartConfiguration

Represents the autostart state for the application.

| Field | Type | Description |
|-------|------|-------------|
| enabled | bool | Whether autostart is enabled |
| app_name | String | Application name for registry entry |
| app_path | String | Full path to executable |
| args | Vec<String> | Launch arguments (e.g., "--hidden") |

**Storage**: Windows Registry
- Primary: `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Run`
- Secondary: `HKEY_CURRENT_USER\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run`

**Lifecycle**:
- Created: When user enables autostart in settings
- Updated: When user toggles autostart setting
- Deleted: When user disables autostart

### 2. AutostartError

Represents errors that occur during autostart operations.

| Field | Type | Description |
|-------|------|-------------|
| operation | String | "enable", "disable", "verify" |
| error_code | Option<u32> | Windows error code if applicable |
| message | String | Human-readable error message |
| timestamp | DateTime | When the error occurred |

**Storage**: Application log file (via tauri-plugin-log)

**Lifecycle**:
- Created: When an autostart operation fails
- Persisted: Written to log file immediately
- Displayed: Shown to user via notification (non-blocking)

### 3. StartupVerificationResult

Result of checking autostart status on application launch.

| Field | Type | Description |
|-------|------|-------------|
| expected_enabled | bool | What the user setting says |
| actual_enabled | bool | What the registry says |
| registry_entry_valid | bool | Whether the registry path is correct |
| needs_repair | bool | Whether autostart needs to be re-enabled |

**Storage**: Transient (in-memory only)

**Lifecycle**:
- Created: On each application launch
- Used: To trigger repair or notification if mismatch detected
- Discarded: After startup verification completes

## State Transitions

### AutostartConfiguration States

```
                    ┌─────────────────┐
                    │    Disabled     │
                    │ (no registry    │
                    │  entry exists)  │
                    └────────┬────────┘
                             │
                   User enables autostart
                             │
                             ▼
                    ┌─────────────────┐
                    │    Enabled      │
                    │ (registry entry │◄──── Repair (if removed)
                    │  exists)        │
                    └────────┬────────┘
                             │
                   User disables autostart
                             │
                             ▼
                    ┌─────────────────┐
                    │    Disabled     │
                    └─────────────────┘
```

### Startup Verification Flow

```
App Launch
    │
    ▼
Check user setting (store)
    │
    ▼
Check registry entry
    │
    ├── Match? ──────► Continue normally
    │
    └── Mismatch?
           │
           ▼
    ┌──────────────────┐
    │ Setting: enabled │
    │ Registry: missing│
    └────────┬─────────┘
             │
             ▼
    Attempt repair (re-add registry)
             │
             ├── Success ──► Log info, continue
             │
             └── Failure ──► Log error, notify user
```

## Validation Rules

### AutostartConfiguration

1. `app_path` must be an absolute path
2. `app_path` must point to an existing executable
3. `app_name` must not be empty
4. `app_name` should match the registry key name format (alphanumeric, no special chars)

### Registry Entry

1. Value name must match `app_name`
2. Value data must be a valid path (optionally with arguments)
3. Path must use backslashes (Windows format)
4. Arguments must be properly quoted if they contain spaces

## Relationships

```
┌─────────────────────────────────────────────────┐
│                 Application                      │
│                                                 │
│  ┌─────────────┐    reads     ┌──────────────┐ │
│  │ User        │◄────────────►│ Settings     │ │
│  │ Settings UI │    writes    │ Store        │ │
│  └──────┬──────┘              └──────┬───────┘ │
│         │                            │         │
│         │ invokes                    │ syncs   │
│         ▼                            ▼         │
│  ┌─────────────────────────────────────────┐   │
│  │         Autostart Module                │   │
│  │  ┌─────────────┐    ┌────────────────┐  │   │
│  │  │ auto-launch │    │ Verification   │  │   │
│  │  │ crate       │    │ Logic          │  │   │
│  │  └──────┬──────┘    └───────┬────────┘  │   │
│  │         │                   │           │   │
│  └─────────┼───────────────────┼───────────┘   │
│            │                   │               │
└────────────┼───────────────────┼───────────────┘
             │                   │
             ▼                   ▼
    ┌─────────────────────────────────────┐
    │         Windows Registry             │
    │  HKCU\...\Run\SmartPrtScr           │
    └─────────────────────────────────────┘
```

## Data Volume

- **AutostartConfiguration**: 1 entry per installation
- **Registry entries**: 1-2 per installation (Run + StartupApproved)
- **Log entries**: Minimal (only on errors or significant events)
- **Storage impact**: Negligible (<1KB)
