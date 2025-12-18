# Quickstart: BR Lite Mode Development

**Feature**: 019-br-lite
**Date**: 2025-12-16

## Overview

This guide helps developers get started with implementing and testing BR Lite mode.

## Prerequisites

- Rust 1.75+ (stable)
- Plix repository cloned and built
- Existing familiarity with plix-server and plix-common crates

## Project Setup

### 1. Create BR Lite Module

```bash
# Create the br_lite module structure
mkdir -p crates/plix-server/src/br_lite

# Create module files
touch crates/plix-server/src/br_lite/mod.rs
touch crates/plix-server/src/br_lite/config.rs
touch crates/plix-server/src/br_lite/zone.rs
touch crates/plix-server/src/br_lite/damage.rs
touch crates/plix-server/src/br_lite/loot.rs
touch crates/plix-server/src/br_lite/state.rs
touch crates/plix-server/src/br_lite/coordinator.rs
```

### 2. Add GameMode Variant

In `crates/plix-common/src/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    #[default]
    Tdm,
    Ffa,
    Ctf,
    BrLite,  // NEW
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Tdm => write!(f, "TDM"),
            GameMode::Ffa => write!(f, "FFA"),
            GameMode::Ctf => write!(f, "CTF"),
            GameMode::BrLite => write!(f, "BR"),  // NEW
        }
    }
}
```

### 3. Create Test Arena

Create `assets/arenas/br_test.toml`:

```toml
[metadata]
name = "BR Test Arena"
version = "1.0"
size = [32, 16, 32]
game_mode = "br_lite"

[br_lite]
min_players = 2  # Low for testing
post_match_delay = 5

[[br_lite.phases]]
stable_duration = 10
shrink_duration = 5
end_radius = 12.0
damage_per_tick = 10

[[br_lite.phases]]
stable_duration = 5
shrink_duration = 5
end_radius = 4.0
damage_per_tick = 25

[[spawn_points]]
team = 0
position = [5.0, 2.0, 5.0]
rotation = 45.0

[[spawn_points]]
team = 0
position = [27.0, 2.0, 27.0]
rotation = 225.0

[[loot_spawns]]
position = [16.0, 2.0, 16.0]
type = "health_pack"
heal_amount = 50

[blocks]
# Minimal block setup for testing
```

## Implementation Order

### Phase 1: Core Types (plix-common)

1. Add `GameMode::BrLite` variant ✓
2. Add protocol message types in `protocol/messages.rs`

### Phase 2: Config & State (plix-server/br_lite)

1. `config.rs` - BrLiteConfig, ZonePhase structs
2. `state.rs` - BrLiteState, PlayerBrState, ZoneState
3. `zone.rs` - ZoneController with phase logic
4. `loot.rs` - LootManager, LootItem, ActiveEffect

### Phase 3: Game Logic (plix-server/br_lite)

1. `damage.rs` - DamageController (out-of-zone damage)
2. `coordinator.rs` - BrLiteCoordinator (event orchestration)

### Phase 4: Integration (plix-server)

1. `match_state.rs` - Add `br_lite_default()` config
2. `session.rs` - Integrate coordinator in game loop
3. `lib.rs` - Export br_lite module

### Phase 5: Arena Parsing (plix-arena)

1. `format.rs` - Add BrLiteArenaConfig parsing

## Running Tests

### Unit Tests

```bash
# Run all BR Lite tests
cargo test -p plix-server br_

# Run specific test
cargo test -p plix-server br_zone_test

# Run with output
cargo test -p plix-server br_ -- --nocapture
```

### Integration Test

```bash
# Run full match test
cargo test -p plix-server br_match_test
```

### Manual Testing

```bash
# Start server with BR arena
cargo run -p plix-server -- --arena assets/arenas/br_test.toml

# In another terminal, connect client
cargo run -p plix-client -- --connect 127.0.0.1:7878
```

## Key Code Patterns

### Zone Distance Check

```rust
pub fn is_in_zone(player_pos: Vec3, zone: &ZoneState) -> bool {
    let dx = player_pos.x - zone.center.x;
    let dz = player_pos.z - zone.center.y;  // Vec2 uses y for z
    let dist_sq = dx * dx + dz * dz;
    dist_sq < zone.current_radius * zone.current_radius
}
```

### Phase Transition

```rust
pub fn tick(&mut self, current_tick: Tick) {
    if self.phase_timer > 0 {
        self.phase_timer -= 1;

        // Interpolate radius during shrink
        if self.phase_mode == PhaseMode::Shrinking {
            let total_ticks = self.shrink_duration_ticks;
            let elapsed = total_ticks - self.phase_timer;
            let progress = elapsed as f32 / total_ticks as f32;
            self.current_radius = lerp(self.start_radius, self.target_radius, progress);
        }

        return;
    }

    // Transition to next phase/mode
    match self.phase_mode {
        PhaseMode::Stable => {
            self.phase_mode = PhaseMode::Shrinking;
            self.phase_timer = self.phases[self.phase_index].shrink_duration * 60;
            self.start_radius = self.current_radius;
        }
        PhaseMode::Shrinking => {
            self.current_radius = self.target_radius;
            self.phase_index += 1;
            if self.phase_index < self.phases.len() {
                self.phase_mode = PhaseMode::Stable;
                self.phase_timer = self.phases[self.phase_index].stable_duration * 60;
                self.target_radius = self.phases[self.phase_index].end_radius;
                self.damage_per_tick = self.phases[self.phase_index].damage_per_tick;
            }
            // else: stay in final phase indefinitely
        }
    }
}
```

### Elimination Check

```rust
pub fn eliminate(&mut self, player_id: PlayerId) -> Option<PlayerId> {
    self.alive_players.remove(&player_id);
    self.eliminated_players.insert(player_id);
    self.total_eliminations += 1;

    match self.alive_players.len() {
        1 => {
            let winner = *self.alive_players.iter().next().unwrap();
            self.winner = Some(winner);
            Some(winner)
        }
        0 => {
            // Edge case: use lowest ID as winner
            let winner = self.eliminated_players.iter().min().copied().unwrap();
            self.winner = Some(winner);
            Some(winner)
        }
        _ => None,
    }
}
```

## Debugging

### Enable BR Lite Tracing

```bash
RUST_LOG=br_lite=debug cargo run -p plix-server -- --arena br_test.toml
```

### Log Output Examples

```
INFO br_lite: phase_change phase=0 radius=30.0 alive=4
INFO br_lite: player_eliminated player_id=Player(3) alive_remaining=3
INFO br_lite: loot_pickup player_id=Player(1) loot_id=0 type="health_pack"
INFO br_lite: match_end winner_id=Player(2) match_duration_secs=180
```

## Common Issues

### Zone Not Shrinking

- Check `phase_timer` is decrementing
- Verify phase config has `shrink_duration > 0`
- Ensure `tick()` is called every server tick

### Players Not Taking Damage

- Verify `is_in_zone()` uses correct coordinate mapping (XZ plane)
- Check damage tick interval (should be every 60 ticks)
- Ensure `alive_players` set is updated correctly

### Loot Not Working

- Verify loot positions are within arena bounds
- Check pickup radius (default 1.0 blocks)
- Ensure server validates pickup before broadcasting

## Next Steps

After basic implementation:

1. Add client-side zone visualization
2. Implement spectator mode for eliminated players
3. Add UI for loot effects (speed boost indicator)
4. Create additional BR arenas for variety
