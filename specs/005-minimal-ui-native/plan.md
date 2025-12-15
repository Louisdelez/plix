# Implementation Plan: Minimal Native UI

**Branch**: `005-minimal-ui-native` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-minimal-ui-native/spec.md`

## Summary

Implement a minimal native UI layer (no CEF) to make the game playable and configurable. The UI includes a crosshair overlay, pause menu with Resume/Settings/Quit options, and settings for mouse sensitivity, FOV, fullscreen toggle, keybinds, and audio mute. All settings persist to a TOML config file.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: wgpu (rendering), winit (window management), glam (math), serde + toml (config persistence)
**Storage**: TOML config file at `~/.config/plix/config.toml` (Linux)
**Testing**: `cargo test --workspace`
**Target Platform**: Linux (primary), Windows/macOS (compatible via winit/wgpu)
**Project Type**: Multi-crate workspace (plix-client, plix-server, plix-common, plix-arena)
**Performance Goals**: Settings changes apply within 100ms, menu toggle within frame time
**Constraints**: UI must not block game loop (VII. Player Experience), no panics in production (V. Code Quality)
**Scale/Scope**: 7 user stories, 33 functional requirements

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Compliance | Notes |
|-----------|------------|-------|
| I. Security | PASS | Settings are local-only, no server interaction |
| II. Performance | PASS | UI renders in game loop, event-driven updates |
| III. Architecture | PASS | UI module isolated in `plix-client/src/ui/` |
| V. Code Quality | PASS | Tests for config loading/saving, no panics |
| VI. Technical Standards | PASS | Stable Rust, clippy/fmt compliance |
| VII. Player Experience | PASS | UI responsive, doesn't block game logic |
| IX. Scoping | PASS | Minimal scope - only essential settings |

## Project Structure

### Documentation (this feature)

```text
specs/005-minimal-ui-native/
├── plan.md              # This file
├── research.md          # Phase 0 output (not needed - using existing wgpu)
├── data-model.md        # Config and state data structures
├── quickstart.md        # Development setup notes
├── contracts/           # API contracts
│   ├── config.md        # Config file format specification
│   └── menu-state.md    # Menu state machine specification
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-client/
├── src/
│   ├── config.rs        # NEW: GameConfig, load/save, defaults
│   ├── input.rs         # MODIFY: Add Action enum, keybind system
│   ├── main.rs          # MODIFY: Menu state machine, ESC handling
│   ├── ui/
│   │   ├── mod.rs       # MODIFY: Export new modules
│   │   ├── crosshair.rs # NEW: Crosshair rendering
│   │   ├── menu.rs      # NEW: Pause menu, settings menu
│   │   └── hud.rs       # EXISTING: Combat feedback (unchanged)
│   └── render/
│       ├── camera.rs    # MODIFY: Add set_fov() method
│       └── engine.rs    # MODIFY: Support fullscreen toggle
└── tests/
    └── config_test.rs   # NEW: Config load/save tests
```

**Structure Decision**: Existing multi-crate workspace. UI code concentrated in `plix-client/src/ui/` module with config persistence in dedicated `config.rs`.

## Complexity Tracking

No constitution violations requiring justification.

---

## Phase 0: Research

**Purpose**: Understand existing infrastructure for settings and rendering.

### Current Implementation Analysis

**Input System** (`crates/plix-client/src/input.rs`):
- `Key` enum: W, A, S, D, Space, Ctrl, LeftClick, RightClick (hardcoded)
- `InputManager`: sensitivity field (0.003 default), `set_sensitivity()` exists
- `InputState`: tracks movement, jump, crouch, attack, block actions
- **Gap**: No rebindable keybinds, no Action abstraction

**Camera System** (`crates/plix-client/src/render/camera.rs`):
- `Camera`: fov field (stored as radians), created with `new(fov_degrees, aspect)`
- Default FOV: 70.0 degrees
- **Gap**: No `set_fov()` method, FOV set only at construction

**Window Management** (`crates/plix-client/src/main.rs`):
- Uses winit `Window` with cursor grab/release
- `cursor_grabbed` state controls mouse capture
- ESC releases cursor (doesn't open menu)
- **Gap**: No pause menu, no fullscreen toggle, no settings persistence

**HUD System** (`crates/plix-client/src/ui/hud.rs`):
- `Hud` struct with `visible` flag
- Event queue for combat feedback
- `render()` is placeholder (TODO comment)
- **Gap**: No crosshair, no menu rendering

**Rendering** (`crates/plix-client/src/render/engine.rs`):
- `RenderEngine` with wgpu pipeline
- No 2D overlay/UI rendering capability yet
- **Decision**: Use simple geometry for crosshair (2 quads), text not required for MVP

### Key Technical Decisions

1. **Crosshair Rendering**: Draw 2 thin rectangles at screen center using wgpu. No text/fonts needed.

2. **Menu Rendering**: Use window title for debug feedback (current pattern). For settings values, can update title or use simple colored quads to indicate selection. Text rendering is out of scope.

3. **Menu State Machine**: Add `MenuState` enum to `GameState`:
   - `None` - Playing, cursor grabbed, crosshair visible
   - `Paused` - Pause menu, cursor released, crosshair hidden
   - `Settings` - Settings submenu
   - `KeybindRebind(Action)` - Awaiting key press for rebind

4. **Config Persistence**: TOML file using `serde` + `toml` crate. Load on startup, save on change.

5. **Keybind System**: New `Action` enum for rebindable actions. `Keybinds` struct maps `Action -> Key`. Convert winit `KeyCode` to our `Key` at input layer.

---

## Phase 1: Design Artifacts

### Data Model (see `data-model.md`)

Core structures:
- `GameConfig`: sensitivity, fov_degrees, fullscreen, audio_muted, keybinds
- `Keybinds`: HashMap<Action, Key>
- `Action` enum: Forward, Backward, Left, Right, Jump, Attack, PlaceBlock, RemoveBlock, Pause
- `MenuState` enum: None, Paused, Settings, KeybindRebind(Action)

### Contracts (see `contracts/`)

- `config.md`: TOML file format, default values, validation rules
- `menu-state.md`: State transitions, input handling per state

---

## Phase 2: Implementation Phases

### Phase 2.1 - Config Infrastructure (Foundation)

**Goal**: Config file loading/saving with defaults.

- Create `crates/plix-client/src/config.rs`
- Define `GameConfig` struct with serde derive
- Implement `load()` with fallback to defaults
- Implement `save()` to `~/.config/plix/config.toml`
- Handle missing/corrupted config gracefully
- Add unit tests for load/save roundtrip

### Phase 2.2 - Crosshair Rendering (US1)

**Goal**: Display crosshair at screen center during gameplay.

- Create `crates/plix-client/src/ui/crosshair.rs`
- Implement crosshair as 2 white rectangles (horizontal + vertical)
- Add to render pipeline (after 3D, before UI)
- Control visibility via `MenuState::None`
- Verify crosshair remains centered on window resize

### Phase 2.3 - Pause Menu & State Machine (US2)

**Goal**: ESC toggles pause menu, cursor released, inputs blocked.

- Add `MenuState` enum to `GameState`
- Handle ESC to toggle `MenuState::None <-> Paused`
- In `Paused`: release cursor, hide crosshair, block gameplay inputs
- Add Resume button (keyboard: Enter or ESC to resume)
- Add Settings button (keyboard: S key)
- Add Quit button (keyboard: Q key)
- Verify network connection maintained while paused

### Phase 2.4 - Settings Menu (US3, US4, US5, US7)

**Goal**: Settings for sensitivity, FOV, fullscreen, audio mute.

- Create settings menu state
- **Sensitivity**: Show current value, adjust with left/right arrows
- **FOV**: Show current value (60-110), adjust with left/right
- **Fullscreen**: Toggle with Enter
- **Audio**: Toggle muted/unmuted with Enter
- Apply changes immediately
- Save to config on change
- Back button returns to pause menu

### Phase 2.5 - Keybind System (US6)

**Goal**: Rebindable controls with conflict detection.

- Add `Action` enum for all rebindable actions
- Create `Keybinds` struct with HashMap<Action, Key>
- Extend `Key` enum for all keyboard keys needed
- Update input handling to use keybinds
- Add keybinds settings screen
- Implement rebind mode (wait for key press)
- Implement conflict detection with swap confirmation
- Persist keybinds to config

### Phase 2.6 - Persistence & Integration

**Goal**: Settings persist across restarts, apply on load.

- Load config in `GameState::new()`
- Apply sensitivity to `InputManager`
- Apply FOV to `Camera` (add `set_fov()` method)
- Apply fullscreen to window
- Apply keybinds to input system
- Verify all settings survive restart

### Phase 2.7 - Validation & Non-Regression

**Goal**: Ensure all tests pass, no regressions.

- Run `cargo test --workspace`
- Run `cargo clippy --workspace`
- Run `cargo fmt --check`
- Manual test: crosshair visible/hidden
- Manual test: pause menu navigation
- Manual test: settings changes apply immediately
- Manual test: settings persist after restart
- Manual test: keybind rebinding with conflict swap
- Verify headless server unaffected

---

## Implementation Notes

### Crosshair Rendering Strategy

Without a text/font system, the crosshair will be:
- 2 thin white rectangles at screen center
- Horizontal: 20px wide, 2px tall
- Vertical: 2px wide, 20px tall
- Rendered using wgpu with simple vertex/fragment shaders
- Uses normalized device coordinates (-1 to 1), no projection needed

### Menu Interaction Without Text

Since text rendering is out of scope:
- Use window title to show current menu context
- Use colored rectangles for menu items (highlight = selected)
- Keyboard navigation: Up/Down to select, Enter to activate, ESC to back
- Mouse click on colored areas for selection
- Debug output to console for setting values

### Config File Location

```
Linux:   ~/.config/plix/config.toml
Windows: %APPDATA%\plix\config.toml
macOS:   ~/Library/Application Support/plix/config.toml
```

Use `dirs` crate for cross-platform config directory.

### Keybind Default Values

| Action | Default Key |
|--------|-------------|
| Forward | W |
| Backward | S |
| Left | A |
| Right | D |
| Jump | Space |
| Attack | LeftClick |
| PlaceBlock | RightClick |
| RemoveBlock | LeftClick |
| Pause | Escape |

Note: Attack and RemoveBlock share LMB (context-dependent in future).

---

## Dependencies

### New Crate Dependencies (plix-client)

```toml
[dependencies]
toml = "0.8"
dirs = "5.0"
```

`serde` already in workspace.

### Phase Dependencies

```
Phase 2.1 (Config) ← Phase 2.2 (Crosshair)
                  ← Phase 2.3 (Pause Menu)
                  ← Phase 2.4 (Settings)
                  ← Phase 2.5 (Keybinds)
Phase 2.3 (Pause Menu) ← Phase 2.4 (Settings) ← Phase 2.5 (Keybinds)
All ← Phase 2.6 (Integration) ← Phase 2.7 (Validation)
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| wgpu 2D rendering complexity | Use simple colored quads, avoid text |
| Menu UX without text | Use window title + colored indicators |
| Config directory permissions | Handle errors gracefully, fallback to defaults |
| Keybind conflicts | Warn + swap pattern with visual feedback |
| Performance impact | UI renders only when menu open or crosshair visible |

---

## Success Criteria Mapping

| SC | Requirement | Implementation |
|----|-------------|----------------|
| SC-001 | Crosshair visible within 1s | Render crosshair in first frame |
| SC-002 | Pause/resume within 2s | Single ESC press toggles menu |
| SC-003 | Settings apply <100ms | Direct field assignment, no restart |
| SC-004 | Config persists 100% | TOML save after each change |
| SC-005 | Pause menu 5+ min no disconnect | No network changes in pause state |
| SC-006 | All 9 actions rebindable | Action enum with full keybind UI |
| SC-007 | Fullscreen toggle <2s | winit `set_fullscreen()` call |
| SC-008 | Headless server unaffected | UI code isolated to plix-client |
| SC-009 | All tests pass | cargo test --workspace in validation |
