# Research: BR Lite Mode

**Feature**: 019-br-lite
**Date**: 2025-12-16

## Research Questions

All technical decisions have been resolved based on existing codebase patterns and user-provided architecture guidance.

### 1. Game Mode Integration Pattern

**Question**: How should BR Lite integrate with the existing match state machine?

**Decision**: Follow the CTF coordinator pattern (`crates/plix-server/src/ctf/`)

**Rationale**:
- CTF already demonstrates decoupled game mode logic with coordinator, state, and rules modules
- Same session integration points (on_player_death, on_player_position, tick)
- Proven pattern for match lifecycle hooks

**Alternatives Considered**:
- Inline logic in session.rs → Rejected: violates separation of concerns
- New trait-based game mode abstraction → Rejected: over-engineering for 4 game modes

### 2. Zone Representation

**Question**: How should the shrinking zone be represented and computed?

**Decision**: Circular zone on XZ plane with fixed center (arena center)

**Rationale**:
- Simple O(1) containment check: `distance(player.xz, center) < radius`
- Matches player expectations from existing BR games
- Sufficient for arena-sized maps (64x64 blocks typical)

**Alternatives Considered**:
- Rectangular zone (AABB) → Rejected: less intuitive player experience
- Polygon zone → Rejected: over-complex for V1
- Dynamic center → Deferred to future version

### 3. Zone Damage Application

**Question**: How frequently should zone damage be applied?

**Decision**: Once per second (60 ticks at 60Hz server tick rate)

**Rationale**:
- Per spec assumption: "Zone damage is applied once per second"
- Matches typical BR games (PUBG, Fortnite use 1-second damage ticks)
- Reduces per-tick computation

**Implementation**:
```rust
// In DamageController::tick()
if current_tick.0 % 60 == 0 {  // Every 60 ticks = 1 second
    for player in alive_players {
        if !is_in_zone(player.position, zone_state) {
            apply_damage(player, zone_state.damage_per_tick);
        }
    }
}
```

### 4. Loot Pickup Detection

**Question**: How should loot pickup be detected?

**Decision**: Server-side position overlap check during movement processing

**Rationale**:
- Server-authoritative (per constitution)
- Uses existing player position updates from session
- Simple sphere-sphere collision (player capsule vs loot point)

**Implementation**:
```rust
const PICKUP_RADIUS: f32 = 1.0;  // 1 block radius

pub fn check_loot_pickup(player_pos: Vec3, loot_pos: Vec3) -> bool {
    let dx = player_pos.x - loot_pos.x;
    let dz = player_pos.z - loot_pos.z;
    let dist_sq = dx * dx + dz * dz;
    dist_sq < PICKUP_RADIUS * PICKUP_RADIUS
}
```

### 5. Speed Boost Effect Application

**Question**: How should temporary speed boosts be applied and expired?

**Decision**: Effect stored per-player with expiration tick, checked in movement system

**Rationale**:
- Deterministic (tick-based expiration)
- Integrates with existing movement processing
- No polling overhead (check on movement only)

**Implementation**:
```rust
pub struct ActiveEffect {
    pub effect_type: EffectType,
    pub expires_at: Tick,
}

pub enum EffectType {
    SpeedBoost { multiplier: f32 },
}

// In movement processing
let speed_multiplier = active_effects
    .get(&player_id)
    .filter(|e| e.expires_at > current_tick)
    .map(|e| match e.effect_type {
        EffectType::SpeedBoost { multiplier } => multiplier,
    })
    .unwrap_or(1.0);
```

### 6. Elimination and Victory Detection

**Question**: How should elimination be tracked and victory detected?

**Decision**: HashSet for alive players, victory when size == 1

**Rationale**:
- O(1) elimination (remove from set)
- O(1) victory check (set size)
- Simple deterministic tie-breaker (lowest PlayerId)

**Implementation**:
```rust
pub fn eliminate(&mut self, player_id: PlayerId) -> Option<PlayerId> {
    self.alive_players.remove(&player_id);
    self.eliminated_players.insert(player_id);

    // Check victory
    if self.alive_players.len() == 1 {
        let winner = *self.alive_players.iter().next().unwrap();
        self.winner = Some(winner);
        Some(winner)
    } else if self.alive_players.is_empty() {
        // Edge case: simultaneous deaths - lowest ID wins
        let winner = *self.eliminated_players.iter().min().unwrap();
        self.winner = Some(winner);
        Some(winner)
    } else {
        None
    }
}
```

### 7. Protocol Message Design

**Question**: What protocol messages are needed for BR Lite?

**Decision**: 5 new message types (zone update, loot spawn, loot pickup, elimination, victory)

**Rationale**:
- Minimal set covering all client-visible state changes
- Event-driven (not polled) for efficiency
- Zone updates periodic (every 5s) plus on phase change

**Messages**:
| Message | When Sent | Payload |
|---------|-----------|---------|
| BrZoneUpdate | Phase change + every 5s | center, radii, phase info, damage |
| BrLootSpawn | Match start | loot_id, position, type |
| BrLootPickup | On pickup | loot_id, player_id |
| BrElimination | On death | player_id, alive_count |
| BrVictory | Match end | winner_id |

### 8. Arena Configuration Parsing

**Question**: How should BR-specific arena config be parsed?

**Decision**: Extend Arena struct with optional `br_lite` section and `loot_spawns` array

**Rationale**:
- Follows existing pattern (flag_zones for CTF)
- Optional section allows reuse of arenas for other modes
- TOML native array for phases

**Schema**:
```toml
[br_lite]
min_players = 4
initial_radius = 30.0

[[br_lite.phases]]
stable_duration = 60
shrink_duration = 30
end_radius = 24.0
damage_per_tick = 5

[[loot_spawns]]
position = [10.0, 2.0, 10.0]
type = "health_pack"
heal_amount = 25
```

## Dependencies

All dependencies are already in the workspace:
- `glam` - Vec2/Vec3 math for zone calculations
- `bincode` - Protocol serialization
- `serde` - TOML parsing for arena config
- `tracing` - Observability logging
- `tokio` - Async runtime (not directly used, inherited from session)

No new dependencies required.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Zone sync desync | Low | Medium | Periodic resync every 5s |
| Loot race condition | Low | Low | Server-authoritative validation |
| Performance with 16 players | Low | Low | O(n) per-tick, n ≤ 16 |
| Complex phase timing bugs | Medium | Medium | Comprehensive unit tests |

## Conclusion

All research questions resolved. The implementation follows established patterns (CTF coordinator) with minimal new abstractions. Ready for Phase 1 (data model and contracts).
