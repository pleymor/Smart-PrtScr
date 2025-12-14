# Tasks: Timestamp on Selection Rectangle

**Input**: Design documents from `/specs/1-timestamp-overlay/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Not requested - manual testing only (per plan.md)

**Organization**: Tasks are grouped by user story/component to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which component this task belongs to (US1, US2, US3)
- Includes exact file paths in descriptions

## Path Conventions

- **Frontend**: `src/` (HTML/CSS/JS files)
- **Backend**: `src-tauri/src/` (Rust code)

---

## Phase 1: Setup

**Purpose**: No setup tasks required - existing project is already configured

- [x] T001 Verify development environment with `npm run tauri:dev`

---

## Phase 2: Foundational

**Purpose**: No foundational tasks required

**Checkpoint**: Foundation ready - user story implementation can begin

---

## Phase 3: User Story 1 - Live Timestamp Preview (Priority: P1) ✅ DONE

**Goal**: Display timestamp preview directly on the selection rectangle during capture, with real-time updates

**Independent Test**: Start capture, draw selection, verify timestamp appears with configured style. Resize selection and verify timestamp updates.

### Implementation for User Story 1

- [x] T002 [US1] Add textColors map constant in `src/selection.html`
- [x] T003 [US1] Add timestampOptions field to appState object in `src/selection.html`
- [x] T004 [US1] Add loadTimestampOptions() async function in `src/selection.html`
- [x] T005 [US1] Add drawTimestampPreview(sel, options) function in `src/selection.html`
- [x] T006 [US1] Implement banner-dark rendering in drawTimestampPreview() in `src/selection.html`
- [x] T007 [US1] Implement banner-light rendering in drawTimestampPreview() in `src/selection.html`
- [x] T008 [US1] Implement overlay rendering in drawTimestampPreview() in `src/selection.html`
- [x] T009 [US1] Implement text styling (bold, italic, underline, alignment, color) in drawTimestampPreview() in `src/selection.html`
- [x] T010 [US1] Modify drawSelection() to call drawTimestampPreview() when enabled in `src/selection.html`
- [x] T011 [US1] Call loadTimestampOptions() during initialization in `src/selection.html`
- [x] T012 [US1] Add fullscreen capture event listener for 'capture-full-screen' event in `src/selection.html`
- [x] T013 [US1] Implement createFullscreenSelection() function for fullscreen mode in `src/selection.html`

**Checkpoint**: User Story 1 complete - timestamp preview visible during selection

---

## Phase 4: User Story 2 - Remove Preview Thumbnail (Priority: P2) ✅ DONE

**Goal**: Simplify save dialog by removing the preview thumbnail

**Independent Test**: Complete a capture, verify save dialog opens without preview canvas, with reduced window height.

### Implementation for User Story 2

- [x] T014 [US2] Remove `.preview` CSS styles in `src/filename-dialog.html`
- [x] T015 [US2] Remove `<div class="preview">` and `<canvas id="preview-canvas">` HTML elements in `src/filename-dialog.html`
- [x] T016 [US2] Remove `originalImage` variable declaration in `src/filename-dialog.html`
- [x] T017 [US2] Remove updatePreview() function in `src/filename-dialog.html`
- [x] T018 [US2] Remove updateOptionsState() function calls to updatePreview() in `src/filename-dialog.html`
- [x] T019 [US2] Remove get_preview_image invocation in init() function in `src/filename-dialog.html`
- [x] T020 [US2] Remove image loading logic (img.onload, originalImage assignment) in init() in `src/filename-dialog.html`
- [x] T021 [US2] Simplify adjustWindowHeight() call in init() - remove dependency on image loading in `src/filename-dialog.html`

**Checkpoint**: User Story 2 complete - save dialog simplified without preview

---

## Phase 5: User Story 3 - Simultaneous Windows & Real-time Sync (Priority: P3) 🆕 NEW

**Goal**: Open filename-dialog simultaneously with selection window and sync timestamp options in real-time

**Independent Test**: Start capture, verify both windows open together. Change options in dialog and verify the selection preview updates immediately.

### Implementation for User Story 3

#### Backend Changes (lib.rs)

- [x] T022 [US3] Modify open_selection_window() to call open_filename_dialog() in `src-tauri/src/lib.rs`
- [x] T023 [US3] Position filename-dialog in bottom-right corner (not centered) in `src-tauri/src/lib.rs`
- [x] T024 [US3] Ensure selection window has focus after both windows open in `src-tauri/src/lib.rs`
- [x] T025 [US3] Remove open_filename_dialog() call from process_selection() in `src-tauri/src/lib.rs`

#### filename-dialog Changes

- [x] T026 [US3] Import Tauri emit function in `src/filename-dialog.html`
- [x] T027 [US3] Add emitOptionsChanged() function to broadcast options in `src/filename-dialog.html`
- [x] T028 [US3] Call emitOptionsChanged() on every option change event in `src/filename-dialog.html`
- [x] T029 [US3] Emit initial options on window load in `src/filename-dialog.html`
- [x] T030 [US3] Update Save button to only save options (not trigger capture) in `src/filename-dialog.html`
- [x] T031 [US3] Remove filename input requirement - make optional for when user confirms in `src/filename-dialog.html`

#### selection.html Changes

- [x] T032 [US3] Add listener for 'timestamp-options-changed' event in `src/selection.html`
- [x] T033 [US3] Update appState.timestampOptions when event received in `src/selection.html`
- [x] T034 [US3] Call drawSelection() to refresh preview after options update in `src/selection.html`
- [x] T035 [US3] Keep loadTimestampOptions() as fallback - options from dialog override in `src/selection.html`

**Checkpoint**: User Story 3 complete - both windows open together, options sync in real-time

---

## Phase 6: Polish & Validation

**Purpose**: Final validation and edge case handling

- [ ] T036 Manual test: Timestamp with banner-dark type, bottom position
- [ ] T037 Manual test: Timestamp with banner-light type, top position
- [ ] T038 Manual test: Timestamp with overlay type
- [ ] T039 Manual test: All text colors (white, black, gray, red, green, blue, yellow, cyan, magenta)
- [ ] T040 Manual test: Font sizes at boundaries (8px, 72px)
- [ ] T041 Manual test: Text alignments (left, center, right)
- [ ] T042 Manual test: Style combinations (bold, italic, underline)
- [ ] T043 Manual test: Timestamp disabled - no overlay shown
- [ ] T044 Manual test: Fullscreen capture flow with timestamp
- [ ] T045 Manual test: Very small selection rectangle
- [ ] T046 Manual test: Save dialog opens without preview, correct height
- [ ] T047 Manual test: Real-time option sync between windows
- [ ] T048 Manual test: Window positioning (dialog in corner, selection focused)
- [ ] T049 Manual test: Close dialog before confirming selection
- [ ] T050 Run quickstart.md validation scenarios

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - verify environment
- **Foundational (Phase 2)**: N/A - no tasks
- **User Story 1 (Phase 3)**: ✅ DONE
- **User Story 2 (Phase 4)**: ✅ DONE
- **User Story 3 (Phase 5)**: Can start now - builds on US1 and US2
- **Polish (Phase 6)**: Depends on User Story 3 completion

### Within User Story 3

```
T022-T025 (Backend) ──┬──► T026-T031 (filename-dialog) ──┬──► T032-T035 (selection.html)
                      │                                   │
                      └─── Can be developed in parallel ──┘
```

### Parallel Opportunities

Within User Story 3:
- Backend changes (T022-T025) must be done first
- Frontend changes in filename-dialog (T026-T031) and selection.html (T032-T035) can be done in parallel after backend

---

## Implementation Strategy

### Current Status

1. ✅ Phase 1: Setup complete
2. ✅ Phase 3: User Story 1 complete (timestamp preview)
3. ✅ Phase 4: User Story 2 complete (remove preview thumbnail)
4. 🔄 Phase 5: User Story 3 in progress (simultaneous windows + real-time sync)

### Next Steps

1. Complete T022-T025 (backend changes)
2. Complete T026-T031 (filename-dialog event emission)
3. Complete T032-T035 (selection.html event listener)
4. Run validation tests (Phase 6)

---

## Notes

- Backend changes are required first to enable simultaneous window opening
- Tauri events API is used for inter-window communication
- Selection window maintains keyboard focus for shortcuts (Enter, Escape)
- filename-dialog can be closed independently without affecting selection
- Commit after each logical group of tasks
