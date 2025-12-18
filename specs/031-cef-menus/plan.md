# Implementation Plan: CEF Menus (Main Menu / Settings / Server Browser)

**Branch**: `031-cef-menus` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/031-cef-menus/spec.md`

## Summary

Implement HTML/CSS/JS menus (main menu, settings, server browser) rendered via the optional CEF shell (Feature 030). The UI uses a bidirectional JS↔Rust message bridge for all game interactions, with client-side filtering and server-side data fetching. Favorites persist locally and share with native UI. Native UI fallback (Feature 005) activates automatically when CEF is unavailable.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution) + HTML5/CSS3/ES6 JavaScript
**Primary Dependencies**: plix-client (rendering, config, server_browser), plix-common (types, protocol), serde_json (bridge serialization), Feature 030 (CEF shell), Feature 026 (server browser), Feature 025 (account identity)
**Storage**: ~/.config/plix/favorites.toml (local file, TOML format, shared with native UI)
**Testing**: cargo test for Rust bridge logic, manual testing for UI interactions
**Target Platform**: Linux (primary), with CEF binaries bundled
**Project Type**: Client-side UI extension within plix-client crate
**Performance Goals**: 60fps during UI interactions, <2s screen transitions, <3s server list refresh
**Constraints**: No external network from JS, local assets only, string limits (server name 64, search 32, display name 32)
**Scale/Scope**: 3 screens (main, settings, servers), ~10 bridge message types, ~100-500 servers in browser list

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | ✅ PASS | JS cannot access network (FR-012), string sanitization (FR-013), typed message bridge (FR-003) |
| II. Performance | ✅ PASS | Async bridge (NFR-001), no per-frame updates (NFR-002), 60fps target (SC-007) |
| III. Architecture | ✅ PASS | UI layer decoupled from game logic, uses engine primitives via bridge |
| IV. Modding | N/A | Not a mod feature, but UI assets could be extensible later |
| V. Code Quality | ✅ PASS | Structured code (NFR-003), explicit error handling (FR-014) |
| VI. Technical Standards | ✅ PASS | Stable Rust, documented protocol (bridge messages), explicit serialization (JSON) |
| VII. Player Experience | ✅ PASS | Server browser integrated (FR-007), no manual config required (SC-005) |
| VIII. Open Source | ✅ PASS | Local assets, no proprietary dependencies |
| IX. Scoping | ✅ PASS | Minimal scope (3 screens), builds on existing features (030, 026, 025) |
| X. Long-Term Vision | ✅ PASS | Versioned bridge (FR-003), fallback preserves stability |

**Gate Result**: PASS - No violations. Feature aligns with all applicable constitution principles.

## Project Structure

### Documentation (this feature)

```text
specs/031-cef-menus/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (bridge message schemas)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-client/
├── src/
│   ├── ui_cef/
│   │   ├── mod.rs           # Existing CEF shell (Feature 030)
│   │   ├── config.rs        # Existing CEF config
│   │   ├── input.rs         # Existing input focus
│   │   ├── bridge/          # NEW: JS↔Rust bridge
│   │   │   ├── mod.rs       # Bridge dispatcher
│   │   │   ├── messages.rs  # Message types
│   │   │   ├── handlers.rs  # Request handlers
│   │   │   └── serialize.rs # JSON serialization
│   │   └── menus/           # NEW: Menu-specific handlers
│   │       ├── mod.rs
│   │       ├── config.rs    # GetConfig/SetConfig handlers
│   │       ├── servers.rs   # FetchServers/Connect handlers
│   │       └── favorites.rs # Favorites persistence
│   ├── config.rs            # Existing GameConfig
│   └── server_browser/      # Existing Feature 026
└── tests/
    └── ui_cef/
        ├── bridge_test.rs   # Bridge message routing
        └── favorites_test.rs # Favorites persistence

assets/ui/
├── index.html               # App shell + router
├── app.js                   # Main application logic
├── styles.css               # Global styles
├── pages/
│   ├── main.js              # Main menu page
│   ├── settings.js          # Settings page
│   └── servers.js           # Server browser page
└── components/
    ├── button.js            # Reusable button component
    ├── list.js              # Server list component
    └── modal.js             # Error/loading modal
```

**Structure Decision**: Extends existing plix-client crate with new bridge/ and menus/ submodules under ui_cef/. UI assets placed in assets/ui/ following Feature 030 conventions.

## Complexity Tracking

> No violations to justify. Feature uses existing infrastructure (Feature 030 CEF shell, Feature 026 server browser, Feature 025 identity) and adds minimal new code.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none)    | -          | -                                   |
