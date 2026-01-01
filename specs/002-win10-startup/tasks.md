# Tasks: Windows 10 Startup Compatibility Fix

**Input**: Design documents from `/specs/002-win10-startup/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: No automated tests requested. Manual testing checklist provided in quickstart.md.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Project structure**: `src-tauri/src/` for Rust backend
- **Configuration**: `src-tauri/Cargo.toml`
- **Main logic**: `src-tauri/src/lib.rs`
- **New module**: `src-tauri/src/autostart.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add dependencies and prepare project structure

- [x] T001 Add `auto-launch = "0.5"` dependency to `src-tauri/Cargo.toml` under `[target.'cfg(windows)'.dependencies]`
- [x] T002 Run `cargo build` in `src-tauri/` to verify dependency resolves correctly
- [x] T003 [P] Create empty `src-tauri/src/autostart.rs` module file with module documentation header

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core autostart module that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T004 Implement `get_exe_path()` function in `src-tauri/src/autostart.rs` to get current executable path
- [x] T005 Implement `get_auto_launch()` function in `src-tauri/src/autostart.rs` to create AutoLaunch instance with app name "SmartPrtScr" and `--hidden` arg
- [x] T006 Implement `is_enabled()` public function in `src-tauri/src/autostart.rs` to check autostart status
- [x] T007 Implement `enable()` public function in `src-tauri/src/autostart.rs` with logging
- [x] T008 Implement `disable()` public function in `src-tauri/src/autostart.rs` with logging
- [x] T009 Add `mod autostart;` declaration at top of `src-tauri/src/lib.rs`
- [x] T010 Run `cargo build` in `src-tauri/` to verify module compiles without errors

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Automatic Application Startup on Windows 10 (Priority: P1) MVP

**Goal**: Fix the core bug - make auto-start work reliably on Windows 10 by replacing the buggy tauri-plugin-autostart

**Independent Test**: Enable auto-start on Windows 10, reboot, verify app launches automatically in system tray. Repeat reboot to verify persistence.

### Implementation for User Story 1

- [x] T011 [US1] Update `get_auto_start` command in `src-tauri/src/lib.rs` to use `autostart::is_enabled()` instead of tauri-plugin-autostart
- [x] T012 [US1] Update `set_auto_start` command in `src-tauri/src/lib.rs` to use `autostart::enable()` and `autostart::disable()`
- [x] T013 [US1] Remove `use tauri_plugin_autostart::ManagerExt;` import from `src-tauri/src/lib.rs` (no longer needed)
- [x] T014 [US1] Remove `.plugin(tauri_plugin_autostart::init(...))` from plugin chain in `src-tauri/src/lib.rs`
- [x] T015 [US1] Remove `tauri-plugin-autostart = "2"` from `src-tauri/Cargo.toml`
- [x] T016 [US1] Run `cargo build` to verify compilation succeeds after plugin removal
- [ ] T017 [US1] Manual test: Build release, install on Windows 10 VM, enable autostart, reboot, verify app starts

**Checkpoint**: User Story 1 complete - Windows 10 auto-start should now work

---

## Phase 4: User Story 2 - Consistent Startup Behavior Across Windows Versions (Priority: P2)

**Goal**: Ensure identical auto-start behavior on both Windows 10 and Windows 11

**Independent Test**: Compare startup behavior on Windows 10 and Windows 11 VMs with identical settings

### Implementation for User Story 2

- [ ] T018 [US2] Manual test: Build release, install on Windows 11 VM, enable autostart, reboot, verify app starts (regression test)
- [ ] T019 [US2] Manual test: Compare toggle behavior between Windows 10 and Windows 11 - verify identical UX
- [ ] T020 [US2] Manual test: Verify registry entries are identical on both platforms (`HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run\SmartPrtScr`)
- [ ] T021 [US2] Manual test: Verify multiple reboot persistence on both Windows 10 (3 reboots) and Windows 11 (3 reboots)

**Checkpoint**: User Story 2 complete - consistent behavior verified on both platforms

---

## Phase 5: User Story 3 - Diagnostic Information for Startup Issues (Priority: P3)

**Goal**: Provide meaningful feedback and logging when auto-start configuration fails

**Independent Test**: Intentionally create startup failures and verify diagnostic output appears in logs and as user notification

### Implementation for User Story 3

- [x] T022 [US3] Implement `verify_and_repair()` function in `src-tauri/src/autostart.rs` that compares expected vs actual state and repairs if needed
- [x] T023 [US3] Add startup verification call in `src-tauri/src/lib.rs` setup function after settings are loaded
- [x] T024 [US3] Add error logging with `[AUTOSTART]` prefix for all failure cases in `src-tauri/src/autostart.rs`
- [x] T025 [P] [US3] Implement non-blocking notification using Tauri dialog API when autostart configuration fails in `src-tauri/src/lib.rs`
- [x] T026 [US3] Handle edge case: Log and notify when user lacks permissions to modify registry
- [x] T027 [US3] Handle edge case: Log and notify when Windows 10 S Mode blocks autostart
- [ ] T028 [US3] Manual test: Disable autostart via Task Manager, launch app, verify notification appears

**Checkpoint**: User Story 3 complete - diagnostic capabilities fully implemented

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [x] T029 Remove any unused imports from `src-tauri/src/lib.rs` after plugin removal
- [x] T030 Run `cargo clippy` in `src-tauri/` and fix any warnings
- [x] T031 [P] Run `cargo fmt` in `src-tauri/` to ensure consistent formatting
- [x] T032 Build final release with `npm run tauri build`
- [ ] T033 Run full testing checklist from `specs/002-win10-startup/quickstart.md` on Windows 10 VM
- [ ] T034 Run full testing checklist from `specs/002-win10-startup/quickstart.md` on Windows 11 VM
- [ ] T035 Update `specs/002-win10-startup/checklists/requirements.md` with test results

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Foundational phase completion
- **User Story 2 (Phase 4)**: Can start after User Story 1 (same platform testing)
- **User Story 3 (Phase 5)**: Can start after Foundational phase (parallel with US1/US2 if desired)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Core fix - must complete first to have working functionality
- **User Story 2 (P2)**: Validation story - depends on US1 being complete to test consistency
- **User Story 3 (P3)**: Enhancement story - can technically start after Foundational, but logically follows US1

### Within Each User Story

- Implementation tasks are sequential within each story
- Tasks marked [P] can run in parallel if no file conflicts
- Complete story before moving to next priority

### Parallel Opportunities

- T003 can run in parallel with T001-T002 (different files)
- T025 can run in parallel with T024 (different files: lib.rs vs autostart.rs)
- T030 and T031 can run in parallel (different tools)

---

## Parallel Example: Phase 2 (Foundational)

```bash
# These must be sequential (same file, dependencies):
T004 → T005 → T006 → T007 → T008  # All in autostart.rs, each builds on previous

# Then:
T009  # lib.rs (can only run after autostart.rs is complete)
T010  # Verification
```

---

## Parallel Example: User Story 3

```bash
# Launch in parallel (different files):
Task T024: "Add error logging in src-tauri/src/autostart.rs"
Task T025: "Implement notification in src-tauri/src/lib.rs"

# Sequential (same file, edge cases):
T026 → T027 → T028
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T010)
3. Complete Phase 3: User Story 1 (T011-T017)
4. **STOP and VALIDATE**: Test on Windows 10 VM - does it work?
5. Deploy/demo if ready - this fixes the core bug!

### Incremental Delivery

1. Setup + Foundational → Core autostart module ready
2. Add User Story 1 → **Core bug fixed** (MVP!)
3. Add User Story 2 → Cross-platform consistency verified
4. Add User Story 3 → Diagnostic capabilities added
5. Polish → Production-ready release

### Estimated Effort

| Phase | Tasks | Complexity |
|-------|-------|------------|
| Setup | 3 | Low |
| Foundational | 7 | Medium |
| User Story 1 | 7 | Medium |
| User Story 2 | 4 | Low (testing only) |
| User Story 3 | 7 | Medium |
| Polish | 7 | Low |
| **Total** | **35** | |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- This feature requires manual testing (no automated test framework in project)
- Testing requires Windows 10 and Windows 11 VMs or physical machines
