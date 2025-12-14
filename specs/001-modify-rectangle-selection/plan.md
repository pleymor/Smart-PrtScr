# Implementation Plan: Modify Rectangle After Selection

**Branch**: `001-modify-rectangle-selection` | **Date**: 2024-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-modify-rectangle-selection/spec.md`

## Summary

Enable users to modify (resize/move) the screenshot selection rectangle after initial drawing, with visual handles for resizing from corners/edges, drag-to-move from center, and keyboard shortcuts for confirm (Enter) or cancel (Escape). Implementation will enhance the existing `selection.html` frontend with a state machine for interaction modes and add visual handle elements.

## Technical Context

**Language/Version**: Rust 1.92 (backend), JavaScript ES6+ (frontend)
**Primary Dependencies**: Tauri 2.0, HTML5 Canvas API
**Storage**: N/A (transient selection state only)
**Testing**: Manual testing (no automated test framework in project)
**Target Platform**: Windows 10/11 desktop
**Project Type**: Desktop application (Tauri hybrid)
**Performance Goals**: 60 fps during selection manipulation, <16ms per frame
**Constraints**: Selection must remain responsive during resize/move; minimal CPU usage while idle
**Scale/Scope**: Single-user desktop app, one selection window at a time

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

The project constitution is a template with no specific gates defined. Proceeding with standard best practices:
- [x] Feature is self-contained within existing architecture
- [x] No new dependencies required
- [x] Changes are limited to frontend selection logic
- [x] No breaking changes to existing functionality

## Project Structure

### Documentation (this feature)

```text
specs/001-modify-rectangle-selection/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── src/
│   ├── lib.rs           # Backend commands (no changes needed)
│   └── main.rs          # Entry point (no changes needed)
└── Cargo.toml           # Dependencies (no changes needed)

src/
├── selection.html       # Main file to modify - add handles, state machine
├── index.html           # Main window (no changes needed)
└── filename-dialog.html # Save dialog (no changes needed)
```

**Structure Decision**: This is a Tauri desktop application with Rust backend and HTML/JS/CSS frontend. The feature implementation is entirely in the frontend (`src/selection.html`) as the selection logic is client-side JavaScript using HTML5 Canvas.

## Implementation Approach

### Core Changes to `src/selection.html`

1. **Selection State Machine**
   - States: `idle` → `drawing` → `modifiable` → `resizing`/`moving` → `modifiable`
   - After initial drawing completes, transition to `modifiable` instead of immediate capture

2. **Handle System**
   - 8 handles: 4 corners (NW, NE, SW, SE) + 4 edges (N, S, E, W)
   - Handle size: 10x10 pixels with larger hit area (20x20 pixels)
   - Visual: white fill with dark border for visibility on any background

3. **Cursor Management**
   - Corners: `nwse-resize`, `nesw-resize`
   - Edges: `ns-resize`, `ew-resize`
   - Interior: `move`
   - Outside: `crosshair`

4. **Interaction Logic**
   - Resize: Drag handle, anchor opposite corner/edge
   - Move: Drag from interior, constrain to screen bounds
   - New selection: Click outside current selection
   - Confirm: Enter key or double-click inside
   - Cancel: Escape key

### Constraints Implementation

- Minimum size: 10x10 pixels
- Screen bounds: Selection cannot extend beyond captured screen area
- No negative dimensions: Handle swap logic when dragging past opposite edge

## Complexity Tracking

No constitution violations - this is a straightforward frontend enhancement.
