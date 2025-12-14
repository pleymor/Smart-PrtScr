# Research: Modify Rectangle After Selection

**Feature**: 001-modify-rectangle-selection
**Date**: 2024-12-14

## Research Questions

### 1. Best approach for interactive resize handles in HTML5 Canvas

**Decision**: Use Canvas 2D API to draw handles and track mouse position for hit detection

**Rationale**:
- The existing implementation already uses Canvas for selection drawing
- No external dependencies needed
- Full control over visual appearance
- Hit detection is straightforward with rectangle bounds checking

**Alternatives Considered**:
- **DOM-based handles (div elements)**: Rejected because mixing DOM overlays with Canvas would complicate coordinate systems and z-index management
- **SVG overlays**: Rejected because it adds complexity for no benefit in this simple use case
- **Third-party library (fabric.js, konva.js)**: Rejected because it would add significant bundle size for a simple feature

### 2. State machine implementation pattern

**Decision**: Simple JavaScript state object with explicit state transitions

**Rationale**:
- Clean separation of behavior based on current state
- Easy to extend with new states if needed
- No framework/library overhead
- Matches existing code style in selection.html

**Implementation**:
```javascript
const SelectionState = {
  IDLE: 'idle',
  DRAWING: 'drawing',
  MODIFIABLE: 'modifiable',
  RESIZING: 'resizing',
  MOVING: 'moving'
};

let state = {
  current: SelectionState.IDLE,
  selection: null,  // {x, y, width, height}
  activeHandle: null,  // which handle is being dragged
  dragStart: null  // {x, y} for move operations
};
```

### 3. Handle hit detection approach

**Decision**: Calculate handle positions from selection bounds, use distance-based hit detection with padding

**Rationale**:
- Handles are computed dynamically from selection bounds
- Larger hit area than visual size improves usability
- Simple to implement and maintain

**Handle Positions**:
```javascript
function getHandles(sel) {
  const {x, y, width, height} = sel;
  return {
    nw: {x: x, y: y},
    n:  {x: x + width/2, y: y},
    ne: {x: x + width, y: y},
    e:  {x: x + width, y: y + height/2},
    se: {x: x + width, y: y + height},
    s:  {x: x + width/2, y: y + height},
    sw: {x: x, y: y + height},
    w:  {x: x, y: y + height/2}
  };
}
```

### 4. Cursor management

**Decision**: Set cursor via CSS on canvas element based on hover position

**Rationale**:
- Standard web approach
- Immediate visual feedback
- No performance impact

**Cursor Mapping**:
| Handle | Cursor |
|--------|--------|
| nw, se | nwse-resize |
| ne, sw | nesw-resize |
| n, s | ns-resize |
| e, w | ew-resize |
| interior | move |
| outside | crosshair |

### 5. Resize behavior when dragging past opposite edge

**Decision**: Swap the anchored edge/corner when crossing the opposite boundary

**Rationale**:
- Standard behavior in most design tools (Photoshop, Figma)
- Intuitive for users
- Prevents negative dimensions

**Example**: Dragging NW handle past SE corner position would:
1. Selection inverts
2. Active handle becomes SE
3. Original SE position becomes the new anchor

### 6. Double-click detection

**Decision**: Use native `dblclick` event on canvas

**Rationale**:
- Browser handles the timing threshold
- Consistent with user expectations
- No custom timing code needed

## No Unresolved Clarifications

All technical decisions have been made. Ready for Phase 1 design.
