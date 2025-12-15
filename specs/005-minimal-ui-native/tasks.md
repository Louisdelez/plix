# Tasks: Minimal Native UI

**Input**: Design documents from `/specs/005-minimal-ui-native/`
**Prerequisites**: plan.md (required), spec.md (required), data-model.md, contracts/config.md, contracts/menu-state.md

**Tests**: Unit tests included for config persistence (per constitution V. Code Quality).

**Organization**: Tasks grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5, US6, US7)
- Include exact file paths in descriptions

## Path Conventions

Multi-crate workspace structure:
- **plix-client**: `crates/plix-client/src/` - Client logic and UI
- **Tests**: `crates/plix-client/tests/`

---

## Phase 1: Setup (Dependencies & Project Structure)

**Purpose**: Add required dependencies and create module structure

- [x] T001 [P] Add `toml = "0.8"` and `dirs = "5.0"` to crates/plix-client/Cargo.toml
- [x] T002 [P] Create crates/plix-client/src/ui/state.rs (empty module)
- [x] T003 [P] Create crates/plix-client/src/ui/crosshair.rs (empty module)
- [x] T004 [P] Create crates/plix-client/src/ui/menu.rs (empty module)
- [x] T005 [P] Create crates/plix-client/src/config.rs (empty module)
- [x] T006 Update crates/plix-client/src/ui/mod.rs to export new modules
- [x] T007 Update crates/plix-client/src/lib.rs to export config module
- [x] T008 Verify client builds with `cargo build -p plix-client`

**Checkpoint**: Module structure ready, dependencies added

---

## Phase 2: Foundational (UI State & Config Infrastructure)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### UI State Machine

- [ ] T009 Define UiState enum (InGame, PauseMenu, Settings, Rebinding(Action), ConfirmSwap) in crates/plix-client/src/ui/state.rs
- [ ] T010 Add Default impl for UiState (returns InGame) in crates/plix-client/src/ui/state.rs
- [ ] T011 Add ui_state: UiState field to GameState struct in crates/plix-client/src/main.rs
- [ ] T012 Add should_process_gameplay_input() method based on ui_state in crates/plix-client/src/main.rs
- [ ] T013 Gate gameplay input processing (WASD, mouse, attacks) on should_process_gameplay_input() in crates/plix-client/src/main.rs

### Mouse Capture Rules

- [ ] T014 Create apply_cursor_state() method that grabs/releases cursor based on UiState in crates/plix-client/src/main.rs
- [ ] T015 Call apply_cursor_state() whenever ui_state changes in crates/plix-client/src/main.rs

### Config Infrastructure

- [ ] T016 [P] Define Action enum (Forward, Backward, Left, Right, Jump, Attack, PlaceBlock, RemoveBlock, Pause) in crates/plix-client/src/config.rs
- [ ] T017 [P] Define Key enum (extended with all keyboard keys + mouse buttons) in crates/plix-client/src/config.rs
- [ ] T018 Define Keybinds struct with HashMap<Action, Key> and Default impl in crates/plix-client/src/config.rs
- [ ] T019 Define GameConfig struct (sensitivity, fov_degrees, fullscreen, audio_muted, keybinds) with serde derive in crates/plix-client/src/config.rs
- [ ] T020 Implement Default for GameConfig with default values in crates/plix-client/src/config.rs
- [ ] T021 Implement config_path() function using dirs crate in crates/plix-client/src/config.rs
- [ ] T022 Implement load_config() with defaults fallback in crates/plix-client/src/config.rs
- [ ] T023 Implement save_config() with atomic write (temp file + rename) in crates/plix-client/src/config.rs
- [ ] T024 Implement validate() method to clamp out-of-range values in crates/plix-client/src/config.rs

### Config Tests

- [ ] T025 [P] Unit test: load missing file returns defaults in crates/plix-client/tests/config_test.rs
- [ ] T026 [P] Unit test: save and reload roundtrip in crates/plix-client/tests/config_test.rs
- [ ] T027 [P] Unit test: clamp out-of-range values in crates/plix-client/tests/config_test.rs

### Config Integration

- [ ] T028 Add config: GameConfig field to GameState in crates/plix-client/src/main.rs
- [ ] T029 Load config in GameState::new() in crates/plix-client/src/main.rs
- [ ] T030 Verify client builds and runs with `cargo run -p plix-client`

**Checkpoint**: Foundation ready - user story implementation can begin

---

## Phase 3: User Story 1 - Crosshair Display (Priority: P1) 🎯 MVP

**Goal**: Display a crosshair at screen center during gameplay, hidden when menu is open.

**Independent Test**: Launch windowed client, verify crosshair appears at screen center during gameplay, disappears when menu is open.

### 2D Overlay Rendering Infrastructure

- [ ] T031 [US1] Create UI vertex/fragment shader for screen-space quads in crates/plix-client/src/render/ui.wgsl (new)
- [ ] T032 [US1] Add UIVertex struct and UI render pipeline to RenderEngine in crates/plix-client/src/render/engine.rs
- [ ] T033 [US1] Add render_ui_quads() method to RenderEngine in crates/plix-client/src/render/engine.rs
- [ ] T034 [US1] Verify test quad renders on screen with `cargo run -p plix-client`

### Crosshair Implementation

- [ ] T035 [US1] Define Crosshair struct with render() method in crates/plix-client/src/ui/crosshair.rs
- [ ] T036 [US1] Implement crosshair as 2 white rectangles (horizontal + vertical) in crates/plix-client/src/ui/crosshair.rs
- [ ] T037 [US1] Add crosshair: Crosshair field to GameState in crates/plix-client/src/main.rs
- [ ] T038 [US1] Render crosshair only when ui_state == InGame in crates/plix-client/src/main.rs
- [ ] T039 [US1] Verify crosshair stays centered on window resize in crates/plix-client/src/main.rs

**Checkpoint**: Crosshair visible during gameplay, hidden when paused

---

## Phase 4: User Story 2 - Pause Menu Navigation (Priority: P1) 🎯 MVP

**Goal**: ESC toggles pause menu with Resume/Settings/Quit options, cursor released, inputs blocked, network maintained.

**Independent Test**: Press ESC during gameplay, verify menu appears, mouse is released, inputs are blocked, network stays connected. Press Resume or ESC again to return to gameplay.

### Menu State Handling

- [ ] T040 [US2] Handle ESC key to toggle ui_state between InGame and PauseMenu in crates/plix-client/src/main.rs
- [ ] T041 [US2] Update window title to show current menu state in crates/plix-client/src/main.rs

### Pause Menu Structure

- [ ] T042 [US2] Define PauseMenuItem enum (Resume, Settings, Quit) in crates/plix-client/src/ui/menu.rs
- [ ] T043 [US2] Define PauseMenu struct with selected item and navigation methods in crates/plix-client/src/ui/menu.rs
- [ ] T044 [US2] Add pause_menu: PauseMenu field to GameState in crates/plix-client/src/main.rs

### Menu Rendering

- [ ] T045 [US2] Render pause menu as colored rectangles (highlight selected) in crates/plix-client/src/ui/menu.rs
- [ ] T046 [US2] Call pause menu render when ui_state == PauseMenu in crates/plix-client/src/main.rs

### Menu Navigation

- [ ] T047 [US2] Handle Up/Down arrows to navigate menu items in crates/plix-client/src/main.rs
- [ ] T048 [US2] Handle Enter key to activate selected menu item in crates/plix-client/src/main.rs
- [ ] T049 [US2] Implement Resume action (set ui_state to InGame) in crates/plix-client/src/main.rs
- [ ] T050 [US2] Implement Settings action (set ui_state to Settings) in crates/plix-client/src/main.rs
- [ ] T051 [US2] Implement Quit action (exit event loop cleanly, no panic) in crates/plix-client/src/main.rs

### Network Maintenance

- [ ] T052 [US2] Verify network processing continues while paused (snapshot handling) in crates/plix-client/src/main.rs

**Checkpoint**: Pause menu works, Resume/Settings/Quit functional, network maintained

---

## Phase 5: User Story 3 - Mouse Sensitivity Setting (Priority: P2)

**Goal**: Adjust mouse sensitivity with immediate feedback and persistence.

**Independent Test**: Open Settings, adjust sensitivity, verify mouse look speed changes immediately in-game.

### Settings Menu Framework

- [ ] T053 [US3] Define SettingsMenuItem enum (Sensitivity, FOV, Fullscreen, Audio, Keybinds, Back) in crates/plix-client/src/ui/menu.rs
- [ ] T054 [US3] Define SettingsMenu struct with selected item and navigation in crates/plix-client/src/ui/menu.rs
- [ ] T055 [US3] Add settings_menu: SettingsMenu field to GameState in crates/plix-client/src/main.rs
- [ ] T056 [US3] Render settings menu when ui_state == Settings in crates/plix-client/src/main.rs
- [ ] T057 [US3] Handle Back action (return to PauseMenu) in crates/plix-client/src/main.rs
- [ ] T058 [US3] Handle ESC in Settings to return to PauseMenu in crates/plix-client/src/main.rs

### Sensitivity Control

- [ ] T059 [US3] Handle Left/Right arrows to adjust sensitivity when Sensitivity selected in crates/plix-client/src/main.rs
- [ ] T060 [US3] Apply sensitivity change to InputManager immediately in crates/plix-client/src/main.rs
- [ ] T061 [US3] Save config after sensitivity change in crates/plix-client/src/main.rs
- [ ] T062 [US3] Apply loaded sensitivity to InputManager on startup in crates/plix-client/src/main.rs
- [ ] T063 [US3] Show sensitivity value in window title when Sensitivity selected in crates/plix-client/src/main.rs

**Checkpoint**: Sensitivity adjustable, applies immediately, persists

---

## Phase 6: User Story 4 - Field of View Setting (Priority: P2)

**Goal**: Adjust FOV with immediate feedback and persistence.

**Independent Test**: Open Settings, adjust FOV slider, verify view angle changes immediately.

### Camera FOV Support

- [ ] T064 [US4] Add set_fov(degrees: f32) method to Camera in crates/plix-client/src/render/camera.rs
- [ ] T065 [US4] Apply loaded FOV to Camera on startup in crates/plix-client/src/main.rs

### FOV Control

- [ ] T066 [US4] Handle Left/Right arrows to adjust FOV when FOV selected in crates/plix-client/src/main.rs
- [ ] T067 [US4] Apply FOV change to Camera immediately in crates/plix-client/src/main.rs
- [ ] T068 [US4] Save config after FOV change in crates/plix-client/src/main.rs
- [ ] T069 [US4] Show FOV value in window title when FOV selected in crates/plix-client/src/main.rs

**Checkpoint**: FOV adjustable, applies immediately, persists

---

## Phase 7: User Story 5 - Fullscreen Toggle (Priority: P2)

**Goal**: Toggle between windowed and fullscreen mode with persistence.

**Independent Test**: Open Settings, toggle fullscreen option, verify window mode changes.

### Fullscreen Implementation

- [ ] T070 [US5] Add apply_fullscreen() method using winit set_fullscreen() in crates/plix-client/src/main.rs
- [ ] T071 [US5] Handle Enter key to toggle fullscreen when Fullscreen selected in crates/plix-client/src/main.rs
- [ ] T072 [US5] Save config after fullscreen change in crates/plix-client/src/main.rs
- [ ] T073 [US5] Apply loaded fullscreen setting on startup in crates/plix-client/src/main.rs
- [ ] T074 [US5] Show fullscreen state in window title when Fullscreen selected in crates/plix-client/src/main.rs

**Checkpoint**: Fullscreen toggles, persists

---

## Phase 8: User Story 6 - Keybind Customization (Priority: P3)

**Goal**: Rebind controls with conflict detection and swap confirmation.

**Independent Test**: Open Settings, select a keybind to change, press a new key, verify the action now uses the new key.

### Keybind Infrastructure

- [ ] T075 [US6] Create Key::from_keycode() conversion from winit KeyCode in crates/plix-client/src/config.rs
- [ ] T076 [US6] Add action_for_key() method to Keybinds in crates/plix-client/src/config.rs
- [ ] T077 [US6] Add swap() method to Keybinds for conflict resolution in crates/plix-client/src/config.rs

### Input System Integration

- [ ] T078 [US6] Refactor InputManager to use Keybinds for key -> action mapping in crates/plix-client/src/input.rs
- [ ] T079 [US6] Apply loaded keybinds to InputManager on startup in crates/plix-client/src/main.rs

### Keybinds Menu

- [ ] T080 [US6] Define KeybindsMenuItem for each rebindable action in crates/plix-client/src/ui/menu.rs
- [ ] T081 [US6] Define KeybindsMenu struct with action list and navigation in crates/plix-client/src/ui/menu.rs
- [ ] T082 [US6] Add keybinds_menu: KeybindsMenu field to GameState in crates/plix-client/src/main.rs
- [ ] T083 [US6] Render keybinds menu showing action -> key mappings in crates/plix-client/src/ui/menu.rs
- [ ] T084 [US6] Navigate to keybinds menu when Keybinds selected in Settings in crates/plix-client/src/main.rs

### Rebind Flow

- [ ] T085 [US6] Handle Enter on action to enter Rebinding(Action) state in crates/plix-client/src/main.rs
- [ ] T086 [US6] Show "Press key..." indicator in window title when Rebinding in crates/plix-client/src/main.rs
- [ ] T087 [US6] Capture next key press in Rebinding state in crates/plix-client/src/main.rs
- [ ] T088 [US6] Detect conflict when binding key already in use in crates/plix-client/src/main.rs
- [ ] T089 [US6] Enter ConfirmSwap state when conflict detected in crates/plix-client/src/main.rs
- [ ] T090 [US6] Show swap confirmation in window title when ConfirmSwap in crates/plix-client/src/main.rs
- [ ] T091 [US6] Handle Enter to confirm swap, ESC to cancel in crates/plix-client/src/main.rs
- [ ] T092 [US6] Execute swap using Keybinds::swap() method in crates/plix-client/src/main.rs
- [ ] T093 [US6] Save config after keybind change in crates/plix-client/src/main.rs
- [ ] T094 [US6] Handle ESC in Rebinding state to cancel in crates/plix-client/src/main.rs

**Checkpoint**: Keybinds rebindable with conflict swap, persists

---

## Phase 9: User Story 7 - Audio Mute Toggle (Priority: P3)

**Goal**: Toggle audio mute setting with persistence (placeholder until audio system exists).

**Independent Test**: Open Settings, toggle audio on/off, verify setting persists.

### Audio Toggle

- [ ] T095 [US7] Handle Enter key to toggle audio_muted when Audio selected in crates/plix-client/src/main.rs
- [ ] T096 [US7] Save config after audio toggle in crates/plix-client/src/main.rs
- [ ] T097 [US7] Show audio state in window title when Audio selected in crates/plix-client/src/main.rs

**Checkpoint**: Audio mute toggle works and persists

---

## Phase 10: Headless Compatibility & Non-Regression

**Purpose**: Ensure headless mode works and no regressions introduced

### Headless Bypass

- [ ] T098 Verify --headless mode skips all UI initialization in crates/plix-client/src/main.rs
- [ ] T099 Manual test: `cargo run -p plix-client -- --headless --server 127.0.0.1:7777` connects successfully

### Non-Regression Tests

- [ ] T100 Run `cargo test --workspace` and verify all tests pass
- [ ] T101 Run `cargo clippy --workspace` and fix any warnings
- [ ] T102 Run `cargo fmt --check` and fix any formatting issues

### Manual E2E Validation

- [ ] T103 Manual test: Crosshair shows in-game, hides in menu
- [ ] T104 Manual test: ESC pause/resume blocks gameplay input
- [ ] T105 Manual test: Settings apply live and persist after restart
- [ ] T106 Manual test: Keybind conflict triggers warn + confirm swap
- [ ] T107 Manual test: Fullscreen toggles without crash
- [ ] T108 Manual test: Audio toggle persists
- [ ] T109 Manual test: Config file created at expected location (~/.config/plix/config.toml)

### Optional Cleanup

- [ ] T110 [P] Run `cargo fix --workspace` to reduce warnings
- [ ] T111 [P] Add doc comments to public API in config.rs

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - can start immediately
- **Phase 2 (Foundational)**: Depends on Phase 1 - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Phase 2 completion
  - US1 and US2 are both P1 but share overlay infrastructure; implement sequentially
  - US3, US4, US5 share settings menu; implement US3 first (creates framework), then US4/US5 in parallel
  - US6 (keybinds) depends on settings menu from US3
  - US7 (audio) depends on settings menu from US3
- **Phase 10 (Non-Regression)**: Depends on all user stories complete

### User Story Dependencies

- **US1 (Crosshair)**: Depends on Phase 2 - Creates overlay rendering infrastructure
- **US2 (Pause Menu)**: Depends on Phase 2 - Creates menu state machine
- **US3 (Sensitivity)**: Depends on US2 - Creates settings menu framework
- **US4 (FOV)**: Depends on US3 - Uses settings menu framework
- **US5 (Fullscreen)**: Depends on US3 - Uses settings menu framework
- **US6 (Keybinds)**: Depends on US3 - Uses settings menu framework, complex rebind flow
- **US7 (Audio)**: Depends on US3 - Uses settings menu framework

### Within Each User Story

- Models/types before rendering
- Rendering before integration
- Integration before persistence
- Manual verification at each checkpoint

### Parallel Opportunities

**Phase 1** (all independent setup):
```
T001 || T002 || T003 || T004 || T005
```

**Phase 2** (independent types):
```
T016 (Action enum) || T017 (Key enum)
T025 || T026 || T027 (config tests)
```

**Phase 3** (independent shader/struct):
```
T031 (shader) || T035 (Crosshair struct)
```

**US4 and US5** (after US3 settings framework):
```
Phase 6 (US4 - FOV) || Phase 7 (US5 - Fullscreen)
```

**Phase 10** (independent cleanup):
```
T110 || T111
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (UI state + config infrastructure)
3. Complete Phase 3: US1 - Crosshair Display
4. Complete Phase 4: US2 - Pause Menu Navigation
5. **STOP and VALIDATE**: Test crosshair + pause menu work
6. This is a deployable MVP - game is playable with basic controls

### Full Feature

1. Complete MVP (above)
2. Add Phase 5: US3 - Mouse Sensitivity (creates settings framework)
3. Add Phase 6: US4 - FOV Setting
4. Add Phase 7: US5 - Fullscreen Toggle
5. Add Phase 8: US6 - Keybind Customization
6. Add Phase 9: US7 - Audio Mute Toggle
7. Complete Phase 10: Non-regression validation

### Task Count by Phase

| Phase | Story | Task Count |
|-------|-------|------------|
| Phase 1 | Setup | 8 |
| Phase 2 | Foundational | 22 |
| Phase 3 | US1 - Crosshair | 9 |
| Phase 4 | US2 - Pause Menu | 13 |
| Phase 5 | US3 - Sensitivity | 11 |
| Phase 6 | US4 - FOV | 6 |
| Phase 7 | US5 - Fullscreen | 5 |
| Phase 8 | US6 - Keybinds | 20 |
| Phase 9 | US7 - Audio | 3 |
| Phase 10 | Non-Regression | 14 |
| **Total** | | **111** |

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story
- Each user story can be tested independently at its checkpoint
- Commit after each task or logical group
- Stop at any checkpoint to validate independently
- MVP = US1 + US2 (crosshair + pause menu for playable game)
- Window title used for menu feedback (text rendering out of scope)
- Headless mode MUST continue working (FR-032)
