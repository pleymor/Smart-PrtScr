# Implementation Plan: Timestamp on Selection Rectangle

**Branch**: `1-timestamp-overlay` | **Date**: 2024-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/1-timestamp-overlay/spec.md`

## Summary

Display timestamp preview directly on the selection rectangle during screen capture, allowing users to see exactly how their final screenshot will look. The options dialog opens simultaneously with the selection window, enabling real-time modification of timestamp options with immediate visual feedback on the selection preview.

**Technical Approach**:
1. Extend the existing canvas drawing logic in `selection.html` to render the timestamp overlay
2. Remove the preview thumbnail from the save dialog (no longer needed)
3. Open filename-dialog simultaneously with selection window
4. Use Tauri events for inter-window communication to sync timestamp options in real-time

## Technical Context

**Language/Version**: Rust 1.92 (backend), HTML/CSS/JS (frontend)
**Primary Dependencies**: Tauri 2.x, HTML5 Canvas 2D API, Tauri Events API
**Storage**: tauri-plugin-store (settings.json)
**Testing**: Manual testing (no automated test framework configured)
**Target Platform**: Windows (primary)
**Project Type**: Desktop application (Tauri)
**Performance Goals**: Real-time preview update (<16ms response to option changes)
**Constraints**: No additional dependencies; must use existing canvas/font infrastructure

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution is a template without specific rules defined. No violations to report.

## Project Structure

### Documentation (this feature)

```text
specs/1-timestamp-overlay/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── checklists/
    └── requirements.md  # Quality checklist
```

### Source Code (repository root)

```text
src/
├── index.html           # Main settings window (no changes)
├── selection.html       # Selection UI (MODIFY: timestamp preview + event listener)
└── filename-dialog.html # Save dialog (MODIFY: remove preview, emit option changes)

src-tauri/
├── src/
│   └── lib.rs           # Backend (MODIFY: open both windows simultaneously)
├── fonts/
│   └── arial.ttf        # Font file (existing)
└── Cargo.toml           # Dependencies (no changes)
```

**Structure Decision**: Single Tauri project with HTML frontend and Rust backend. Inter-window communication via Tauri events.

## Implementation Overview

### Component 1: Timestamp Preview on Selection (P1) ✅ DONE

**File**: `src/selection.html`

**Changes Implemented**:
- Added `textColors` map constant
- Added `timestampOptions` field to `appState`
- Added `loadTimestampOptions()` function
- Added `drawTimestampPreview(sel, options)` function with banner/overlay support
- Modified `drawSelection()` to call timestamp preview
- Added fullscreen capture event listener

### Component 2: Remove Preview Thumbnail (P2) ✅ DONE

**File**: `src/filename-dialog.html`

**Changes Implemented**:
- Removed preview HTML elements and CSS
- Removed `originalImage` variable and `updatePreview()` function
- Simplified `init()` function

### Component 3: Simultaneous Windows & Real-time Sync (P3) 🆕 NEW

**Files**: `src-tauri/src/lib.rs`, `src/selection.html`, `src/filename-dialog.html`

**Changes Required**:

1. **Backend: Open both windows simultaneously** (`lib.rs`)
   - Modify `open_selection_window()` to also open filename-dialog
   - Position filename-dialog appropriately (bottom-right corner)
   - Ensure selection window has focus

2. **filename-dialog: Emit option changes** (`filename-dialog.html`)
   - Add event emission on every option change
   - Use Tauri `emit()` to broadcast `timestamp-options-changed` event
   - Include full options object in event payload

3. **selection.html: Listen for option changes** (`selection.html`)
   - Add listener for `timestamp-options-changed` event
   - Update `appState.timestampOptions` when event received
   - Trigger `drawSelection()` to refresh preview

4. **Workflow changes**
   - filename-dialog no longer calls `process_selection` - selection window does
   - filename-dialog Save button saves options and emits final event
   - Selection window confirms capture via Enter/double-click

## Inter-Window Communication Flow

```
┌─────────────────────────┐     ┌─────────────────────────┐
│   filename-dialog.html  │     │    selection.html       │
│                         │     │                         │
│  User changes option    │     │                         │
│         │               │     │                         │
│         ▼               │     │                         │
│  emit('timestamp-       │────►│  listen('timestamp-     │
│   options-changed',     │     │   options-changed')     │
│   { options })          │     │         │               │
│                         │     │         ▼               │
│                         │     │  Update appState        │
│                         │     │         │               │
│                         │     │         ▼               │
│                         │     │  drawSelection()        │
│                         │     │  (shows new preview)    │
└─────────────────────────┘     └─────────────────────────┘
```

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Canvas rendering performance | Low | Medium | Use efficient canvas operations, avoid redundant redraws |
| Font rendering differences | Low | Low | Use same font family as filename-dialog (Arial) |
| Fullscreen selection UX confusion | Medium | Low | Show clear instructions, maintain existing keyboard shortcuts |
| Inter-window event latency | Low | Medium | Events are local, should be <10ms |
| Window focus issues | Medium | Medium | Ensure selection window maintains focus for keyboard shortcuts |

## Dependencies

- **Internal**: Existing `get_timestamp_options` Tauri command, Tauri events API
- **External**: None (all features use existing APIs)

## Testing Strategy

1. **Manual Testing**:
   - Test all timestamp types (banner-dark, banner-light, overlay)
   - Test all positions (top, bottom)
   - Test all text colors
   - Test font sizes at boundaries (8px, 72px)
   - Test text alignments (left, center, right)
   - Test style combinations (bold, italic, underline)
   - Test fullscreen capture flow
   - Test with timestamp disabled
   - Verify save dialog layout without preview
   - **NEW**: Test real-time option sync between windows
   - **NEW**: Test window positioning and focus behavior

2. **Edge Cases**:
   - Very small selection rectangles
   - Very large font sizes
   - Rapid selection resizing
   - Multiple consecutive captures
   - **NEW**: Rapid option changes in dialog
   - **NEW**: Closing dialog before confirming selection

## Next Steps

Run `/speckit.tasks` to generate the updated task list for Component 3 implementation.
