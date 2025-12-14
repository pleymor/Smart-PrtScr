# Data Model: Timestamp on Selection Rectangle

**Feature Branch**: `1-timestamp-overlay`
**Date**: 2024-12-14

## Entities

### TimestampOptions (Existing - No Changes)

Defined in `src-tauri/src/lib.rs:136-166`

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| enabled | bool | Whether timestamp is enabled | N/A |
| font_size | u32 | Font size in pixels | 8-72 |
| display_type | String | "banner-dark", "banner-light", "overlay" | Enum validation |
| text_color | String | Color name | One of: white, black, gray, red, green, blue, yellow, cyan, magenta |
| text_align | String | "left", "center", "right" | Enum validation |
| position | String | "top", "bottom" | Enum validation |
| bold | bool | Bold text style | N/A |
| italic | bool | Italic text style | N/A |
| underline | bool | Underline text style | N/A |

### SelectionState (Existing - Extended)

Defined in `src/selection.html:112-127`

| Field | Type | Description | New? |
|-------|------|-------------|------|
| current | SelectionState | State machine state | No |
| selection | Object | {x, y, width, height} | No |
| activeHandle | String | Active resize handle | No |
| dragStart | Object | Drag start coordinates | No |
| drawStart | Object | Draw start coordinates | No |
| **timestampOptions** | Object | Cached timestamp settings | **Yes** |

### Text Colors Map (Frontend)

To be added in `src/selection.html` (copied from `filename-dialog.html:482-493`)

| Color Name | Hex Value |
|------------|-----------|
| white | #ffffff |
| black | #000000 |
| gray | #808080 |
| red | #ff0000 |
| green | #00ff00 |
| blue | #0000ff |
| yellow | #ffff00 |
| cyan | #00ffff |
| magenta | #ff00ff |

## State Transitions

### Selection State Machine (Existing - Extended for Fullscreen)

```
IDLE ──────────────────────────────────────────────┐
  │                                                │
  │ [mousedown]                                    │ [fullscreen trigger]
  ▼                                                ▼
DRAWING ─────────────────────────────────────── MODIFIABLE (fullscreen)
  │                                                │
  │ [mouseup, valid size]                          │
  ▼                                                │
MODIFIABLE ◄───────────────────────────────────────┘
  │
  ├─[handle mousedown]──► RESIZING ──[mouseup]──► MODIFIABLE
  │
  └─[inside mousedown]──► MOVING ───[mouseup]──► MODIFIABLE
```

**New behavior**: When fullscreen capture is triggered, the state transitions directly to MODIFIABLE with a selection covering the entire screen.

## Data Flow

### Timestamp Preview Rendering Flow

```
1. Selection window opens
   ↓
2. Frontend invokes 'get_timestamp_options'
   ↓
3. Options cached in appState.timestampOptions
   ↓
4. On each drawSelection() call:
   ├─ If timestampOptions.enabled:
   │    └─ Call drawTimestampPreview(selection, options)
   └─ Else: Skip timestamp rendering
```

### Fullscreen Capture Flow (Modified)

```
1. Hotkey pressed (Win+Shift+PrintScreen or PrintScreen)
   ↓
2. Selection window opens
   ↓
3. Timestamp options loaded
   ↓
4. Fullscreen selection rectangle created
   ↓
5. Timestamp preview rendered on fullscreen selection
   ↓
6. User confirms (Enter/double-click)
   ↓
7. process_selection with full screen bounds
```

## File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `src/selection.html` | Modified | Add timestamp preview rendering |
| `src/filename-dialog.html` | Modified | Remove preview canvas |
| `src-tauri/src/lib.rs` | No change | Backend already complete |
