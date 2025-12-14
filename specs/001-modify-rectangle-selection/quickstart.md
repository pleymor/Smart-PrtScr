# Quickstart: Modify Rectangle After Selection

**Feature**: 001-modify-rectangle-selection
**Date**: 2024-12-14

## Overview

This feature enhances the screenshot selection window to allow users to modify their selection after drawing it, instead of immediately capturing. Users can resize via handles, move by dragging, and confirm with Enter or double-click.

## Prerequisites

- Tauri development environment set up
- Node.js and npm installed
- Rust toolchain installed

## Development Setup

```bash
# Clone and enter project
cd C:/Users/pleym/Projects/simple-printscreen

# Install dependencies
npm install

# Run in development mode
npm run tauri:dev
```

## File to Modify

**Primary file**: `src/selection.html`

This single file contains all the selection UI logic. The feature is implemented entirely in the inline JavaScript.

## Implementation Steps

### 1. Add State Management

Replace the simple `isDrawing` boolean with a proper state machine:

```javascript
const SelectionState = {
  IDLE: 'idle',
  DRAWING: 'drawing', 
  MODIFIABLE: 'modifiable',
  RESIZING: 'resizing',
  MOVING: 'moving'
};

let appState = {
  current: SelectionState.IDLE,
  selection: null,
  activeHandle: null,
  dragStart: null
};
```

### 2. Modify mouseup Handler

Instead of immediately calling `completeSelection()`, transition to `MODIFIABLE` state:

```javascript
canvas.addEventListener('mouseup', (e) => {
  if (appState.current === SelectionState.DRAWING) {
    // Calculate selection bounds
    const sel = normalizeSelection(startX, startY, currentX, currentY);
    
    if (sel.width > MIN_SIZE && sel.height > MIN_SIZE) {
      appState.selection = sel;
      appState.current = SelectionState.MODIFIABLE;
      drawSelectionWithHandles();
    } else {
      // Too small - treat as click for full screen
      completeSelection(0, 0, canvas.width, canvas.height);
    }
  }
  // ... handle other states
});
```

### 3. Add Handle Drawing

```javascript
function drawHandles(ctx, sel) {
  const handles = getHandlePositions(sel);
  ctx.fillStyle = 'white';
  ctx.strokeStyle = '#333';
  ctx.lineWidth = 1;
  
  Object.values(handles).forEach(pos => {
    ctx.fillRect(pos.x - 5, pos.y - 5, 10, 10);
    ctx.strokeRect(pos.x - 5, pos.y - 5, 10, 10);
  });
}
```

### 4. Add Hit Detection

```javascript
function getHandleAtPoint(x, y, sel) {
  const handles = getHandlePositions(sel);
  const hitSize = 20; // Larger than visual for easier interaction
  
  for (const [name, pos] of Object.entries(handles)) {
    if (Math.abs(x - pos.x) < hitSize/2 && Math.abs(y - pos.y) < hitSize/2) {
      return name;
    }
  }
  return null;
}

function isInsideSelection(x, y, sel) {
  return x >= sel.x && x <= sel.x + sel.width &&
         y >= sel.y && y <= sel.y + sel.height;
}
```

### 5. Update Keyboard Handler

```javascript
document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    cancelSelection();
  } else if (e.key === 'Enter') {
    if (appState.selection) {
      const s = appState.selection;
      completeSelection(s.x, s.y, s.width, s.height);
    } else {
      completeSelection(0, 0, canvas.width, canvas.height);
    }
  }
});
```

### 6. Add Double-Click Handler

```javascript
canvas.addEventListener('dblclick', (e) => {
  if (appState.current === SelectionState.MODIFIABLE && appState.selection) {
    if (isInsideSelection(e.clientX, e.clientY, appState.selection)) {
      const s = appState.selection;
      completeSelection(s.x, s.y, s.width, s.height);
    }
  }
});
```

## Testing

1. Press PrintScreen to open selection window
2. Draw a rectangle - it should remain visible with handles
3. Test resize by dragging corners/edges
4. Test move by dragging from center
5. Test confirm with Enter or double-click
6. Test cancel with Escape
7. Test new selection by clicking outside

## Key Constants

```javascript
const HANDLE_SIZE = 10;      // Visual handle size
const HANDLE_HIT_AREA = 20;  // Hit detection area
const MIN_SELECTION_SIZE = 10; // Minimum selection dimension
```
