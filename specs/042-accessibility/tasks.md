# Tasks: Accessibility

**Input**: Design documents from `/specs/042-accessibility/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Unit tests requested per constitution (Code Quality principle V).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

This is a multi-crate Rust workspace:
- `crates/plix-client/src/` - Client source code
- `crates/plix-client/tests/` - Client tests
- `assets/ui/` - CEF UI assets (HTML/CSS/JS)
- `docs/` - Documentation

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and foundational types needed by all stories

- [ ] T001 Create accessibility module directory structure in crates/plix-client/src/accessibility/
- [ ] T002 [P] Create accessibility module entry point in crates/plix-client/src/accessibility/mod.rs
- [ ] T003 [P] Add accessibility module to crates/plix-client/src/lib.rs exports
- [ ] T004 [P] Create CSS directory structure for accessibility in assets/ui/css/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 Create AccessibilityConfig struct with defaults in crates/plix-client/src/accessibility/config.rs
- [ ] T006 Implement AccessibilityConfig::validate() with clamping in crates/plix-client/src/accessibility/config.rs
- [ ] T007 [P] Create ColorblindPreset enum with css_class() method in crates/plix-client/src/accessibility/config.rs
- [ ] T008 [P] Create SubtitleConfig struct with SubtitleSize enum in crates/plix-client/src/accessibility/config.rs
- [ ] T009 Add AccessibilityConfig field to GameConfig in crates/plix-client/src/config.rs
- [ ] T010 [P] Create colorblind-filters.svg with SVG feColorMatrix definitions in assets/ui/css/colorblind-filters.svg
- [ ] T011 [P] Create accessibility.css with high-contrast and colorblind classes in assets/ui/css/accessibility.css
- [ ] T012 Unit tests for AccessibilityConfig defaults and validation in crates/plix-client/tests/accessibility_test.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Player Remaps Keybindings (Priority: P1)

**Goal**: Players can customize keybindings via Settings > Controls with conflict detection, swap resolution, and 5-second capture timeout

**Independent Test**: Open settings > Controls, rebind "Forward" from W to Up Arrow, verify the change persists after restart and works in-game

### Implementation for User Story 1

- [ ] T013 [P] [US1] Create KeybindCaptureState enum (Idle, Listening, Conflict) in crates/plix-client/src/accessibility/keybind_capture.rs
- [ ] T014 [P] [US1] Implement 5-second timeout logic in KeybindCaptureState in crates/plix-client/src/accessibility/keybind_capture.rs
- [ ] T015 [US1] Create KeybindConflict struct for conflict representation in crates/plix-client/src/accessibility/keybind_capture.rs
- [ ] T016 [US1] Implement detect_conflict() function using existing Keybinds::action_for_key() in crates/plix-client/src/accessibility/keybind_capture.rs
- [ ] T017 [P] [US1] Add keybinds_list bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T018 [P] [US1] Add keybind_conflict bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T019 [P] [US1] Add keybind_capture_timeout bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T020 [P] [US1] Add start_keybind_capture bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T021 [P] [US1] Add rebind_action bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T022 [P] [US1] Add swap_keybinds bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T023 [P] [US1] Add reset_keybinds bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T024 [US1] Create keybind settings page HTML structure in assets/ui/menus/settings/controls.html
- [ ] T025 [US1] Implement keybind capture UI state machine (listening, timeout, conflict modal) in assets/ui/menus/settings/controls.html
- [ ] T026 [US1] Add /rebind console command in crates/plix-client/src/console/commands.rs
- [ ] T027 [US1] Add /rebind list subcommand in crates/plix-client/src/console/commands.rs
- [ ] T028 [US1] Add /rebind reset subcommand in crates/plix-client/src/console/commands.rs
- [ ] T029 [US1] Unit tests for KeybindCaptureState timeout behavior in crates/plix-client/tests/accessibility_test.rs
- [ ] T030 [US1] Unit tests for conflict detection and swap in crates/plix-client/tests/accessibility_test.rs

**Checkpoint**: Keybinding remapping complete - players can customize all 10 actions with conflict resolution

---

## Phase 4: User Story 2 - Player Adjusts Visual Accessibility Settings (Priority: P1)

**Goal**: Players can adjust UI scale (75-150%), FOV (60-110), high contrast, and colorblind presets with live preview

**Independent Test**: Enable High Contrast mode, verify UI elements have enhanced borders. Enable Deuteranopia preset, verify color shift via CSS filter

### Implementation for User Story 2

- [ ] T031 [P] [US2] Add accessibility_settings bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T032 [P] [US2] Add set_accessibility bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T033 [P] [US2] Add get_accessibility_settings bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T034 [US2] Implement live UI scale application via CSS variable in crates/plix-client/src/ui_cef/menus/settings.rs
- [ ] T035 [US2] Implement live FOV update to camera system in crates/plix-client/src/ui_cef/menus/settings.rs
- [ ] T036 [US2] Implement high contrast class toggle on CEF root in crates/plix-client/src/ui_cef/menus/settings.rs
- [ ] T037 [US2] Implement colorblind preset class toggle on CEF root in crates/plix-client/src/ui_cef/menus/settings.rs
- [ ] T038 [US2] Add visual accessibility section to settings page in assets/ui/menus/settings/display.html
- [ ] T039 [US2] Create UI scale slider component (75-150%) in assets/ui/menus/settings/display.html
- [ ] T040 [US2] Create FOV slider component (60-110) in assets/ui/menus/settings/display.html
- [ ] T041 [US2] Create high contrast toggle component in assets/ui/menus/settings/display.html
- [ ] T042 [US2] Create colorblind preset dropdown component in assets/ui/menus/settings/display.html
- [ ] T043 [US2] Add /ui_scale console command in crates/plix-client/src/console/commands.rs
- [ ] T044 [US2] Add /colorblind console command in crates/plix-client/src/console/commands.rs
- [ ] T045 [US2] Add /highcontrast console command in crates/plix-client/src/console/commands.rs
- [ ] T046 [US2] Apply accessibility settings on startup in crates/plix-client/src/lib.rs
- [ ] T047 [US2] Unit tests for ColorblindPreset css_class() mapping in crates/plix-client/tests/accessibility_test.rs
- [ ] T048 [US2] Unit tests for ui_scale clamping (75-150) in crates/plix-client/tests/accessibility_test.rs

**Checkpoint**: Visual accessibility complete - players can adjust UI scale, FOV, contrast, and colorblind modes

---

## Phase 5: User Story 3 - Player Enables Subtitles for Game Audio (Priority: P3)

**Goal**: Players can enable subtitles for audio events with configurable size and background, max 3-line queue

**Independent Test**: Enable subtitles, trigger a chat message sound, verify subtitle "[Chat]" appears on screen with configured styling

### Implementation for User Story 3

- [ ] T049 [P] [US3] Create AudioEvent enum with subtitle_text() method in crates/plix-client/src/accessibility/audio_events.rs
- [ ] T050 [P] [US3] Create SubtitleEntry struct (id, text, remaining_ms) in crates/plix-client/src/accessibility/subtitle_queue.rs
- [ ] T051 [US3] Create SubtitleQueue struct with max 3 capacity in crates/plix-client/src/accessibility/subtitle_queue.rs
- [ ] T052 [US3] Implement SubtitleQueue::push() with drop-oldest behavior in crates/plix-client/src/accessibility/subtitle_queue.rs
- [ ] T053 [US3] Implement SubtitleQueue::tick() for expiry processing in crates/plix-client/src/accessibility/subtitle_queue.rs
- [ ] T054 [P] [US3] Add subtitle_show bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T055 [P] [US3] Add subtitle_clear bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs
- [ ] T056 [US3] Create subtitle overlay HTML component in assets/ui/ingame/subtitles.html
- [ ] T057 [US3] Implement subtitle queue display with auto-dismiss in assets/ui/ingame/subtitles.html
- [ ] T058 [US3] Style subtitle overlay with size/opacity options in assets/ui/css/accessibility.css
- [ ] T059 [US3] Add subtitle settings section to audio page in assets/ui/menus/settings/audio.html
- [ ] T060 [US3] Create subtitle toggle component in assets/ui/menus/settings/audio.html
- [ ] T061 [US3] Create subtitle size selector (Small/Medium/Large) in assets/ui/menus/settings/audio.html
- [ ] T062 [US3] Create subtitle background opacity slider in assets/ui/menus/settings/audio.html
- [ ] T063 [US3] Add /subtitles console command in crates/plix-client/src/console/commands.rs
- [ ] T064 [US3] Hook subtitle triggers to existing events (chat, player join/leave) in crates/plix-client/src/lib.rs
- [ ] T065 [US3] Unit tests for SubtitleQueue max 3 behavior in crates/plix-client/tests/accessibility_test.rs
- [ ] T066 [US3] Unit tests for SubtitleQueue expiry and drop-oldest in crates/plix-client/tests/accessibility_test.rs
- [ ] T067 [US3] Unit tests for AudioEvent::subtitle_text() in crates/plix-client/tests/accessibility_test.rs

**Checkpoint**: Subtitles complete - players can enable captions for audio events with queue management

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Documentation, validation, and final cleanup

- [ ] T068 [P] Create docs/feature-042.md with options, defaults, and limitations
- [ ] T069 [P] Document keybind capture UX (5s timeout, conflicts) in docs/feature-042.md
- [ ] T070 [P] Document colorblind presets and CSS filter values in docs/feature-042.md
- [ ] T071 [P] Document subtitle system limitations (events depend on audio) in docs/feature-042.md
- [ ] T072 Run cargo fmt --all and verify no formatting issues
- [ ] T073 Run cargo clippy --all and fix any warnings
- [ ] T074 Run cargo test --all and verify all tests pass
- [ ] T075 Validate quickstart.md scenarios work end-to-end

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1 and US2 are both P1 priority - can run in parallel
  - US3 depends on AccessibilityConfig (Phase 2) but not on US1/US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 2 (P1)**: Can start after Foundational - No dependencies on other stories
- **User Story 3 (P3)**: Can start after Foundational - No dependencies on US1/US2

### Within Each User Story

- Bridge message types can be created in parallel
- Config/types before handlers
- Rust handlers before UI implementation
- Unit tests after implementation

### Parallel Opportunities

**Phase 1 (Setup):**
- T002, T003, T004 can all run in parallel

**Phase 2 (Foundational):**
- T007, T008, T010, T011 can run in parallel

**Phase 3 (US1):**
- T013, T014 (capture state) can run in parallel
- T017, T018, T019, T020, T021, T022, T023 (bridge messages) can run in parallel
- T026, T027, T028 (console commands) can run in parallel

**Phase 4 (US2):**
- T031, T032, T033 (bridge messages) can run in parallel
- T039, T040, T041, T042 (UI components) can run in parallel
- T043, T044, T045 (console commands) can run in parallel

**Phase 5 (US3):**
- T049, T050 (types) can run in parallel
- T054, T055 (bridge messages) can run in parallel
- T065, T066, T067 (tests) can run in parallel

**Phase 6 (Polish):**
- T068, T069, T070, T071 (docs) can all run in parallel

---

## Parallel Example: User Story 1

```bash
# Phase 1: Launch bridge message types in parallel:
Task: "Add keybinds_list bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs"
Task: "Add keybind_conflict bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs"
Task: "Add keybind_capture_timeout bridge message type in crates/plix-client/src/ui_cef/bridge/messages.rs"
Task: "Add start_keybind_capture bridge message handler in crates/plix-client/src/ui_cef/bridge/messages.rs"

# Phase 2: Launch console commands in parallel:
Task: "Add /rebind console command in crates/plix-client/src/console/commands.rs"
Task: "Add /rebind list subcommand in crates/plix-client/src/console/commands.rs"
Task: "Add /rebind reset subcommand in crates/plix-client/src/console/commands.rs"
```

## Parallel Example: User Story 2

```bash
# Launch UI component creation in parallel:
Task: "Create UI scale slider component (75-150%) in assets/ui/menus/settings/display.html"
Task: "Create FOV slider component (60-110) in assets/ui/menus/settings/display.html"
Task: "Create high contrast toggle component in assets/ui/menus/settings/display.html"
Task: "Create colorblind preset dropdown component in assets/ui/menus/settings/display.html"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Keybinding remapping)
4. Complete Phase 4: User Story 2 (Visual accessibility)
5. **STOP and VALIDATE**: Test both stories independently
6. Deploy/demo if ready - players can remap keys and adjust visuals

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Keybinding customization works
3. Add User Story 2 → Test independently → Visual accessibility works
4. Add User Story 3 → Test independently → Subtitles work
5. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1 (Keybindings)
   - Developer B: User Story 2 (Visual accessibility)
3. After US1+US2:
   - Either developer: User Story 3 (Subtitles)
4. Stories complete and integrate independently

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Existing Action/Key enums in config.rs are source of truth for keybindings
- CEF bridge messages follow contracts/bridge-messages.md specification
- Native fallback console commands required per FR-024/FR-025
- 5-second capture timeout per clarification Q1
- Max 3 subtitle queue per clarification Q2
