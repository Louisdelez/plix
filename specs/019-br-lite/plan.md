# Implementation Plan: BR Lite Mode (Mini Battle Royale)

**Branch**: `019-br-lite` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/019-br-lite/spec.md`

## Summary

Implement a simplified Battle Royale game mode for Plix with a shrinking circular safe zone, permanent player elimination (no respawn), minimal loot pickups (instant health restore, speed boost), and last-player-standing victory condition. The mode integrates with the existing match state machine (Lobby → InProgress → PostMatch → Reset) and follows the CTF coordinator pattern for modular game mode logic.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: plix-common (types, protocol, math), plix-server (match state, game loop), plix-arena (arena loading), glam (Vec3), bincode (serialization), tokio (async), tracing (logging)
**Storage**: N/A (in-memory state only - zone, alive roster, loot reset on match end)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux server (primary), cross-platform client
**Project Type**: Rust workspace with multiple crates
**Performance Goals**: 60Hz tick rate, O(n) per-tick where n = alive players (max 16)
**Constraints**: Server-authoritative, <100ms zone sync latency, deterministic zone shrinking
**Scale/Scope**: 4-16 concurrent players per match

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | Server authoritative for zone, damage, loot, eliminations. Client displays only. |
| II. Performance (Low Latency) | ✅ PASS | O(n) alive players per tick. No world scans. Event-driven loot. |
| III. Architecture (Engine-First) | ✅ PASS | Uses existing match state machine, follows CTF coordinator pattern. |
| IV. Modding (Extensibility) | ✅ PASS | Config-driven via arena TOML. No hardcoded values. |
| V. Code Quality (Tested) | ✅ PASS | Unit tests for zone, damage, loot. Integration test for full match. |
| VI. Technical Standards (Stable Rust) | ✅ PASS | Stable Rust only. cargo clippy/fmt compliant. |
| VII. Player Experience | ✅ PASS | Zone synced to clients. Spectator mode for eliminated players. |
| VIII. Open Source | ✅ PASS | No proprietary dependencies. |
| IX. Scoping & Realism | ✅ PASS | Minimal MVP scope. No complex inventory, airdrops, or teams. |
| X. Long-Term Vision | ✅ PASS | Extensible for future BR variants. Clear module boundaries. |

## Project Structure

### Documentation (this feature)

```text
specs/019-br-lite/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── types.rs              # Add GameMode::BrLite variant
│       └── protocol/
│           └── messages.rs       # Add zone/loot protocol messages
├── plix-server/
│   └── src/
│       ├── br_lite/              # NEW: BR Lite game mode module
│       │   ├── mod.rs            # Module exports
│       │   ├── config.rs         # BrLiteConfig (phases, damage, loot)
│       │   ├── zone.rs           # ZoneController (shrinking logic)
│       │   ├── damage.rs         # DamageController (out-of-zone damage)
│       │   ├── loot.rs           # LootManager (pickups, effects)
│       │   ├── state.rs          # BrLiteState (alive roster, winner)
│       │   └── coordinator.rs    # BrLiteCoordinator (event orchestration)
│       ├── match_state.rs        # Add br_lite_default() config
│       ├── session.rs            # Integrate BR Lite coordinator
│       └── lib.rs                # Export br_lite module
│   └── tests/
│       ├── br_zone_test.rs       # Zone shrinking tests
│       ├── br_damage_test.rs     # Out-of-zone damage tests
│       ├── br_loot_test.rs       # Loot pickup tests
│       ├── br_elimination_test.rs # Elimination and victory tests
│       └── br_match_test.rs      # Full match integration test
└── plix-arena/
    └── src/
        └── format.rs             # Add BR zone/loot config parsing
```

**Structure Decision**: Follow the CTF module pattern (`crates/plix-server/src/ctf/`) for the BR Lite implementation. The module is self-contained with config, state, rules logic, and coordinator. Integration hooks into existing match state machine and session handling.

## Complexity Tracking

No violations. The design follows existing patterns (CTF coordinator) and adds no new abstractions beyond those required for BR mechanics.

## Architecture Overview

### Component Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          Session                                 │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   MatchStateMachine                         ││
│  │  Lobby → Countdown → Playing → EndScreen → Resetting → Lobby││
│  └─────────────────────────────────────────────────────────────┘│
│                              │                                   │
│                              ▼ (if game_mode == BrLite)          │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │                   BrLiteCoordinator                         ││
│  │  ┌─────────────┐  ┌───────────────┐  ┌─────────────────┐   ││
│  │  │ZoneController│  │DamageController│  │   LootManager  │   ││
│  │  │ - phases    │  │ - tick damage │  │ - pickups      │   ││
│  │  │ - radius    │  │ - per phase   │  │ - effects      │   ││
│  │  │ - shrink    │  │               │  │                │   ││
│  │  └─────────────┘  └───────────────┘  └─────────────────┘   ││
│  │                                                             ││
│  │  ┌─────────────────────────────────────────────────────┐   ││
│  │  │                    BrLiteState                       │   ││
│  │  │ - alive_players: HashSet<PlayerId>                  │   ││
│  │  │ - eliminated_players: HashSet<PlayerId>             │   ││
│  │  │ - winner: Option<PlayerId>                          │   ││
│  │  │ - zone_state: ZoneState                             │   ││
│  │  │ - loot_items: Vec<LootItem>                         │   ││
│  │  │ - active_effects: HashMap<PlayerId, Vec<Effect>>    │   ││
│  │  └─────────────────────────────────────────────────────┘   ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
```

### Event Flow

```
1. Match starts (Playing phase)
   └── BrLiteCoordinator::start() → initialize zone, spawn loot

2. Per-tick update
   ├── ZoneController::tick() → update phase timer, shrink radius
   ├── DamageController::tick() → check players outside zone, apply damage
   ├── LootManager::tick() → expire temporary effects
   └── BrLiteState::check_victory() → if alive_count == 1, declare winner

3. Player death
   ├── BrLiteState::eliminate(player_id) → mark as eliminated
   ├── LootManager::clear_effects(player_id) → remove active bonuses
   └── Check victory condition

4. Player pickup loot
   ├── LootManager::try_pickup(player_id, position) → validate & apply
   └── Broadcast pickup event to clients

5. Victory
   └── BrLiteCoordinator::end_match(winner) → transition to EndScreen
```

## Key Design Decisions

### 1. Zone Representation (Circular XZ-plane)

```rust
pub struct ZoneState {
    pub center: Vec2,           // XZ center (fixed for V1)
    pub current_radius: f32,    // Current safe zone radius
    pub target_radius: f32,     // Target radius for current phase
    pub phase_index: usize,     // Current phase (0-indexed)
    pub phase_mode: PhaseMode,  // Stable or Shrinking
    pub phase_timer: u32,       // Ticks remaining in current phase
}

pub enum PhaseMode {
    Stable,     // Zone size fixed
    Shrinking,  // Zone interpolating toward target
}
```

**Rationale**: Circular zone is simpler to compute (distance check) and matches player expectations from BR games. Center is fixed at arena center for V1.

### 2. Phase Configuration

```rust
pub struct ZonePhase {
    pub stable_duration_secs: u32,   // Time zone is stable
    pub shrink_duration_secs: u32,   // Time to shrink to target
    pub end_radius: f32,             // Target radius after shrink
    pub damage_per_tick: u32,        // Damage per second outside zone
}
```

Default 5-phase schedule for ~8 minute matches:
1. Phase 0: 60s stable, 30s shrink to 80% radius, 5 damage/s
2. Phase 1: 45s stable, 25s shrink to 60% radius, 10 damage/s
3. Phase 2: 30s stable, 20s shrink to 40% radius, 15 damage/s
4. Phase 3: 20s stable, 15s shrink to 20% radius, 25 damage/s
5. Phase 4: 15s stable, 10s shrink to 5% radius, 50 damage/s (final)

### 3. Loot Types (Minimal V1)

```rust
pub enum LootType {
    HealthPack { heal_amount: u32 },           // Instant heal
    SpeedBoost { multiplier: f32, duration_secs: u32 }, // Temp speed buff
}
```

No weapon pickups in V1 (players use default combat). Health is instant, speed boost lasts 10 seconds (per clarification).

### 4. Elimination vs Spectating

```rust
pub enum PlayerBrState {
    Alive,
    Eliminated { can_spectate: bool },
    Spectating { target: Option<PlayerId> },
}
```

Eliminated players can enter spectator mode (free camera or follow alive player). No complex spectator UI in V1.

### 5. Victory Condition

- `alive_count == 1`: Last player wins
- `alive_count == 0` (simultaneous deaths): Player with lowest ID wins (deterministic)
- Disconnection = elimination (immediate, no grace period in V1)

## Integration Points

### 1. GameMode Enum Extension

```rust
// plix-common/src/types.rs
pub enum GameMode {
    Tdm,
    Ffa,
    Ctf,
    BrLite,  // NEW
}
```

### 2. Arena Configuration

```toml
# assets/arenas/br_arena.toml
[metadata]
name = "BR Arena"
version = "1.0"
size = [64, 32, 64]
game_mode = "br_lite"

[br_lite]
min_players = 4
post_match_delay_secs = 10
zone_center = [32.0, 32.0]  # Optional, defaults to arena center
initial_radius = 30.0        # Optional, defaults to half arena size

[[br_lite.phases]]
stable_duration = 60
shrink_duration = 30
end_radius = 24.0
damage_per_tick = 5

# ... more phases

[[loot_spawns]]
position = [10.0, 2.0, 10.0]
type = "health_pack"
heal_amount = 25

[[loot_spawns]]
position = [50.0, 2.0, 50.0]
type = "speed_boost"
multiplier = 1.5
duration = 10
```

### 3. Protocol Messages

```rust
// New messages for BR state sync
pub enum ServerMessage {
    // ... existing

    /// Zone state update (sent every phase change + every 5 seconds)
    BrZoneUpdate {
        center: [f32; 2],
        current_radius: f32,
        target_radius: f32,
        phase_index: u8,
        phase_mode: u8,  // 0 = Stable, 1 = Shrinking
        phase_time_remaining_secs: u16,
        damage_per_tick: u16,
    },

    /// Loot spawn (sent at match start)
    BrLootSpawn {
        loot_id: u16,
        position: [f32; 3],
        loot_type: u8,  // 0 = HealthPack, 1 = SpeedBoost
    },

    /// Loot collected
    BrLootPickup {
        loot_id: u16,
        player_id: PlayerId,
    },

    /// Player eliminated (no respawn)
    BrElimination {
        player_id: PlayerId,
        alive_count: u8,
    },

    /// Victory
    BrVictory {
        winner_id: PlayerId,
    },
}
```

### 4. Match State Integration

```rust
impl MatchConfig {
    pub fn br_lite_default() -> Self {
        Self {
            min_players: 4,              // Per clarification Q1
            countdown_ticks: 180,        // 3 seconds
            time_limit_seconds: 600,     // 10 minutes max
            score_limit: 0,              // Not used in BR
            end_screen_ticks: 600,       // 10 seconds
            respawn_delay_ticks: 0,      // No respawn
            arena_rotation: Vec::new(),
            team_size: 0,                // FFA mode
        }
    }
}
```

## Observability

### Metrics (tracing spans)

```rust
tracing::info!(
    target: "br_lite",
    phase = phase_index,
    radius = current_radius,
    alive = alive_count,
    "phase_change"
);

tracing::info!(
    target: "br_lite",
    player_id = %player_id,
    alive_remaining = alive_count,
    "player_eliminated"
);

tracing::info!(
    target: "br_lite",
    winner_id = %winner_id,
    match_duration_secs = duration,
    "match_end"
);
```

### Debug State (exposed via server query)

```rust
pub struct BrLiteDebugInfo {
    pub alive_count: u8,
    pub phase_index: u8,
    pub zone_radius: f32,
    pub zone_mode: &'static str,
    pub phase_time_remaining: u32,
    pub loot_remaining: u8,
    pub total_eliminations: u8,
}
```

## Test Strategy

### Unit Tests

1. **ZoneController**
   - Phase transitions (stable → shrink → stable)
   - Linear interpolation during shrink
   - Final phase handling
   - Determinism (same ticks = same radius)

2. **DamageController**
   - Player inside zone: no damage
   - Player outside zone: damage applied
   - Damage scales with phase
   - Damage applied at 1-second intervals

3. **LootManager**
   - Pickup detection (position overlap)
   - Health pack instant heal
   - Speed boost applied and expires after 10s
   - Loot removed after pickup

4. **BrLiteState**
   - Elimination marks player correctly
   - Victory at alive_count == 1
   - Simultaneous death edge case
   - Disconnect = elimination

### Integration Tests

1. **Full Match Cycle**
   - Lobby → Countdown → Playing → zone shrinks → eliminations → Victory → PostMatch → Reset
   - 4 players, 2 eliminated by combat, 1 by zone, 1 winner

2. **Min Players Gate**
   - Match doesn't start below 4 players
   - Starts when 4th player joins

3. **Edge Cases**
   - All players disconnect → no winner
   - Last two die to zone simultaneously → deterministic winner

## Performance Considerations

- Zone check: O(1) per player (distance calculation)
- Damage tick: O(n) alive players, once per second
- Loot check: O(m) loot items × O(n) players (small constants, event-driven)
- Memory: ~1KB per active match (zone state + player states + loot)

No chunk scanning, no pathfinding, no physics simulation beyond existing combat.

## Out of Scope (Documented)

Per spec, the following are explicitly NOT implemented:
- Complex inventory/crafting
- Airdrops, vehicles
- Team-based BR (duos/squads)
- Dynamic zone center
- Minimap/zone preview UI
- Knockdown/revive mechanics
