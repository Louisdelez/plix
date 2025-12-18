# Tasks: CEF UI Shell (Optional)

**Input**: Design documents from `/specs/030-cef-ui-shell/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Manual validation for visual/interaction tests. Automated tests for init/fallback paths.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

```text
crates/plix-client/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── ui_cef/
│   │   ├── mod.rs
│   │   ├── config.rs
│   │   ├── browser.rs
│   │   ├── texture.rs
│   │   ├── input.rs
│   │   └── fallback.rs
│   └── ui/                 # Existing native UI (Feature 005)
└── tests/

assets/
└── ui/
    └── index.html

docs/
└── dev/
    └── cef.md
```

---

## Phase 1: Spike & Technical Decision

**Purpose**: Evaluate CEF Rust bindings and prove off-screen rendering works before committing

**CRITICAL**: This phase must complete with a documented decision before any implementation

- [ ] T001 [P] Create isolated spike directory at `spike/cef-test/` with minimal Cargo.toml
- [ ] T002 [P] Evaluate cef-ui binding: test compilation and basic OSR initialization
- [ ] T003 [P] Evaluate cef-rs binding: test compilation and basic OSR initialization
- [ ] T004 Test CEF paint callback - verify BGRA buffer reception in spike
- [ ] T005 Document binding decision in `docs/dev/cef.md` (existing binding vs FFI wrapper)

**Checkpoint**: Technical decision documented - implementation approach determined

---

## Phase 2: Setup & Foundational (Blocking Prerequisites)

**Purpose**: Create module structure and configuration - BLOCKS all user stories

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T006 Add `cef-ui` feature flag to `crates/plix-client/Cargo.toml` (optional dependency)
- [x] T007 Create `crates/plix-client/src/ui_cef/mod.rs` with CefShell stub and CefError types
- [x] T008 [P] Create `crates/plix-client/src/ui_cef/config.rs` with CefConfig struct
- [x] T009 [P] Create `crates/plix-client/src/ui_cef/input.rs` with InputFocus enum
- [x] T010 Add CLI flags `--cef-ui`, `--no-cef-ui`, `--cef-devtools` to `crates/plix-client/src/main.rs`
- [x] T011 Add `[ui]` config section parsing in client config loader
- [x] T012 Implement conditional CEF initialization in main.rs (disabled by default)
- [x] T013 Add CEF availability logging on startup
- [x] T014 [P] Create `assets/ui/index.html` with basic test page (styled button, text input)
- [x] T015 Test: verify `cargo build -p plix-client --features cef-ui` succeeds
- [x] T016 Test: verify `cargo build -p plix-client` succeeds (without CEF feature)

**Checkpoint**: Foundation ready - CEF module structure exists, feature flag works

---

## Phase 3: User Story 1 - Display HTML UI in Game (Priority: P1) 🎯 MVP

**Goal**: Render HTML/CSS content as a GPU texture visible in the game viewport

**Independent Test**: Load a simple HTML page, render it to texture, display in viewport. Verify HTML renders correctly with CSS styling.

### Implementation for User Story 1

- [ ] T017 [US1] Create `crates/plix-client/src/ui_cef/browser.rs` with CEF browser initialization (OSR mode)
- [ ] T018 [US1] Implement windowless CEF settings (no native window, OSR enabled) in browser.rs
- [ ] T019 [US1] Implement local-only URL scheme handler in browser.rs (block external URLs)
- [ ] T020 [US1] Implement CefRenderHandler paint callback to receive BGRA buffer in browser.rs
- [ ] T021 [US1] Create `crates/plix-client/src/ui_cef/texture.rs` with CefTexture struct
- [ ] T022 [US1] Implement wgpu texture creation (Bgra8Unorm format) in texture.rs
- [ ] T023 [US1] Implement texture update from paint callback buffer in texture.rs
- [ ] T024 [US1] Add UI render pass to renderer (fullscreen quad with CEF texture)
- [ ] T025 [US1] Implement z-order: CEF UI above world, below debug overlays
- [ ] T026 [US1] Implement alpha blending support for transparent UI backgrounds
- [ ] T027 [US1] Implement window resize handling: resize CEF viewport and recreate texture
- [ ] T028 [US1] Implement CEF message loop processing (call each frame)
- [ ] T029 [US1] Implement CEF shutdown on client exit (no zombie subprocesses)
- [ ] T030 [US1] Wire CefShell into main game loop in lib.rs

**Checkpoint**: User Story 1 complete - HTML renders as texture in game viewport

---

## Phase 4: User Story 2 - Input Focus Handling (Priority: P1)

**Goal**: Allow players to interact with HTML UI elements using keyboard and mouse

**Independent Test**: Display HTML form with buttons/inputs. Click button, type in text field, verify interactions work.

### Implementation for User Story 2

- [ ] T031 [US2] Implement InputFocus state machine (Game | CefUI) in input.rs
- [ ] T032 [US2] Implement click-to-focus: detect mouse click on UI area → focus CEF
- [ ] T033 [US2] Implement mouse event forwarding to CEF (position, buttons, scroll) in input.rs
- [ ] T034 [US2] Implement keyboard event forwarding to CEF (keycodes, modifiers, chars) in input.rs
- [ ] T035 [US2] Implement Escape key handling: release CEF focus, return to game
- [ ] T036 [US2] Implement click-outside-UI unfocus handling
- [ ] T037 [US2] Block game input processing while CEF has focus in main input handler
- [ ] T038 [US2] Map winit keycodes to CEF keycodes in input.rs
- [ ] T039 [US2] Map winit mouse buttons to CEF mouse buttons in input.rs

**Checkpoint**: User Story 2 complete - UI is fully interactive via mouse and keyboard

---

## Phase 5: User Story 3 - Optional/Fallback Mode (Priority: P2)

**Goal**: Game works without CEF, using native UI fallback

**Independent Test**: Launch with `--no-cef-ui` flag or disable CEF. Verify game starts normally with native UI.

### Implementation for User Story 3

- [ ] T040 [US3] Create `crates/plix-client/src/ui_cef/fallback.rs` with fallback detection logic
- [ ] T041 [US3] Implement CEF availability check (feature flag, config, init success)
- [ ] T042 [US3] Implement graceful fallback on CEF init failure (no crash)
- [ ] T043 [US3] Implement runtime CEF crash detection and fallback switch
- [ ] T044 [US3] Wire fallback to Feature 005 native UI system in mod.rs
- [ ] T045 [US3] Add clear logging when fallback is activated ("CEF unavailable, using native UI")
- [ ] T046 [US3] Ensure same menus accessible via both CEF UI and native fallback
- [ ] T047 [US3] Test: verify client starts with `--no-cef-ui` flag (uses native UI)
- [ ] T048 [US3] Test: verify client starts when CEF init deliberately fails (uses native UI)

**Checkpoint**: User Story 3 complete - game is playable without CEF

---

## Phase 6: User Story 4 - Engine Integration (Priority: P2)

**Goal**: CEF integrated cleanly with wgpu rendering pipeline without performance issues

**Independent Test**: Run at 60fps with CEF overlay. Measure frame time impact (<2ms). Verify no flickering.

### Implementation for User Story 4

- [ ] T049 [US4] Implement efficient texture upload (queue.write_texture, no CPU readback)
- [ ] T050 [US4] Implement dirty rect optimization in texture.rs (partial updates)
- [ ] T051 [US4] Implement CEF frame rate matching game frame rate in browser.rs
- [ ] T052 [US4] Implement pause CEF rendering when game minimized/unfocused
- [ ] T053 [US4] Add frame time measurement for CEF operations (log if >2ms)
- [ ] T054 [US4] Implement memory leak prevention: proper texture cleanup on resize
- [ ] T055 [US4] Test: verify frame time impact <2ms at 1080p with typical UI
- [ ] T056 [US4] Test: verify no visual artifacts or flickering during gameplay

**Checkpoint**: User Story 4 complete - CEF integrates cleanly with <2ms overhead

---

## Phase 7: User Story 5 - Debug and Development (Priority: P3)

**Goal**: Developers can use CEF DevTools and hot-reload UI

**Independent Test**: Launch with `--cef-devtools`, connect Chrome DevTools, inspect DOM elements.

### Implementation for User Story 5

- [ ] T057 [US5] Implement DevTools enable via `--cef-devtools` flag in browser.rs
- [ ] T058 [US5] Implement UI hot-reload hotkey (F6) to reload current page
- [ ] T059 [US5] Redirect CEF JavaScript console logs to game log (tracing)
- [ ] T060 [US5] Implement debug overlay option (CEF on/off, focus state, page URL)
- [ ] T061 [US5] Log CEF subprocess startup and shutdown events
- [ ] T062 [US5] Test: verify DevTools can connect and inspect rendered page

**Checkpoint**: User Story 5 complete - developers can debug CEF UI effectively

---

## Phase 8: Packaging & Distribution

**Purpose**: Include CEF binaries in distribution, compatible with launcher

- [ ] T063 [P] Document CEF binary bundling in `docs/dev/cef.md` (paths, sizes)
- [ ] T064 [P] Update launcher manifest template to include CEF binaries (Feature 029 integration)
- [ ] T065 Verify client works without CEF feature compiled (no runtime errors)
- [ ] T066 Document size overhead (~100-200MB) in docs/dev/cef.md

**Checkpoint**: CEF is bundled correctly with client distribution

---

## Phase 9: Tests & Validation

**Purpose**: Verify all functionality with manual and automated tests

### Manual Tests

- [ ] T067 Manual test: start client with CEF enabled, verify HTML renders
- [ ] T068 Manual test: start client with `--no-cef-ui`, verify native UI fallback
- [ ] T069 Manual test: click-to-focus and Escape unfocus
- [ ] T070 Manual test: window resize with CEF UI visible
- [ ] T071 Manual test: F6 hot-reload UI

### Automated Tests

- [ ] T072 [P] Unit test: CefConfig parsing (valid/invalid TOML) in crates/plix-client/tests/cef_config_test.rs
- [ ] T073 [P] Unit test: InputFocus state transitions in crates/plix-client/tests/input_focus_test.rs
- [ ] T074 [P] Unit test: fallback activation logic in crates/plix-client/tests/fallback_test.rs

**Checkpoint**: All tests pass, feature is validated

---

## Phase 10: Documentation

**Purpose**: Document feature for users and developers

- [ ] T075 [P] Create `docs/CEF_UI.md` with feature overview and user guide
- [ ] T076 [P] Document CLI flags (`--cef-ui`, `--no-cef-ui`, `--cef-devtools`) in docs/CEF_UI.md
- [ ] T077 [P] Document config options (`cef_enabled`, `cef_initial_page`) in docs/CEF_UI.md
- [ ] T078 [P] Document known limitations (performance, size, local files only) in docs/CEF_UI.md
- [ ] T079 Add note: CEF UI is optional feature, not required for gameplay

**Checkpoint**: Feature is documented for users and developers

---

## Phase 11: Polish & Completion

**Purpose**: Final cleanup and validation

- [ ] T080 Run `cargo fmt --all -- --check`
- [ ] T081 Run `cargo clippy --all-targets -p plix-client`
- [ ] T082 Remove temporary debug logs
- [ ] T083 Verify non-regression: native UI (Feature 005) still works correctly
- [ ] T084 Verify `cargo test -p plix-client` passes
- [ ] T085 Mark feature 030 as complete

**Checkpoint**: Feature complete - all stories implemented, tested, documented

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Spike)**: No dependencies - can start immediately - MUST complete first
- **Phase 2 (Setup)**: Depends on Phase 1 decision - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Phase 2 completion
  - US1 and US2 (P1): Core functionality - should complete first
  - US3 and US4 (P2): Can start after US1/US2 complete
  - US5 (P3): Can start after US1/US2 complete
- **Phase 8-11**: Depend on user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: After Phase 2 - Core rendering (must be first)
- **User Story 2 (P1)**: After Phase 2 - Can run in parallel with US1 (different files)
- **User Story 3 (P2)**: After US1 - Fallback depends on knowing what to fall back from
- **User Story 4 (P2)**: After US1 - Performance tuning of existing rendering
- **User Story 5 (P3)**: After US1 - DevTools for existing CEF

### Parallel Opportunities

**Within Phase 1 (Spike)**:
- T001, T002, T003 can run in parallel (different directories/bindings)

**Within Phase 2 (Setup)**:
- T008, T009 can run in parallel (different files)
- T014 can run in parallel with code tasks

**Within User Stories**:
- US1 and US2 can largely run in parallel (rendering vs input)
- T072, T073, T074 can run in parallel (different test files)
- T075, T076, T077, T078 can run in parallel (different doc sections)

---

## Implementation Strategy

### MVP First (User Story 1 + 2 Only)

1. Complete Phase 1: Spike (5 tasks) → **Technical decision made**
2. Complete Phase 2: Setup (11 tasks) → **Foundation ready**
3. Complete Phase 3: User Story 1 (14 tasks) → **HTML renders in game**
4. Complete Phase 4: User Story 2 (9 tasks) → **UI is interactive**
5. **STOP and VALIDATE**: Test rendering + input
6. Deploy/demo if ready - **MVP complete**

### Incremental Delivery

1. Spike + Setup → Module structure ready
2. Add User Story 1 → HTML renders as texture
3. Add User Story 2 → UI is interactive
4. Add User Story 3 → Fallback works
5. Add User Story 4 → Performance optimized
6. Add User Story 5 → DevTools work
7. Polish → Tests, docs, cleanup

### Single Developer Strategy

Execute in order:
1. Phase 1 (Spike): 5 tasks → Decision documented
2. Phase 2 (Setup): 11 tasks
3. Phase 3 (US1): 14 tasks → **MVP rendering**
4. Phase 4 (US2): 9 tasks → **MVP interactive**
5. Phase 5 (US3): 9 tasks → Fallback
6. Phase 6 (US4): 8 tasks → Performance
7. Phase 7 (US5): 6 tasks → DevTools
8. Phase 8 (Packaging): 4 tasks
9. Phase 9 (Tests): 8 tasks
10. Phase 10 (Docs): 5 tasks
11. Phase 11 (Polish): 6 tasks

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Spike (Phase 1) is MANDATORY before any implementation
- Each user story is independently testable after US1 foundation
- Manual validation required for visual/interaction tests
- CEF feature is optional - client must work without it
- Commit after each task or logical group
- Total: 85 tasks across 11 phases
