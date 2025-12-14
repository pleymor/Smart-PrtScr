# Research: Timestamp on Selection Rectangle

**Feature Branch**: `1-timestamp-overlay`
**Date**: 2024-12-14

## Research Questions

### Q1: How to render timestamp preview on the selection canvas?

**Decision**: Use HTML5 Canvas 2D API directly in `selection.html`

**Rationale**:
- The selection interface already uses a canvas element for drawing the selection rectangle
- Canvas 2D API provides `fillText()`, `measureText()`, and font styling capabilities
- No additional dependencies needed
- Same rendering logic can be used as in `filename-dialog.html` preview

**Alternatives considered**:
- SVG overlay: More complex, no benefit for dynamic text rendering
- WebGL: Overkill for simple text rendering

### Q2: How to retrieve timestamp options from backend during selection?

**Decision**: Add a new Tauri command to fetch timestamp options and call it when selection window initializes

**Rationale**:
- `get_timestamp_options` command already exists in the backend
- Frontend can invoke it using `window.__TAURI__.core.invoke('get_timestamp_options')`
- Options are cached and reused throughout the selection session

**Alternatives considered**:
- Pass options via URL parameters: Limited by URL length, not clean
- Global state in main window: Selection window is independent

### Q3: How to handle fullscreen capture with timestamp preview?

**Decision**: Create a fullscreen selection rectangle automatically when capture is triggered, showing the timestamp preview

**Rationale**:
- Clarification session confirmed user wants to see the timestamp preview even for fullscreen captures
- Selection window will open with a pre-created selection covering the entire screen
- User confirms capture with Enter or double-click (consistent with existing flow)

**Alternatives considered**:
- Skip preview for fullscreen: Rejected in clarification session
- Show temporary overlay: More complex, inconsistent UX

### Q4: How to handle banner position outside the selection rectangle?

**Decision**: Visually extend the canvas rendering area to include the banner, but capture only the original selection bounds

**Rationale**:
- Clarification confirmed banner should appear outside the selection (not covering captured content)
- Canvas can draw beyond the selection rectangle for preview purposes
- Final capture uses original bounds; timestamp is applied during save (existing flow)

**Alternatives considered**:
- Modify selection bounds to include banner: Would change captured area, rejected

### Q5: How to remove the preview thumbnail from filename-dialog.html?

**Decision**: Remove the `<canvas id="preview-canvas">` element and related JavaScript code

**Rationale**:
- Simple HTML/JS modification
- Window height will auto-adjust via existing `adjustWindowHeight()` function
- Timestamp options section remains for configuration changes

**Alternatives considered**:
- Hide with CSS: Leaves dead code, not clean
- Keep preview but smaller: Defeats purpose of simplification

## Technical Findings

### Existing Canvas Drawing Code (selection.html)

The `drawSelectionWithHandles()` function at line 214-245 handles selection rendering:
- Uses `ctx.strokeRect()` for rectangle border
- Uses `ctx.fillText()` for dimensions display
- Drawing happens on each mouse move during selection

### Timestamp Options Structure

From `lib.rs` lines 136-166:
```rust
pub struct TimestampOptions {
    pub enabled: bool,
    pub font_size: u32,
    pub display_type: String,  // "banner-dark", "banner-light", "overlay"
    pub text_color: String,
    pub text_align: String,    // "left", "center", "right"
    pub position: String,      // "top", "bottom"
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}
```

### Preview Rendering Logic (filename-dialog.html)

The `updatePreview()` function at lines 514-605 contains the complete timestamp rendering logic:
- Text colors mapping (lines 482-493)
- Font style construction
- Banner vs overlay positioning
- Underline drawing

This logic can be adapted for the selection canvas.

## Dependencies

No new dependencies required. All functionality uses existing:
- HTML5 Canvas 2D API
- Tauri IPC (`invoke`)
- Existing timestamp options structure
