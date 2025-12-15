# Research: Server-Authoritative Combat System

**Feature**: 003-combat-visible
**Date**: 2025-12-14

## Executive Summary

Research confirms the existing codebase has substantial combat infrastructure already in place. No major architectural decisions needed - the implementation is an integration and completion task rather than a design task.

## Research Topics

### 1. Existing Combat Infrastructure

**Question**: What combat functionality already exists in the codebase?

**Findings**:

| Component | File | Status |
|-----------|------|--------|
| Attack input flag | `plix-common/src/protocol/messages.rs` | Complete |
| Player health tracking | `plix-server/src/session.rs` | Complete |
| Combat system skeleton | `plix-server/src/sim/combat.rs` | Complete |
| Death/respawn fields | `plix-server/src/session.rs` | Complete |
| Death/respawn events | `plix-server/src/replication/events.rs` | Complete |
| HUD health display | `plix-client/src/ui/hud.rs` | Complete |
| Combat tests | `plix-server/tests/combat_test.rs` | Partial |

**Decision**: Extend existing infrastructure rather than redesign.

**Rationale**: The architecture is sound and aligns with constitution principles. Adding new event types and wiring existing systems is lower risk than refactoring.

### 2. Attack Targeting Mechanism

**Question**: How should the server determine which player is hit by an attack?

**Findings**:

The existing `CombatSystem::try_attack()` already implements:
- Range check: `ATTACK_RANGE = 2.0` blocks
- Facing cone: Checks dot product between attacker forward vector and direction to target
- Closest target selection: Iterates candidates, filters by cone, picks nearest
- Exclusions: Skips self, dead players, and (optionally) teammates

**Decision**: Use existing implementation (closest enemy in facing direction).

**Rationale**: Matches the clarified specification. Implementation is already tested.

### 3. Combat Event Types

**Question**: What events are needed for client feedback?

**Findings**:

| Event | Purpose | Recipients | Status |
|-------|---------|------------|--------|
| `PlayerDied` | Death notification | Broadcast | Exists |
| `PlayerRespawned` | Respawn notification | Broadcast | Exists |
| `HitConfirmed` | Attacker hit feedback | Attacker only | Needs creation |
| `DamageTaken` | Victim damage feedback | Victim only | Needs creation |

**Decision**: Add `HitConfirmed` and `DamageTaken` to `GameEvent` enum.

**Rationale**: Allows distinct feedback per role (attacker vs victim) as specified.

**Alternatives Considered**:
- Single `CombatHit` event sent to both: Rejected because it conflates attacker/victim perspectives and requires client-side logic to determine role.
- No events, use snapshot delta: Rejected because feedback latency would be unacceptable (up to one snapshot interval).

### 4. Respawn Mechanism

**Question**: How should respawn timing and location work?

**Findings**:

- `ServerPlayer.respawn_tick: Option<Tick>` - field exists for scheduling
- `MatchConfig.respawn_delay = 180` - 3 seconds at 60 Hz
- `SpawnManager` in plix-arena provides spawn points per team
- `GameEvent::PlayerRespawned` exists for notification

**Decision**: Use existing respawn_tick field; set on death, check each tick.

**Rationale**: Infrastructure exists. Just needs wiring into tick loop.

### 5. Dead Player Visibility

**Question**: How should dead players be rendered (or not)?

**Findings**:

- `ServerPlayer.is_dead: bool` - already replicated to client
- Client receives player state in snapshots
- Player rendering iterates snapshot players

**Decision**: Skip rendering when `is_dead == true` (immediate disappearance).

**Rationale**: Matches specification clarification. Simpler than fade-out animation.

### 6. Combat Constants

**Question**: What values should be used for damage, cooldown, range?

**Findings**:

Existing constants in `plix-server/src/sim/combat.rs`:
```rust
pub const MELEE_DAMAGE: u8 = 20;
pub const ATTACK_COOLDOWN_TICKS: u32 = 30;  // 500ms at 60 Hz
pub const ATTACK_RANGE: f32 = 2.0;          // blocks
```

Existing in `plix-server/src/match_state.rs`:
```rust
respawn_delay: 180  // ticks (3 seconds)
```

Player health in `plix-server/src/session.rs`:
```rust
health: u8 = 100  // default
```

**Decision**: Use existing constants (5 hits to kill, 0.5s cooldown, 2 block range, 3s respawn).

**Rationale**: Values already defined; spec allows simple defaults.

## Unresolved Items

None. All technical questions have clear answers from existing code or specification.

## Dependencies

| Dependency | Version | Purpose | Risk |
|------------|---------|---------|------|
| glam | existing | 3D math for cone calculation | None (already used) |
| bincode | existing | Event serialization | None (already used) |
| wgpu | existing | Rendering | None (already used) |

## Conclusion

No research blockers. Implementation can proceed immediately with:
1. Add two new event types (`HitConfirmed`, `DamageTaken`)
2. Wire combat system into tick loop
3. Implement respawn check in tick loop
4. Add client event handling for feedback
5. Add dead player visibility check in rendering

Estimated complexity: Low (extension of existing systems).
