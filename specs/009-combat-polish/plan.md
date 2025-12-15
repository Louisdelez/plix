# Implementation Plan: Combat Polish

**Branch**: `009-combat-polish` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-combat-polish/spec.md`

## Summary

Polish the existing melee combat system to be fair, readable, and robust under latency by adding:
- Server-authoritative attack cooldown (configurable, default 30 ticks / 0.5s)
- Attack range tuning with latency tolerance epsilon (1.8 blocks + 0.15 tolerance)
- Knockback feedback that respects collision (4.0 m/s impulse)
- Respawn invulnerability to prevent spawn-killing (120 ticks / 2.0s)
- Improved hit registration under latency (tolerance-based, no rewind)

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (protocol, math, time), plix-server (sim, session, validation)
**Storage**: N/A (in-memory state only)
**Testing**: cargo test (unit and integration tests)
**Target Platform**: Linux server, cross-platform clients
**Project Type**: Game server (multiplayer voxel platform)
**Performance Goals**: 60 Hz tick rate, deterministic simulation
**Constraints**: Server-authoritative, no client trust, no server rewind
**Scale/Scope**: 16 players per server, low-latency network

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | All combat decisions server-authoritative |
| II. Performance (60Hz Stability) | ✅ PASS | No blocking operations, tick-based cooldowns |
| III. Architecture (Engine-First) | ✅ PASS | Extends existing combat primitives |
| V. Code Quality (Tested) | ✅ PASS | Unit tests required for all combat logic |
| VI. Technical Standards (Stable Rust) | ✅ PASS | No nightly features |
| IX. Scoping (Minimal MVP) | ✅ PASS | 5 focused features, no weapon systems |

**All gates pass. No violations to justify.**

## Project Structure

### Documentation (this feature)

```text
specs/009-combat-polish/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (N/A - internal server changes)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   └── combat.rs            # NEW: CombatConfig struct with defaults
├── plix-server/src/
│   ├── session.rs           # MODIFY: Add invulnerable_until_tick field
│   ├── validation.rs        # MODIFY: Update ATTACK_RANGE, add epsilon
│   └── sim/
│       └── combat.rs        # MODIFY: Add knockback, invulnerability checks

tests/
├── crates/plix-server/tests/
│   └── combat_test.rs       # MODIFY: Add new test cases
```

**Structure Decision**: Existing monorepo structure with workspace crates. Combat polish extends existing modules rather than adding new crates.

## Complexity Tracking

> No violations to justify - all gates pass.

---

## Phase 0: Research Summary

### Existing Implementation Analysis

**Current State** (from codebase review):
- `CombatSystem::try_attack()` already checks cooldown via `last_attack_tick`
- `ATTACK_COOLDOWN_TICKS = 30` already defined in validation.rs
- `ATTACK_RANGE = 2.0` defined (needs change to 1.8)
- `ServerPlayer` has `last_attack_tick` field
- No invulnerability field exists
- No knockback implementation exists
- No range epsilon/tolerance exists

**Required Changes**:
1. Add `CombatConfig` struct to centralize tunable parameters
2. Add `invulnerable_until_tick` to `ServerPlayer`
3. Update `ATTACK_RANGE` from 2.0 to 1.8
4. Add `ATTACK_RANGE_EPSILON = 0.15` constant
5. Implement knockback as velocity impulse
6. Check invulnerability before damage/knockback

### Design Decisions

| Decision | Rationale | Alternatives Rejected |
|----------|-----------|----------------------|
| Config in plix-common | Shared between client (prediction) and server | Server-only: blocks client animation sync |
| Tick-based invuln | Consistent with existing respawn_tick pattern | Duration-based: adds complexity |
| Velocity impulse knockback | Integrates with existing movement/collision | Position teleport: bypasses walls |
| Range + epsilon | Simple tolerance for latency | Server rewind: too complex for MVP |

---

## Phase 1: Design Artifacts

### Data Model Changes

See [data-model.md](./data-model.md) for full details.

**New Struct: CombatConfig**
```rust
pub struct CombatConfig {
    pub attack_cooldown_ticks: u32,    // Default: 30 (0.5s @ 60Hz)
    pub attack_range: f32,              // Default: 1.8 blocks
    pub attack_range_epsilon: f32,      // Default: 0.15 blocks
    pub knockback_strength: f32,        // Default: 4.0 m/s
    pub respawn_invuln_ticks: u32,      // Default: 120 (2.0s @ 60Hz)
}
```

**ServerPlayer Addition**
```rust
pub struct ServerPlayer {
    // ... existing fields ...
    pub invulnerable_until_tick: Option<Tick>,  // NEW
}
```

### Combat Pipeline Update

```text
1. Player sends attack input
2. Server receives attack request
3. Check: game phase == Playing? (existing)
4. Check: tick >= last_attack_tick + cooldown? (existing, with config)
5. Select target: closest in facing cone (existing)
6. Check: distance <= attack_range + epsilon? (NEW: add epsilon)
7. Check: target.invulnerable_until_tick < current_tick? (NEW)
8. Apply damage (existing)
9. Apply knockback velocity impulse (NEW)
10. Collision system handles wall interactions (existing from 008)
```

### API Contracts

No external API changes. All modifications are internal server logic.
Client input protocol remains unchanged (attack flag in PlayerInput).

---

## Phases Summary

| Phase | Focus | Key Deliverables |
|-------|-------|------------------|
| 1 | Config & Data Model | CombatConfig, invulnerable_until_tick field |
| 2 | Cooldown Enforcement | Use config values, update tests |
| 3 | Range Tuning | 1.8 + 0.15 epsilon, boundary tests |
| 4 | Knockback | Velocity impulse, collision integration |
| 5 | Respawn Invulnerability | Set on spawn, check before damage |
| 6 | Latency Tolerance | Confirm epsilon works, determinism tests |
| 7 | Validation | All tests pass, manual testing |

---

## Definition of Done

- [ ] CombatConfig struct with defaults implemented
- [ ] Cooldown uses configurable value from CombatConfig
- [ ] Attack range reduced to 1.8 with 0.15 epsilon
- [ ] Knockback applied on valid hits
- [ ] Knockback respects collision (no wall clipping)
- [ ] Respawn grants 2 second invulnerability
- [ ] Invulnerable players take no damage or knockback
- [ ] All combat tests pass deterministically
- [ ] cargo clippy and cargo fmt clean
- [ ] Manual testing: combat feels fair under normal latency
