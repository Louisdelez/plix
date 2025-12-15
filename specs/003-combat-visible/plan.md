# Implementation Plan: Server-Authoritative Combat System

**Branch**: `003-combat-visible` | **Date**: 2025-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/003-combat-visible/spec.md`

## Summary

Implement visible PvP combat with server-authoritative hit validation, HP tracking, death/respawn cycle, and minimal client feedback. The existing codebase already has partial combat infrastructure (attack flag in input, combat system skeleton, HUD with health display). This plan focuses on completing the integration and ensuring end-to-end visibility.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only)
**Primary Dependencies**: glam (math), bincode (serialization), wgpu (rendering), tokio (async)
**Storage**: N/A (in-memory state only, no persistence)
**Testing**: cargo test (unit + integration tests in `crates/*/tests/`)
**Target Platform**: Linux server + desktop clients (Linux/Windows/macOS)
**Project Type**: Workspace with 6 crates (plix-common, plix-net, plix-server, plix-client, plix-arena, plix-tools)
**Performance Goals**: 60 Hz server tick rate, <200ms event propagation
**Constraints**: Server-authoritative only, no client-side prediction for combat
**Scale/Scope**: 2-50 concurrent players, single arena

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | All combat validation server-side; client sends only attack flag |
| II. Performance (Low Latency) | PASS | Event-driven combat on tick; no polling; 60 Hz tick stable |
| III. Architecture (Engine-First) | PASS | Combat system as separate module; uses engine primitives |
| IV. Modding (First-Class) | N/A | Combat is core engine, not mod layer |
| V. Code Quality (Tested) | PASS | Existing combat tests; will add more per spec |
| VI. Technical Standards (Stable Rust) | PASS | No nightly features; clippy/fmt compliant |
| VII. Player Experience (Multiplayer-First) | PASS | Combat designed for multiplayer from start |
| VIII. Open Source | PASS | All code public |
| IX. Scoping (Minimal MVP) | PASS | Simple melee only; no weapons/inventory/projectiles |
| X. Long-Term Vision | PASS | Combat system is reusable engine primitive |

**Constitution Compliance**: All gates pass. No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/003-combat-visible/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (protocol messages)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── protocol/
│   │   └── messages.rs      # Protocol messages (add combat events)
│   └── types.rs             # Shared types (already has PlayerId, etc.)
│
├── plix-server/src/
│   ├── sim/
│   │   ├── combat.rs        # Combat system (EXTEND - add events)
│   │   ├── movement.rs      # Movement system (existing)
│   │   └── collision.rs     # Collision detection (existing)
│   ├── session.rs           # Player state (has health, is_dead, respawn_tick)
│   ├── validation.rs        # Input validation (existing)
│   ├── tick.rs              # Tick loop (integrate combat)
│   ├── match_state.rs       # Match phases (existing)
│   └── replication/
│       └── events.rs        # Event buffer (has PlayerDied, PlayerRespawned)
│
├── plix-client/src/
│   ├── input.rs             # Input capture (attack on LMB - existing)
│   ├── ui/
│   │   └── hud.rs           # HUD rendering (EXTEND - combat feedback)
│   └── render/
│       └── players.rs       # Player rendering (EXTEND - hide dead)
│
└── plix-server/tests/
    └── combat_test.rs       # Combat tests (EXTEND)

tests/                       # Integration tests
```

**Structure Decision**: Existing 6-crate workspace structure is preserved. Combat changes are localized to:
- `plix-common`: Add new event message types
- `plix-server`: Extend combat system, integrate into tick loop
- `plix-client`: Extend HUD and player rendering for combat feedback

## Existing Implementation Status

Based on codebase exploration, the following is **already implemented**:

### Already Complete
- `PlayerInput` struct with `attack: bool` field
- `ServerPlayer` with `health: u8`, `is_dead: bool`, `respawn_tick: Option<Tick>`, `last_attack_tick: Tick`
- `CombatSystem` with `try_attack()` function (range check, cone check, cooldown)
- `GameEvent::PlayerDied`, `GameEvent::PlayerRespawned` in event system
- `HudData` with `health: u8`, `kills: u16`, `deaths: u16`
- Combat tests in `combat_test.rs`
- Constants: `ATTACK_COOLDOWN_TICKS: 30`, `ATTACK_RANGE: 2.0`, `MELEE_DAMAGE: 20`

### Needs Implementation/Integration
1. **Combat events for attacker/victim feedback** (HitConfirmed, DamageTaken)
2. **Tick loop integration** - actually call combat system on tick
3. **Client event handling** - display hit/damage feedback
4. **Client rendering** - hide dead players
5. **Respawn system** - schedule and execute respawn
6. **Additional tests** - cone targeting, feedback events

## Complexity Tracking

> No constitution violations requiring justification.

| Aspect | Complexity Level | Justification |
|--------|-----------------|---------------|
| Protocol changes | Low | Adding 2 event types to existing enum |
| Server combat | Low | Extending existing CombatSystem |
| Client feedback | Low | Extending existing HUD |
| Rendering changes | Low | Conditional render based on is_dead |

## Implementation Phases

### Phase 1: Protocol & Data Model

**Objective**: Extend protocol with combat-specific events.

1. Add new event types to `GameEvent` enum:
   - `HitConfirmed { attacker: PlayerId, target: PlayerId, damage: u8 }`
   - `DamageTaken { victim: PlayerId, attacker: PlayerId, damage: u8, new_health: u8 }`

2. Verify existing types are sufficient:
   - `PlayerSnapshot` already includes health via `ServerPlayer` replication
   - `GameEvent::PlayerDied` and `GameEvent::PlayerRespawned` already exist

3. No new types needed for `PlayerInput` (attack flag exists).

### Phase 2: Server Combat Integration

**Objective**: Wire combat system into tick loop with full event emission.

1. In tick loop (`tick.rs` or `lib.rs`):
   - After movement simulation, iterate players with attack flag
   - Call `CombatSystem::try_attack()` for each
   - On hit: emit `HitConfirmed` to attacker, `DamageTaken` to victim
   - On death: set `is_dead = true`, schedule respawn, emit `PlayerDied`

2. Respawn system:
   - Check `respawn_tick` each tick
   - When tick reached: reset health, set position to spawn, set `is_dead = false`
   - Emit `PlayerRespawned`

3. Match phase gate:
   - Only process attacks during `MatchPhase::Playing`

### Phase 3: Client Integration

**Objective**: Display combat feedback and handle dead player visibility.

1. Event handling in client:
   - On `HitConfirmed`: show "HIT" indicator (HUD text or flash)
   - On `DamageTaken`: show damage indicator + screen flash
   - On `PlayerDied`: show kill feed message
   - On `PlayerRespawned`: optional notification

2. HUD updates:
   - Display local player HP (already in HudData)
   - Show recent combat events (last 3-5 events)

3. Player rendering:
   - Skip rendering players where `is_dead == true`
   - Resume rendering on respawn

### Phase 4: Validation & Testing

**Objective**: Ensure all acceptance criteria met.

1. Manual validation:
   - Two windowed clients attack each other
   - Verify hit feedback appears for attacker
   - Verify damage feedback appears for victim
   - Verify death removes player from view
   - Verify respawn restores player at spawn point

2. Automated tests:
   - Test cone targeting (closest in direction)
   - Test cooldown enforcement
   - Test event emission on hit/death/respawn
   - Test HP updates correctly

3. Non-regression:
   - `cargo test --workspace` passes
   - Headless client connects
   - Load tests run (bots may not attack initially)

## Milestones

| Milestone | Criteria | Verification |
|-----------|----------|--------------|
| M1 | Protocol events added, tests pass | `cargo test -p plix-common` |
| M2 | Server combat integrated, events emitted | `cargo test -p plix-server` |
| M3 | Client shows feedback, dead players hidden | Manual 2-client test |
| M4 | All acceptance criteria met | Full DoD checklist |

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Combat already integrated but broken | Low | Tests exist; verify they pass first |
| Event system capacity | Low | EventBuffer already handles 256 events |
| Client rendering tied to alive state | Low | PlayerSnapshot has is_dead; add conditional |

## Next Steps

After plan approval:
1. Run `/speckit.tasks` to generate implementation tasks
2. Implement in milestone order (M1 → M2 → M3 → M4)
3. Manual validation with two clients after M3
