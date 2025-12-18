# Implementation Plan: CEF UI Shell (Optional)

**Branch**: `030-cef-ui-shell` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/030-cef-ui-shell/spec.md`

## Summary

Implement an optional CEF (Chromium Embedded Framework) integration for rendering HTML/CSS/JS UI as a GPU texture inside the plix game client. This is a technical foundation only - not a full UI system. CEF runs in off-screen rendering mode, outputs RGBA frames to a wgpu texture, and supports click-to-focus input handling. When CEF is unavailable or disabled, the system falls back to Feature 005's native UI.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: CEF (binding TBD via spike), wgpu (existing), winit (existing), clap (CLI), toml/serde (config)
**Storage**: N/A (in-memory state only)
**Testing**: cargo test for unit tests, manual validation for visual/interaction tests
**Target Platform**: Linux x86_64, Windows x86_64 (CEF binaries platform-specific)
**Project Type**: Single crate extension to plix-client
**Performance Goals**: <2ms frame time overhead at 1080p, 60fps texture updates
**Constraints**: CEF subprocess memory <256MB, optional feature flag, graceful fallback
**Scale/Scope**: Single CEF viewport, local HTML files only

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | ✅ PASS | CEF restricted to local files only (FR-005), no external URLs |
| II. Performance | ✅ PASS | <2ms budget specified (NFR-001), async texture upload (FR-021) |
| III. Architecture | ✅ PASS | Clean layer separation - UI Layer decoupled from game logic |
| IV. Modding | ⚠️ N/A | CEF UI is engine feature, not mod system |
| V. Code Quality | ✅ PASS | Mandatory testing, structured logging (FR-030, FR-031) |
| VI. Technical Standards | ✅ PASS | Stable Rust only, cargo clippy/fmt required |
| VII. Player Experience | ✅ PASS | Fallback UI ensures game remains playable (FR-014) |
| VIII. Open Source | ✅ PASS | CEF is BSD-licensed, distributable (NFR-003) |
| IX. Scoping | ✅ PASS | Single viewport MVP, no feature creep |
| X. Long-Term Vision | ✅ PASS | Optional feature, doesn't break existing systems |

**Gate Result**: PASS - No violations. Feature is optional and properly scoped.

## Project Structure

### Documentation (this feature)

```text
specs/030-cef-ui-shell/
├── plan.md              # This file
├── research.md          # Phase 0: CEF binding evaluation
├── data-model.md        # Phase 1: Entity definitions
├── quickstart.md        # Phase 1: Getting started guide
├── contracts/           # Phase 1: Internal API contracts
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (repository root)

```text
crates/plix-client/
├── src/
│   ├── ui/              # Existing native UI (Feature 005)
│   │   ├── mod.rs
│   │   ├── menu.rs
│   │   └── state.rs
│   ├── ui_cef/          # NEW: CEF integration module
│   │   ├── mod.rs       # CefShell, public API
│   │   ├── config.rs    # CefConfig struct
│   │   ├── browser.rs   # CEF browser instance management
│   │   ├── texture.rs   # CefTexture, wgpu integration
│   │   ├── input.rs     # Input routing, focus state
│   │   └── fallback.rs  # Fallback detection logic
│   ├── main.rs          # CLI args (--cef-ui, --no-cef-ui, --cef-devtools)
│   └── lib.rs           # Feature flag: "cef-ui"
└── Cargo.toml           # Optional cef dependency

assets/
└── ui/                  # NEW: HTML/CSS/JS UI assets
    └── index.html       # Initial test page
```

**Structure Decision**: Extend plix-client with new `ui_cef/` module. CEF integration is compile-time optional via Cargo feature flag "cef-ui". Existing `ui/` module remains as fallback.

## Phases Overview

Based on user input and spec requirements:

| Phase | Name | Description |
|-------|------|-------------|
| 1 | Spike & Validation | Evaluate CEF Rust bindings, prove OSR works |
| 2 | Setup & Architecture | Create ui_cef module, CLI flags, config |
| 3 | CEF Initialization | OSR mode, browser instance, local page loading |
| 4 | GPU Texture Rendering | RGBA buffer → wgpu texture pipeline |
| 5 | Render Pipeline Integration | UI layer, quad rendering, z-order |
| 6 | Input & Focus Handling | Click-to-focus, input routing, key mapping |
| 7 | Native UI Fallback | Detection, graceful degradation |
| 8 | Debug & Dev Experience | DevTools, reload, logging |
| 9 | Packaging & Distribution | CEF binaries, launcher compatibility |
| 10 | Tests & Validation | Manual + automated tests |
| 11 | Documentation | User guide, CLI docs, limitations |
| 12 | Polish & Completion | Cleanup, fmt, clippy, logs |

## Phase Details

### Phase 1: Spike & Validation

**Goal**: Evaluate CEF Rust bindings and prove off-screen rendering works

**Research Tasks**:
- Evaluate `cef-rs` crate maturity and API coverage
- Evaluate community FFI wrappers (e.g., `chromium-embedded-rust`)
- Test CEF OSR initialization in isolated spike
- Test paint callback (RGBA buffer extraction)
- Document decision: existing binding vs minimal FFI wrapper

**Output**: Decision document in `research.md`

### Phase 2: Setup & Architecture

**Goal**: Create module structure and configuration

**Tasks**:
- Create `crates/plix-client/src/ui_cef/` module
- Add Cargo feature flag `cef-ui` (optional)
- Add CLI flags: `--cef-ui`, `--no-cef-ui`, `--cef-devtools`
- Add config section `[ui]` with `cef_enabled`, `cef_initial_page`
- Ensure client starts without CEF when disabled

### Phase 3: CEF Initialization (OSR)

**Goal**: Initialize CEF in off-screen rendering mode

**Tasks**:
- Initialize CefApp in windowless mode
- Create single browser instance
- Load initial page (`plix://ui/index.html` or local file)
- Block external URL access (custom scheme handler)
- Handle CEF shutdown on client exit

### Phase 4: GPU Texture Rendering

**Goal**: Render CEF frames to wgpu texture

**Tasks**:
- Implement paint callback to receive RGBA buffer
- Create dynamic wgpu texture for CEF output
- Update texture on each paint callback
- Handle window resize (CEF resize + texture resize)
- Verify no memory leaks

### Phase 5: Render Pipeline Integration

**Goal**: Composite CEF texture over 3D world

**Tasks**:
- Add optional UI layer to renderer
- Render CEF texture as fullscreen quad
- Handle z-order (above world, below debug overlays)
- Allow runtime enable/disable of UI layer
- Support alpha blending for transparent UI

### Phase 6: Input & Focus Handling

**Goal**: Implement click-to-focus input routing

**Tasks**:
- Implement InputFocus state machine (Game | CefUI)
- Click on UI area → focus CEF
- Forward mouse events (position, buttons, scroll) to CEF
- Forward keyboard events (keycodes, modifiers) to CEF
- Escape key → release focus, return to game
- Block game input while CEF has focus

### Phase 7: Native UI Fallback

**Goal**: Graceful degradation when CEF unavailable

**Tasks**:
- Detect CEF unavailable (feature disabled, init failed, runtime crash)
- Fall back to Feature 005 native UI
- Ensure same menus accessible via fallback
- Log fallback activation clearly
- No crashes on CEF failure

### Phase 8: Debug & Dev Experience

**Goal**: Developer tools and debugging support

**Tasks**:
- Add UI reload hotkey (F6)
- Enable CEF DevTools via `--cef-devtools` flag
- Redirect CEF console logs to client log
- Add optional debug overlay (CEF state, focus state)

### Phase 9: Packaging & Distribution

**Goal**: Include CEF binaries in distribution

**Tasks**:
- Bundle CEF binaries with client
- Update manifest for launcher (Feature 029)
- Document size overhead (~100-200MB)
- Verify client works without CEF feature

### Phase 10: Tests & Validation

**Goal**: Verify all functionality

**Manual Tests**:
- Start with/without CEF
- Click focus/unfocus
- Window resize
- UI reload

**Automated Tests**:
- CEF init success/failure paths
- Fallback activation
- Input blocking when focus active

### Phase 11: Documentation

**Goal**: Document feature for users and developers

**Tasks**:
- Document CEF UI shell purpose
- Document CLI flags
- Document config options
- Document known limitations (performance, size)
- Note: optional feature, not required

### Phase 12: Polish & Completion

**Goal**: Final cleanup

**Tasks**:
- Code cleanup
- `cargo fmt --all`
- `cargo clippy --all-targets`
- Clean logs
- Mark feature complete

## Complexity Tracking

No constitution violations to justify. Feature is optional and properly scoped.

## Dependencies

| Dependency | Type | Notes |
|------------|------|-------|
| Feature 005 (minimal-ui-native) | Prerequisite | Provides fallback UI |
| Feature 029 (patch-launcher) | Integration | CEF binaries in manifest |
| wgpu | Existing | GPU texture creation |
| winit | Existing | Input events |
| CEF | New (TBD) | Binding chosen via spike |

## Risks

| Risk | Mitigation |
|------|------------|
| CEF Rust bindings immature | Spike validates before commitment; fallback to FFI wrapper |
| CEF binary size large (~100-200MB) | Document clearly; CEF is optional feature |
| Performance overhead | <2ms budget enforced; pause when minimized |
| CEF crashes | Subprocess isolation; graceful fallback |
