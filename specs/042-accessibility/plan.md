# Implementation Plan: Accessibility

**Branch**: `042-accessibility` | **Date**: 2025-12-19 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/042-accessibility/spec.md`

## Summary

Implement comprehensive accessibility features for the Plix client: keybinding remapping with conflict detection and 5-second capture timeout, visual accessibility options (UI scale 75-150%, FOV slider, high contrast mode, colorblind presets via CSS SVG filters), and a subtitle system for audio events with max 3-line queue. Extends existing `GameConfig` with `AccessibilityConfig` struct, leverages existing CEF UI (Feature 030+) with native console fallback.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: serde, toml (existing), winit (existing key capture), CEF (Feature 030+)
**Storage**: `~/.config/plix/config.toml` (extends existing GameConfig)
**Testing**: `cargo test` for unit tests, manual visual testing for CSS filters
**Target Platform**: Windows, Linux, macOS (client-side feature)
**Project Type**: Multi-crate workspace (plix-client, plix-common)
**Performance Goals**: <5% framerate degradation from any accessibility feature (SC-010)
**Constraints**: No new crate dependencies, must work with CEF disabled (native fallback)
**Scale/Scope**: 10 rebindable actions, 4 colorblind presets, 8 audio event types

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | PASS | Client-only feature, no server state changes, all validation local |
| II. Performance | PASS | CSS filters are GPU-accelerated, <5% overhead target in SC-010 |
| III. Architecture | PASS | Extends existing config.rs, no new architectural patterns |
| IV. Modding | N/A | Accessibility settings not exposed to mod API in v1 |
| V. Code Quality | PASS | Explicit types, validation with clamp(), unit tests required |
| VI. Technical Standards | PASS | Stable Rust, extends existing TOML serialization |
| VII. Player Experience | PASS | Improves accessibility for wider player base |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping | PASS | MVP scope: keybinds + visual + subtitles foundation |
| X. Long-Term Vision | PASS | Extensible design (add actions, presets, audio events later) |

**Gate Result**: PASS - No violations. Proceed with implementation.

## Project Structure

### Documentation (this feature)

```text
specs/042-accessibility/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Technical decisions (already created)
├── data-model.md        # Type definitions (already created)
├── quickstart.md        # Quick validation guide
├── contracts/           # Bridge message contracts
│   └── bridge-messages.md
└── checklists/          # QA checklists (already created)
    ├── requirements.md
    ├── testing.md
    ├── ux.md
    └── security.md
```

### Source Code (repository root)

```text
crates/plix-client/
├── src/
│   ├── config.rs                    # MODIFY: Add AccessibilityConfig to GameConfig
│   ├── accessibility/               # NEW: Accessibility module
│   │   ├── mod.rs                   # Module exports
│   │   ├── config.rs                # AccessibilityConfig, ColorblindPreset, SubtitleConfig
│   │   ├── keybind_capture.rs       # KeybindCaptureState with 5s timeout
│   │   ├── subtitle_queue.rs        # SubtitleQueue (max 3, drop oldest)
│   │   └── audio_events.rs          # AudioEvent enum
│   ├── ui_cef/
│   │   ├── bridge/
│   │   │   └── messages.rs          # MODIFY: Add accessibility bridge messages
│   │   └── menus/
│   │       └── settings.rs          # MODIFY: Add accessibility settings handlers
│   └── console/
│       └── commands.rs              # MODIFY: Add /rebind, /ui_scale, /colorblind commands
└── tests/
    └── accessibility_test.rs        # NEW: Unit tests

assets/ui/
├── menus/
│   └── settings/
│       ├── controls.html            # NEW: Keybinding rebind UI
│       ├── display.html             # MODIFY: Add visual accessibility options
│       └── audio.html               # MODIFY: Add subtitle settings
├── ingame/
│   └── subtitles.html               # NEW: Subtitle overlay component
└── css/
    ├── accessibility.css            # NEW: High contrast, colorblind filters
    └── colorblind-filters.svg       # NEW: SVG filter definitions
```

**Structure Decision**: Extends existing plix-client crate with new `accessibility/` module. UI assets follow existing Feature 032 patterns for ingame overlays.

## Complexity Tracking

> No Constitution Check violations requiring justification.

| Area | Complexity | Justification |
|------|------------|---------------|
| Keybind capture | Low | Reuses existing Key/Action enums, adds timeout state machine |
| CSS filters | Low | Standard SVG feColorMatrix, GPU-accelerated |
| Subtitle queue | Low | Simple VecDeque with max capacity |
| Bridge messages | Low | Extends existing message pattern from Feature 032 |

## Implementation Phases

### Phase Overview

| Phase | Focus | Deliverables |
|-------|-------|--------------|
| 0 | Research | research.md (COMPLETE) |
| 1 | Design | data-model.md, contracts/, quickstart.md |
| 2 | Tasks | tasks.md (via /speckit.tasks) |

### Phase 0: Research (COMPLETE)

Research completed during `/speckit.specify`. See [research.md](research.md) for:
- 7 technical decisions documented
- Existing code integration points identified
- No new dependencies required
- Performance considerations validated

### Phase 1: Design (Current)

**Deliverables**:
1. **data-model.md** - COMPLETE (created during specify)
2. **contracts/bridge-messages.md** - Bridge message contracts
3. **quickstart.md** - Quick validation scenarios

### Phase 2: Tasks

Generated via `/speckit.tasks` command after plan approval.

## Dependencies

### Internal Dependencies (plix crates)

| Dependency | Used For | Status |
|------------|----------|--------|
| plix-client/config.rs | GameConfig, Keybinds, Action, Key | Exists - extend |
| plix-client/ui_cef/bridge | CEF <-> Rust messaging | Exists - extend |
| plix-client/render/camera.rs | FOV live update | Exists - integrate |
| plix-client/console | Console commands | Exists - extend |

### External Dependencies

| Dependency | Version | Purpose | Status |
|------------|---------|---------|--------|
| serde | existing | Serialization | No change |
| toml | existing | Config persistence | No change |
| winit | existing | Key capture events | No change |
| CEF | Feature 030 | UI rendering | No change |

**No new crate dependencies required.**

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| CSS filter perf on low-end GPU | Low | Medium | SC-010 benchmark, disable option |
| Key capture edge cases | Low | Low | 5s timeout prevents stuck states |
| Config migration | Low | Low | serde defaults handle missing fields |
| CEF unavailable | Low | Medium | Native console fallback required |

## Definition of Done

- [ ] All 10 Action variants rebindable via Settings > Controls
- [ ] Keybind conflicts detected with Swap/Cancel resolution
- [ ] 5-second capture timeout implemented
- [ ] UI Scale slider (75-150%) with live preview
- [ ] FOV slider (60-110) with live camera update
- [ ] High Contrast mode toggle functional
- [ ] 4 colorblind presets apply distinct CSS filters
- [ ] Subtitle queue (max 3) with auto-dismiss
- [ ] All settings persist to config.toml
- [ ] Native fallback console commands work
- [ ] Unit tests pass
- [ ] No >5% framerate degradation
