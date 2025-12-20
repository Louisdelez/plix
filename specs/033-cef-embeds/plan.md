# Implementation Plan: CEF Media Embeds (YouTube / Twitch / Spotify)

**Branch**: `033-cef-embeds` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/033-cef-embeds/spec.md`

## Summary

Implement a secure, optional CEF-based media embed panel allowing players to watch YouTube videos and Twitch streams while in-game. The panel operates as a controlled overlay with strict domain whitelisting, rate limiting, and proper input focus management. Spotify support is stubbed for future implementation.

Key technical approach:
- Extend existing CEF bridge protocol (Feature 031) with embed-specific message types
- Add new `EmbedFocus` state to input focus controller (alongside existing `ChatTyping`)
- Rust-side URL normalization and validation before any CEF navigation
- CEF navigation guard intercepts all frame navigations, blocking non-whitelisted domains
- Single embed slot MVP; multi-slot deferred

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-client (ui_cef module), plix-common (types), CEF (via existing Feature 030 shell), serde_json (bridge serialization), tracing (logging)
**Storage**: N/A (in-memory state only, config persisted via existing TOML config system)
**Testing**: `cargo test` (unit tests), manual validation for CEF integration
**Target Platform**: Linux (primary), Windows (cross-platform CEF)
**Project Type**: Multi-crate workspace (existing plix structure)
**Performance Goals**: Zero frame rate impact when hidden; <5% impact when visible but unfocused; <100ms toggle response
**Constraints**: Strict domain whitelist, 1 action/2s rate limit, no stuck focus states
**Scale/Scope**: Single embed slot MVP, client-side only feature

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security | ✅ PASS | Domain whitelist enforced Rust-side; no eval/injection; sandboxed CEF |
| II. Performance | ✅ PASS | Lazy loading; no updates when hidden; throttled when visible |
| III. Architecture | ✅ PASS | Builds on existing CEF shell (Feature 030) and bridge (Feature 031) |
| IV. Modding | ⚪ N/A | Pure UI feature, no mod API exposed |
| V. Code Quality | ✅ PASS | Explicit error codes; structured logging; testable validation |
| VI. Technical Standards | ✅ PASS | Stable Rust; versioned protocol; explicit serialization |
| VII. Player Experience | ✅ PASS | Non-intrusive; optional; proper focus management |
| VIII. Open Source | ✅ PASS | No proprietary dependencies; YouTube/Twitch are public embeds |
| IX. Scoping | ✅ PASS | MVP single slot; Spotify deferred; no feature creep |
| X. Long-Term Vision | ✅ PASS | Extensible provider system; versioned bridge |

**Gate Result**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/033-cef-embeds/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (bridge-messages.md)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/plix-client/src/
├── ui_cef/
│   ├── mod.rs                    # Export embeds module
│   ├── config.rs                 # Add embed config fields
│   ├── input.rs                  # Add EmbedFocus state
│   ├── bridge/
│   │   ├── mod.rs                # Route embed messages
│   │   ├── messages.rs           # Add embed MessageType/PushType/errors
│   │   ├── handlers.rs           # Add embed message handlers
│   │   └── serialize.rs          # Add embed serialization helpers
│   └── embeds/                   # NEW MODULE
│       ├── mod.rs                # EmbedsManager coordinator
│       ├── config.rs             # EmbedConfig struct
│       ├── provider.rs           # EmbedProvider enum, whitelist
│       ├── normalizer.rs         # URL normalization/validation
│       └── navigation_guard.rs   # CEF navigation intercept hook

assets/ui/
├── embeds/                       # NEW DIRECTORY
│   ├── embeds.html               # Panel HTML structure
│   ├── embeds.css                # Panel styles
│   └── embeds.js                 # Bridge integration, iframe management
```

**Structure Decision**: Follows existing ui_cef module pattern. New `embeds/` submodule parallels `ingame/` (Feature 032) and `menus/` (Feature 031). UI assets in dedicated `assets/ui/embeds/` directory.

## Complexity Tracking

> No violations requiring justification. Table intentionally empty.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| (none) | | |
