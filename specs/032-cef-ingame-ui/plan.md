# Implementation Plan: CEF In-Game UI (HUD, Chat, Scoreboard)

**Branch**: `032-cef-ingame-ui` | **Date**: 2025-12-18 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/032-cef-ingame-ui/spec.md`

## Summary

Implement in-game UI overlays rendered via CEF: optional web HUD (HP, RTT, FPS, crosshair), text chat with toast notifications, and hold-to-show scoreboard. Extends the Feature 030 CEF shell and Feature 031 bridge protocol with new message types for real-time gameplay data. Provides native UI fallback when CEF is unavailable.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: wgpu (rendering), winit (input), serde_json (bridge serialization), plix-common (types, protocol), plix-client (existing CEF shell, UI)
**Storage**: N/A (in-memory state only - chat history, scoreboard cache)
**Testing**: cargo test for unit/integration tests
**Target Platform**: Linux/Windows desktop (wgpu backends)
**Project Type**: Multi-crate workspace (plix-client primary, plix-common for protocol)
**Performance Goals**: 60fps minimum with overlay active, HUD updates at 10-20 Hz, scoreboard/chat event-driven
**Constraints**: Chat message ≤200 chars, history ≤100 messages, scoreboard ≤64 players, client rate limit 1 msg/500ms
**Scale/Scope**: Single-player to 64-player matches

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ Pass | Server authoritative for chat distribution; client validates locally but server enforces rate limits and anti-spam |
| II. Performance (Low Latency) | ✅ Pass | HUD throttled to 10-20 Hz; scoreboard updates only when visible; chat event-driven |
| III. Architecture (Engine-First) | ✅ Pass | Extends existing CEF shell (F030) and bridge (F031); no new engine primitives required |
| IV. Modding (Extensibility) | ✅ Pass | UI is HTML/CSS/JS, separable from engine; bridge messages are typed and versioned |
| V. Code Quality (Explicit & Tested) | ✅ Pass | All input focus states explicit; message validation tested |
| VI. Technical Standards (Rust) | ✅ Pass | Stable Rust only; extends existing serialization patterns |
| VII. Player Experience (Multiplayer-First) | ✅ Pass | Chat and scoreboard are multiplayer essentials; native fallback ensures accessibility |
| VIII. Open Source | ✅ Pass | No proprietary dependencies |
| IX. Scoping (Minimal Viable) | ✅ Pass | Core features only; no rich text, no whispers, no moderation UI in v1 |
| X. Long-Term Vision | ✅ Pass | Bridge versioning supports evolution; clean separation of concerns |

**Pre-Design Gate**: ✅ PASSED

## Project Structure

### Documentation (this feature)

```text
specs/032-cef-ingame-ui/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (bridge message schemas)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/plix-common/src/
├── protocol/
│   └── messages.rs      # EXTEND: ChatMessage, ChatSend, ScoreboardRequest
├── chat.rs              # NEW: Chat types (ChatMessageKind, etc.)
└── lib.rs               # MODIFY: pub mod chat

crates/plix-client/src/
├── ui_cef/
│   ├── mod.rs           # MODIFY: Add ingame overlay support
│   ├── bridge/
│   │   ├── messages.rs  # EXTEND: New message types for in-game UI
│   │   └── handlers.rs  # EXTEND: Handlers for chat/hud/scoreboard
│   └── ingame/          # NEW: In-game overlay module
│       ├── mod.rs       # In-game overlay coordinator
│       ├── hud.rs       # HUD state publisher
│       ├── chat.rs      # Chat client (local history, commands)
│       └── scoreboard.rs # Scoreboard client
├── ui/
│   ├── mod.rs           # MODIFY: Export native fallback components
│   ├── chat_native.rs   # NEW: Native chat fallback
│   └── scoreboard_native.rs # NEW: Native scoreboard fallback
├── input.rs             # EXTEND: Add ChatTyping focus state
└── main.rs              # MODIFY: Integrate ingame overlay

assets/ui/ingame/        # NEW: Web UI assets
├── overlay.html         # Combined overlay (HUD + Chat + Scoreboard)
├── overlay.css          # Styles
└── overlay.js           # Bridge integration + components

tests/
├── chat_tests.rs        # Chat message handling, history, commands
├── input_focus_tests.rs # Focus state transitions
└── bridge_ingame_tests.rs # In-game bridge message tests
```

**Structure Decision**: Extends existing multi-crate workspace. New modules added to plix-client/ui_cef for CEF overlay and plix-client/ui for native fallback. Protocol extensions in plix-common/protocol.

## Complexity Tracking

No violations requiring justification. The feature:
- Extends existing modules (no new crates)
- Uses established patterns (bridge messaging, input focus)
- Adds minimal new concepts (chat history, throttled publishing)

## Post-Design Constitution Re-Check

*Gate re-evaluation after Phase 1 design completion.*

| Principle | Status | Design Validation |
|-----------|--------|-------------------|
| I. Security | ✅ Pass | Chat validation at both client (200 char, rate limit) and server (distribution authority); no eval/remote code in bridge |
| II. Performance | ✅ Pass | HudStatePublisher throttles to 15 Hz with HP-change immediate publish; ScoreboardClient updates only when visible |
| III. Architecture | ✅ Pass | Clean extension of existing modules; InputFocus state machine explicit with 3 states |
| IV. Modding | ✅ Pass | Bridge messages documented in contracts/; UI is pure HTML/CSS/JS |
| V. Code Quality | ✅ Pass | All entities have validation rules; state transitions documented |
| VI. Technical Standards | ✅ Pass | Protocol extensions follow existing patterns; serde_json for bridge |
| VII. Player Experience | ✅ Pass | Native fallback covers all features; keybinds configurable |
| VIII. Open Source | ✅ Pass | No new dependencies; all code open |
| IX. Scoping | ✅ Pass | Minimal entity set; no feature creep beyond spec |
| X. Long-Term Vision | ✅ Pass | Bridge version 1.0 extensible; chat protocol ready for whispers in v2 |

**Post-Design Gate**: ✅ PASSED

## Phase 1 Artifacts

Generated artifacts:
- `research.md` - Design decisions and rationale
- `data-model.md` - Entity definitions and relationships
- `contracts/bridge-messages.md` - Bridge message schemas
- `quickstart.md` - Development setup guide

## Next Steps

Run `/speckit.tasks` to generate implementation tasks from this plan.
