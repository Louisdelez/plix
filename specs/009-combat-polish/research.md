# Research: Combat Polish

**Feature**: 009-combat-polish
**Date**: 2025-12-15

## Existing Implementation Analysis

### Current Combat System (`plix-server/src/sim/combat.rs`)

**CombatSystem::try_attack()** already implements:
- Cooldown check: `ticks_since_attack < ATTACK_COOLDOWN_TICKS`
- Range check: `distance > ATTACK_RANGE`
- Facing cone: 90-degree cone (dot product > 0)
- Target selection: closest valid target in range
- Damage application: returns `HitResult` with damage and killed flag

**HitResult struct**:
```rust
pub struct HitResult {
    pub attacker: PlayerId,
    pub target: PlayerId,
    pub damage: u8,
    pub killed: bool,
}
```

### Current Constants (`plix-server/src/validation.rs`)

```rust
pub const ATTACK_COOLDOWN_TICKS: u32 = 30;  // Already correct (0.5s @ 60Hz)
pub const ATTACK_RANGE: f32 = 2.0;          // Needs change to 1.8
```

### Current Player State (`plix-server/src/session.rs`)

**ServerPlayer** has:
- `last_attack_tick: Tick` - Already exists for cooldown
- `respawn_tick: Option<Tick>` - When player can respawn
- `velocity: Vec3` - Used for movement, will receive knockback

**Missing**:
- `invulnerable_until_tick: Option<Tick>` - Needed for spawn protection

### Current Spawn Logic

```rust
pub fn spawn(&mut self, position: Vec3, yaw: f32) {
    self.position = position;
    self.rotation = Rotation::new(yaw, 0.0);
    self.velocity = Vec3::ZERO;
    self.health = 100;
    self.is_dead = false;
    self.respawn_tick = None;
    // Reset anti-cheat position tracking
    self.anti_cheat.update_position(position);
}
```

**Change needed**: Set `invulnerable_until_tick` on spawn.

---

## Design Decisions

### Decision 1: CombatConfig Location

**Chosen**: Add `CombatConfig` to `plix-common/src/combat.rs`

**Rationale**:
- Allows client to access config for animation timing prediction
- Follows existing pattern (MovementConfig in plix-common/physics.rs)
- Centralized configuration for tuning

**Alternative Rejected**: Server-only config
- Would block client from predicting attack animations
- Inconsistent with existing MovementConfig pattern

### Decision 2: Invulnerability Implementation

**Chosen**: `invulnerable_until_tick: Option<Tick>` field on ServerPlayer

**Rationale**:
- Consistent with existing `respawn_tick: Option<Tick>` pattern
- Simple comparison: `tick < invulnerable_until_tick`
- No additional timer system needed

**Alternative Rejected**: Duration-based with Instant
- Adds wall-clock dependency
- Inconsistent with tick-based simulation

### Decision 3: Knockback Implementation

**Chosen**: Velocity impulse added to victim's velocity

**Rationale**:
- Integrates with existing movement/collision system (Feature 008)
- Physics handles wall stopping automatically
- Frame-rate independent (velocity * dt in movement)

**Alternative Rejected**: Position teleport
- Bypasses collision detection
- Could cause wall clipping

### Decision 4: Range Tolerance

**Chosen**: `distance <= attack_range + attack_range_epsilon`

**Rationale**:
- Simple additive tolerance
- Deterministic (same formula on all clients)
- No server history/rewind needed

**Alternative Rejected**: Server rewind lag compensation
- Complex implementation
- Requires position history buffers
- Out of scope per non-goals

---

## Integration Points

### Movement System (Feature 008)

Knockback integrates via velocity:
1. Combat sets `victim.velocity += knockback_dir * knockback_strength`
2. Movement system processes velocity in next tick
3. Collision system prevents wall penetration

**No changes to movement/collision needed** - existing system handles it.

### Event System

Existing events to use:
- `HitConfirmed` - Already sent on successful hit
- `DamageTaken` - Already sent to victim

New events (optional, for HUD feedback):
- Could add `AttackRejectedCooldown` - Not required for MVP
- Could add `DamageBlockedInvuln` - Not required for MVP

**Decision**: No new events for MVP. Existing events sufficient.

---

## Test Strategy

### Unit Tests (combat.rs)

1. **Cooldown enforcement**
   - Attack during cooldown → None
   - Attack after cooldown → Some(HitResult)

2. **Range boundary**
   - Distance = 1.8 → Hit
   - Distance = 1.9 (within epsilon) → Hit
   - Distance = 2.0 (beyond epsilon) → Miss

3. **Invulnerability**
   - Attack invulnerable target → None (or blocked result)
   - Attack after invuln expires → Hit

4. **Knockback direction**
   - Impulse direction = normalize(victim - attacker)

### Integration Tests (combat_test.rs)

1. **Knockback + collision**
   - Knockback toward wall → Player stops at wall

2. **Respawn + invuln**
   - Player respawns → Immediate attack blocked
   - 2 seconds later → Attack succeeds

3. **Determinism**
   - Same inputs → Same combat outcomes

---

## Risk Analysis

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Knockback clips through walls | Low | High | Use velocity impulse, collision handles it |
| Epsilon too generous | Medium | Medium | Start conservative (0.15), tune based on testing |
| Invuln duration too long | Low | Low | Configurable, can tune later |
| Breaking existing tests | Medium | Medium | Update tests alongside implementation |

---

## Summary

**All NEEDS CLARIFICATION items resolved:**
- No unknowns remain
- Implementation path is clear
- Existing infrastructure supports all features
- Changes are additive (low risk of regression)

**Ready for data-model.md generation.**
