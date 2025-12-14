# Data Model: Modify Rectangle After Selection

**Feature**: 001-modify-rectangle-selection
**Date**: 2024-12-14

## Entities

### SelectionState (Enum)

The current interaction mode for the selection window.

| Value | Description |
|-------|-------------|
| `IDLE` | No selection exists, waiting for user to start drawing |
| `DRAWING` | User is currently drawing a new selection rectangle |
| `MODIFIABLE` | Selection exists and can be modified (resized/moved) |
| `RESIZING` | User is currently resizing the selection via a handle |
| `MOVING` | User is currently moving the entire selection |

### Selection (Object)

The rectangular selection area on screen.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| `x` | number | Left edge X coordinate in pixels | >= 0, < canvas.width |
| `y` | number | Top edge Y coordinate in pixels | >= 0, < canvas.height |
| `width` | number | Width in pixels | >= MIN_SIZE (10) |
| `height` | number | Height in pixels | >= MIN_SIZE (10) |

### Handle (Enum)

The 8 resize handles around the selection rectangle.

| Value | Position | Cursor | Resize Behavior |
|-------|----------|--------|-----------------|
| `nw` | Top-left corner | nwse-resize | Anchors SE corner |
| `n` | Top edge center | ns-resize | Anchors bottom edge |
| `ne` | Top-right corner | nesw-resize | Anchors SW corner |
| `e` | Right edge center | ew-resize | Anchors left edge |
| `se` | Bottom-right corner | nwse-resize | Anchors NW corner |
| `s` | Bottom edge center | ns-resize | Anchors top edge |
| `sw` | Bottom-left corner | nesw-resize | Anchors NE corner |
| `w` | Left edge center | ew-resize | Anchors right edge |

### AppState (Object)

The complete state of the selection interaction.

| Field | Type | Description |
|-------|------|-------------|
| `current` | SelectionState | Current interaction mode |
| `selection` | Selection \| null | Current selection bounds, null if none |
| `activeHandle` | Handle \| null | Handle being dragged during resize |
| `dragStart` | {x, y} \| null | Mouse position at start of move operation |
| `lastClick` | number | Timestamp of last click (for double-click detection if needed) |

## Constants

| Name | Value | Description |
|------|-------|-------------|
| `HANDLE_SIZE` | 10 | Visual size of handles in pixels |
| `HANDLE_HIT_AREA` | 20 | Hit detection area for handles in pixels |
| `MIN_SELECTION_SIZE` | 10 | Minimum width/height of selection in pixels |

## State Transitions

```
IDLE
  |-- mousedown --> DRAWING
  
DRAWING
  |-- mouseup (valid size) --> MODIFIABLE
  |-- mouseup (too small) --> IDLE (or full screen capture)
  |-- Escape --> IDLE (cancel)
  
MODIFIABLE
  |-- mousedown on handle --> RESIZING
  |-- mousedown inside selection --> MOVING
  |-- mousedown outside selection --> DRAWING (new selection)
  |-- Enter / double-click inside --> [CAPTURE] (complete)
  |-- Escape --> IDLE (cancel)
  
RESIZING
  |-- mouseup --> MODIFIABLE
  |-- Escape --> MODIFIABLE (revert?)
  
MOVING
  |-- mouseup --> MODIFIABLE
  |-- Escape --> MODIFIABLE (revert?)
```

## Visual Representation

```
    N handle
      |
  NW--+--NE
  |       |
W-+       +-E
  |       |
  SW--+--SE
      |
    S handle

Handle visual: 10x10 white square with 1px dark border
Hit area: 20x20 centered on handle position
Interior: Move cursor when hovering
Exterior: Crosshair cursor (for new selection)
```
