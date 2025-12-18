# Quickstart: CTF Mode Implementation

**Feature**: 018-ctf-mode | **Date**: 2025-12-16

## Prerequisites

- Rust 1.75+ (stable)
- Familiarity with existing TDM/FFA implementation
- Understanding of plix workspace structure

## Quick Reference

### Key Files to Modify

| File | Changes |
|------|---------|
| `crates/plix-common/src/types.rs` | Add `GameMode::Ctf`, `FlagState`, `Flag`, `FlagZone` |
| `crates/plix-common/src/protocol/messages.rs` | Add `CtfFlagUpdate`, `CtfCaptureEvent`, `CtfMatchInfo` |
| `crates/plix-server/src/ctf/mod.rs` | New module (state, rules, coordinator) |
| `crates/plix-server/src/match_state.rs` | Add `ctf_default()`, CTF victory check |
| `crates/plix-server/src/lib.rs` | Integrate CTF coordinator in game loop |
| `crates/plix-arena/src/format.rs` | Add `CtfArenaConfig`, `CtfZoneDef` |
| `crates/plix-arena/src/validate.rs` | Add CTF zone validation |

### Key Types

```rust
// Core flag state - tracks where flag is
pub enum FlagState {
    AtBase,
    Carried { carrier: PlayerId },
    Dropped { position: Vec3, return_tick: Tick },
}

// Zone for interactions
pub struct FlagZone {
    pub team: TeamId,
    pub zone_type: FlagZoneType,  // FlagBase or CaptureZone
    pub min: Vec3,
    pub max: Vec3,
}

// Server-side CTF state
pub struct CtfState {
    pub flags: [Flag; 2],
    pub capture_scores: [u32; 2],
    pub zones: Vec<FlagZone>,
    pub config: CtfConfig,
}
```

## Implementation Order

### Phase 1: Types (plix-common)

1. Add `GameMode::Ctf` variant to enum
2. Add `FlagState` enum
3. Add `Flag` struct with state and base position
4. Add `FlagZone` struct with AABB collision check

### Phase 2: Arena Loading (plix-arena)

1. Add `CtfArenaConfig` struct to format.rs
2. Add `[[ctf.flag_bases]]` and `[[ctf.capture_zones]]` parsing
3. Add validation for CTF zones (2 bases, 2 capture zones)
4. Create example `ctf_arena.toml`

### Phase 3: Protocol (plix-common)

1. Add `CtfFlagUpdate` message
2. Add `CtfCaptureEvent` message
3. Add `CtfMatchInfo` to `MatchState`

### Phase 4: CTF Subsystem (plix-server)

1. Create `ctf/mod.rs`, `ctf/state.rs`
2. Implement `CtfState` with flag management
3. Create `ctf/rules.rs` with pickup/drop/capture logic
4. Create `ctf/coordinator.rs` for event orchestration

### Phase 5: Integration (plix-server)

1. Add `MatchConfig::ctf_default()`
2. Integrate coordinator in game loop
3. Wire up death handling → flag drop
4. Wire up disconnect handling → flag drop
5. Add CTF victory check to match state

### Phase 6: Testing

1. Unit tests for `CtfState`
2. Unit tests for `CtfRules` (all pickup/drop/capture scenarios)
3. Unit tests for `CtfCoordinator`
4. Integration tests for full capture flow
5. Arena validation tests

## Common Patterns

### Flag Pickup Check

```rust
fn can_pickup(player: &Player, flag: &Flag, zones: &[FlagZone]) -> bool {
    // Must be enemy flag
    if player.team == flag.team {
        return false;
    }

    // Check state
    match &flag.state {
        FlagState::AtBase => {
            // Check if player in flag base zone
            zones.iter()
                .find(|z| z.team == flag.team && z.zone_type == FlagZoneType::FlagBase)
                .map(|z| z.contains(player.position))
                .unwrap_or(false)
        }
        FlagState::Dropped { position, .. } => {
            // Check if player near dropped flag
            player.position.distance(*position) < PICKUP_RADIUS
        }
        FlagState::Carried { .. } => false,
    }
}
```

### Capture Check (Classic Rule)

```rust
fn can_capture(player: &Player, state: &CtfState) -> bool {
    let own_team = player.team;
    let enemy_team = own_team.enemy();

    // Must be carrying enemy flag
    let enemy_flag = state.flag(enemy_team);
    if enemy_flag.carrier() != Some(player.id) {
        return false;
    }

    // Must be in own capture zone
    let capture_zone = state.capture_zone(own_team);
    if !capture_zone.map(|z| z.contains(player.position)).unwrap_or(false) {
        return false;
    }

    // Classic rule: own flag must be at base
    state.flag(own_team).is_at_base()
}
```

### Death Handler Integration

```rust
fn handle_player_death(&mut self, player_id: PlayerId, death_pos: Vec3, tick: Tick) {
    // Existing death handling...

    // CTF: drop flag if carrying
    if self.game_mode == GameMode::Ctf {
        let events = self.ctf_coordinator.on_player_death(player_id, death_pos, tick);
        self.broadcast_ctf_events(events);
    }
}
```

## Example Arena TOML

```toml
[metadata]
name = "CTF Arena"
version = "1.0.0"
size = [64, 32, 64]
game_mode = "ctf"

[ctf]
capture_limit = 3
flag_return_delay = 10
respawn_delay = 5
time_limit = 600

# Team 0 (Red) - left side
[[ctf.flag_bases]]
team = 0
min = [4.0, 0.0, 28.0]
max = [12.0, 4.0, 36.0]

[[ctf.capture_zones]]
team = 0
min = [0.0, 0.0, 24.0]
max = [16.0, 4.0, 40.0]

# Team 1 (Blue) - right side
[[ctf.flag_bases]]
team = 1
min = [52.0, 0.0, 28.0]
max = [60.0, 4.0, 36.0]

[[ctf.capture_zones]]
team = 1
min = [48.0, 0.0, 24.0]
max = [64.0, 4.0, 40.0]

# Spawn points and blocks...
```

## Testing Commands

```bash
# Run all tests
cargo test

# Run CTF-specific tests
cargo test ctf

# Run with verbose output
cargo test ctf -- --nocapture

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --all-targets
```

## Checklist

- [ ] `GameMode::Ctf` variant added
- [ ] `FlagState` enum with AtBase/Carried/Dropped
- [ ] `Flag` struct with state and base position
- [ ] `FlagZone` struct with AABB collision
- [ ] `CtfArenaConfig` for TOML parsing
- [ ] CTF zone validation on arena load
- [ ] `CtfFlagUpdate` and `CtfCaptureEvent` messages
- [ ] `CtfState` with flags and scores
- [ ] `CtfRules` with all pickup/drop/capture logic
- [ ] `CtfCoordinator` for event orchestration
- [ ] Game loop integration for positions and deaths
- [ ] `MatchConfig::ctf_default()` configuration
- [ ] Victory detection on capture limit
- [ ] Example `ctf_arena.toml` created
- [ ] All unit tests passing
- [ ] Integration tests for capture flow
- [ ] `cargo clippy` passes
- [ ] `cargo fmt` passes
