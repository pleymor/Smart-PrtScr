# Tasks: Modify Rectangle After Selection

**Input**: Design documents from `/specs/001-modify-rectangle-selection/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md, quickstart.md

**Tests**: No automated tests - manual testing only (as per project conventions)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Primary file**: `src/selection.html` (all changes in this single file)
- No backend changes required

---

## Phase 1: Setup (Foundation)

**Purpose**: Refactor existing code to support the state machine pattern

- [ ] T001 Add constants (HANDLE_SIZE, HANDLE_HIT_AREA, MIN_SELECTION_SIZE) at top of script in src/selection.html
- [ ] T002 Add SelectionState enum and appState object replacing isDrawing boolean in src/selection.html
- [ ] T003 Add helper function normalizeSelection(x1, y1, x2, y2) to compute {x, y, width, height} in src/selection.html
- [ ] T004 Add helper function getHandlePositions(selection) returning 8 handle coordinates in src/selection.html
- [ ] T005 Add helper function getHandleAtPoint(mouseX, mouseY, selection) for hit detection in src/selection.html
- [ ] T006 Add helper function isInsideSelection(mouseX, mouseY, selection) in src/selection.html
- [ ] T007 Add helper function getCursorForPosition(mouseX, mouseY, selection) in src/selection.html

**Checkpoint**: All helper functions in place, state machine structure ready

---

## Phase 2: User Story 1 - Resize Selection Rectangle (Priority: P1) MVP

**Goal**: Allow users to resize their selection by dragging corner or edge handles

**Independent Test**: Draw a selection, drag any handle to resize, verify selection updates correctly

### Implementation for User Story 1

- [ ] T008 [US1] Modify drawSelection() to call new drawSelectionWithHandles() when in MODIFIABLE state in src/selection.html
- [ ] T009 [US1] Implement drawSelectionWithHandles() to render selection border + 8 handles in src/selection.html
- [ ] T010 [US1] Modify mousedown handler to detect handle clicks and transition to RESIZING state in src/selection.html
- [ ] T011 [US1] Implement resizing logic in mousemove handler when state is RESIZING in src/selection.html
- [ ] T012 [US1] Add constraint logic to enforce MIN_SELECTION_SIZE during resize in src/selection.html
- [ ] T013 [US1] Add constraint logic to keep selection within canvas bounds during resize in src/selection.html
- [ ] T014 [US1] Modify mouseup handler to transition from RESIZING back to MODIFIABLE in src/selection.html
- [ ] T015 [US1] Add cursor changes on mousemove based on handle hover (nwse-resize, nesw-resize, ns-resize, ew-resize) in src/selection.html

**Checkpoint**: User Story 1 complete - selection can be resized via handles

---

## Phase 3: User Story 2 - Move Selection Rectangle (Priority: P2)

**Goal**: Allow users to move the entire selection by dragging from the interior

**Independent Test**: Draw a selection, drag from center to move it, verify position changes while size stays constant

### Implementation for User Story 2

- [ ] T016 [US2] Modify mousedown handler to detect interior clicks and transition to MOVING state in src/selection.html
- [ ] T017 [US2] Store dragStart position when entering MOVING state in src/selection.html
- [ ] T018 [US2] Implement move logic in mousemove handler when state is MOVING in src/selection.html
- [ ] T019 [US2] Add constraint logic to keep selection within canvas bounds during move in src/selection.html
- [ ] T020 [US2] Modify mouseup handler to transition from MOVING back to MODIFIABLE in src/selection.html
- [ ] T021 [US2] Add move cursor when hovering over selection interior in src/selection.html

**Checkpoint**: User Story 2 complete - selection can be moved by dragging interior

---

## Phase 4: User Story 3 - Confirm or Cancel Modified Selection (Priority: P3)

**Goal**: Provide clear ways to confirm capture or cancel/restart selection

**Independent Test**: Modify selection, press Enter to confirm or Escape to cancel

### Implementation for User Story 3

- [ ] T022 [US3] Modify mouseup handler in DRAWING state to transition to MODIFIABLE instead of immediate capture in src/selection.html
- [ ] T023 [US3] Update keydown handler for Enter to capture current selection bounds in src/selection.html
- [ ] T024 [US3] Add dblclick handler to confirm capture when double-clicking inside selection in src/selection.html
- [ ] T025 [US3] Modify mousedown handler to detect clicks outside selection and start new DRAWING in src/selection.html
- [ ] T026 [US3] Update instruction text and help text to reflect new behavior in src/selection.html

**Checkpoint**: User Story 3 complete - full interaction flow working

---

## Phase 5: Polish and Edge Cases

**Purpose**: Handle edge cases and improve user experience

- [ ] T027 Add handle swap logic when dragging past opposite edge (prevents negative dimensions) in src/selection.html
- [ ] T028 Ensure selection dimensions display updates during resize/move in src/selection.html
- [ ] T029 Test and fix any issues with full-screen capture fallback (click without drag) in src/selection.html
- [ ] T030 Update CSS for cursor styles if needed in src/selection.html
- [ ] T031 Manual testing: verify all acceptance scenarios from spec.md

---

## Dependencies and Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - foundation for all stories
- **User Story 1 (Phase 2)**: Depends on Setup - core resize functionality
- **User Story 2 (Phase 3)**: Depends on Setup - move functionality (can parallel with US1)
- **User Story 3 (Phase 4)**: Depends on Setup - ties resize/move into flow (needs T022 first)
- **Polish (Phase 5)**: Depends on all user stories complete

### Task Dependencies Within Phases

**Setup (T001-T007)**: Can be done in order or parallel as they are independent functions

**User Story 1 (T008-T015)**:
- T008 then T009 (drawSelectionWithHandles needs to be called)
- T010 then T011 then T012, T013 then T014 (sequential resize flow)
- T015 can parallel with T010-T014

**User Story 2 (T016-T021)**:
- T016 then T017 then T018 then T019 then T020 (sequential move flow)
- T021 can parallel with T016-T020

**User Story 3 (T022-T026)**:
- T022 is critical - changes flow from immediate capture to modifiable
- T023, T024, T025, T026 can be done in any order after T022

---

## Parallel Opportunities

### Within Setup Phase

All T001-T007 can be implemented in parallel (independent helper functions)

### User Stories Can Parallel

Once Setup is complete:
- US1 (T008-T015) and US2 (T016-T021) can be implemented in parallel
- US3 (T022-T026) has the critical T022 that changes base behavior

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T007)
2. Complete Phase 2: User Story 1 (T008-T015)
3. Implement T022 from US3 (critical flow change)
4. **TEST**: Selection stays visible with handles, can resize
5. Handles work but no move yet, Enter/Escape work

### Incremental Delivery

1. Setup + US1 + T022: Resizable selections with confirm/cancel
2. Add US2: Movable selections
3. Add remaining US3: Double-click confirm, click-outside restart
4. Add Polish: Edge cases handled

### Single File Advantage

All work is in `src/selection.html` - no merge conflicts possible. Tasks can be implemented in rapid succession with immediate testing after each.

---

## Notes

- All changes in single file: `src/selection.html`
- No backend (Rust) changes required
- No new dependencies
- Manual testing only - use PrintScreen to test
- Estimated 500-700 lines of new/modified JavaScript code
- Commit after each user story phase completion
