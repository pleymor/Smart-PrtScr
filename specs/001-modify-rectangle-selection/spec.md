# Feature Specification: Modify Rectangle After Selection

**Feature Branch**: `001-modify-rectangle-selection`
**Created**: 2024-12-14
**Status**: Draft
**Input**: User description: "allow to modify the rectangle after selection"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Resize Selection Rectangle (Priority: P1)

As a user taking a screenshot with rectangular selection, I want to resize my selection after drawing it so that I can fine-tune the capture area without starting over.

**Why this priority**: This is the core functionality - being able to adjust selection size is the most common modification users need when they didn't get the exact area on first attempt.

**Independent Test**: Can be fully tested by drawing a selection, then dragging a corner/edge handle to resize, and verifying the selection updates correctly.

**Acceptance Scenarios**:

1. **Given** a selection rectangle is drawn on screen, **When** the user hovers over a corner of the selection, **Then** the cursor changes to a resize cursor indicating diagonal resize capability.
2. **Given** a selection rectangle is drawn on screen, **When** the user drags a corner handle, **Then** the selection resizes from that corner while the opposite corner stays anchored.
3. **Given** a selection rectangle is drawn on screen, **When** the user hovers over an edge (not corner), **Then** the cursor changes to indicate horizontal or vertical resize capability.
4. **Given** a selection rectangle is drawn on screen, **When** the user drags an edge handle, **Then** only that edge moves while other edges remain fixed.

---

### User Story 2 - Move Selection Rectangle (Priority: P2)

As a user, I want to move my entire selection to a different position so that I can capture a different area without changing the selection size.

**Why this priority**: Moving the selection is the second most common adjustment - users often draw the right size but in the wrong position.

**Independent Test**: Can be fully tested by drawing a selection, then dragging from the center to move it, and verifying position changes while size remains constant.

**Acceptance Scenarios**:

1. **Given** a selection rectangle is drawn on screen, **When** the user hovers over the center area of the selection, **Then** the cursor changes to a move cursor.
2. **Given** a selection rectangle is drawn on screen, **When** the user drags from the center area, **Then** the entire selection moves following the cursor while maintaining its dimensions.
3. **Given** a selection is being moved, **When** the selection would move partially off-screen, **Then** the selection is constrained to remain within screen bounds.

---

### User Story 3 - Confirm or Cancel Modified Selection (Priority: P3)

As a user, I want clear ways to confirm my modified selection or cancel and start over so that I have full control over the capture process.

**Why this priority**: Users need clear exit paths from the modification state - either to capture the adjusted selection or to abandon it.

**Independent Test**: Can be fully tested by modifying a selection and then pressing Enter to confirm or Escape to cancel.

**Acceptance Scenarios**:

1. **Given** a selection rectangle exists (modified or not), **When** the user presses Enter or double-clicks inside the selection, **Then** the screenshot is captured using the current selection bounds.
2. **Given** a selection rectangle exists, **When** the user presses Escape, **Then** the selection is cancelled and the capture mode exits.
3. **Given** a selection rectangle exists, **When** the user clicks outside the selection area, **Then** the current selection is discarded and a new selection can be drawn from that point.

---

### Edge Cases

- What happens when user tries to resize the selection to zero or negative dimensions? The selection should have a minimum size (e.g., 10x10 pixels) and not allow inversion.
- What happens when user tries to move/resize selection beyond screen boundaries? The selection should be constrained to remain fully within the visible screen area.
- What happens if user releases the mouse button during resize/move outside the selection window? The operation should complete based on the last valid cursor position.
- What happens on multi-monitor setups? The selection should be constrained to the monitor where the capture was initiated.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST display resize handles at all four corners and four edges of the selection rectangle after initial selection is drawn.
- **FR-002**: System MUST change the cursor to appropriate resize cursors when hovering over handles (diagonal for corners, horizontal/vertical for edges).
- **FR-003**: System MUST allow resizing the selection by dragging any handle, with the opposite side remaining anchored.
- **FR-004**: System MUST change the cursor to a move cursor when hovering over the interior of the selection.
- **FR-005**: System MUST allow moving the entire selection by dragging from the interior.
- **FR-006**: System MUST constrain the selection to remain within screen bounds during move and resize operations.
- **FR-007**: System MUST enforce a minimum selection size of 10x10 pixels.
- **FR-008**: System MUST capture the screenshot when user confirms (Enter key or double-click inside selection).
- **FR-009**: System MUST cancel the selection when user presses Escape.
- **FR-010**: System MUST allow starting a new selection when user clicks outside the current selection.
- **FR-011**: System MUST provide visual feedback (selection outline, handles) that clearly shows the current selection state.

### Key Entities

- **Selection Rectangle**: The rectangular area defined by the user, with properties: position (x, y), dimensions (width, height), and state (drawing, modifying, confirmed).
- **Handle**: Interactive resize points at corners (NW, NE, SW, SE) and edge midpoints (N, S, E, W) of the selection.
- **Selection State**: The current mode of interaction - initial drawing, resize mode, move mode, or idle/modifiable.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can adjust their selection (resize or move) and capture in under 5 seconds from initial selection.
- **SC-002**: 95% of selection adjustments result in successful captures (not cancelled due to frustration).
- **SC-003**: Users can precisely capture their intended area on the first attempt with modifications, reducing re-capture rate by 50%.
- **SC-004**: Selection handles are visible and accessible on all supported display resolutions and DPI settings.

## Assumptions

- The application already has a working rectangular selection capture mechanism that this feature enhances.
- Visual feedback (selection border, overlay) already exists and will be extended with handles.
- The existing keyboard shortcuts (PrintScreen variants) will continue to initiate the capture mode; this feature adds post-selection modification.
- Handle hit areas will be slightly larger than visual handles for easier interaction (standard UI practice).
