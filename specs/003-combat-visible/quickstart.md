# Quickstart: Server-Authoritative Combat System

**Feature**: 003-combat-visible
**Date**: 2025-12-14

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix codebase built successfully
- Two terminal windows for testing

## Quick Verification

Before starting implementation, verify existing tests pass:

```bash
# Run all tests
cargo test --workspace

# Run combat-specific tests
cargo test -p plix-server combat
```

## Implementation Order

### Step 1: Add Protocol Events

**File**: `crates/plix-common/src/protocol/messages.rs` (or `events.rs` if separate)

Add to `GameEvent` enum:

```rust
pub enum GameEvent {
    // ... existing variants ...

    // NEW: Combat feedback events
    HitConfirmed {
        attacker: PlayerId,
        target: PlayerId,
        damage: u8,
    },
    DamageTaken {
        victim: PlayerId,
        attacker: PlayerId,
        damage: u8,
        new_health: u8,
    },
}
```

Verify:
```bash
cargo build -p plix-common
```

### Step 2: Wire Combat into Server Tick

**File**: `crates/plix-server/src/lib.rs` (or wherever tick simulation runs)

In the tick processing loop, after movement:

```rust
// Process combat for players with attack flag
for player in session.players_mut() {
    if player.pending_attack() && match_state.phase == MatchPhase::Playing {
        if let Some(hit_result) = combat_system.try_attack(player, &all_players, current_tick) {
            // Emit events
            event_buffer.push(GameEvent::HitConfirmed { ... });
            event_buffer.push(GameEvent::DamageTaken { ... });

            if hit_result.killed {
                event_buffer.push(GameEvent::PlayerDied { ... });
            }
        }
    }
}

// Process respawns
for player in session.players_mut() {
    if let Some(respawn_tick) = player.respawn_tick {
        if current_tick >= respawn_tick {
            player.respawn(spawn_manager.get_spawn(player.team));
            event_buffer.push(GameEvent::PlayerRespawned { id: player.id });
        }
    }
}
```

Verify:
```bash
cargo test -p plix-server
```

### Step 3: Client Event Handling

**File**: `crates/plix-client/src/game.rs` (or event handler)

Handle new events:

```rust
match event {
    GameEvent::HitConfirmed { attacker, .. } if attacker == local_player_id => {
        hud.show_hit_indicator();
    }
    GameEvent::DamageTaken { victim, damage, new_health, .. } if victim == local_player_id => {
        hud.show_damage_effect(damage);
        hud.update_health(new_health);
    }
    GameEvent::PlayerDied { victim, killer } => {
        hud.add_kill_feed(killer, victim);
    }
    _ => {}
}
```

### Step 4: Dead Player Visibility

**File**: `crates/plix-client/src/render/players.rs`

In player rendering loop:

```rust
for player in snapshot.players.iter() {
    if player.is_dead {
        continue;  // Skip rendering dead players
    }
    // ... render player ...
}
```

### Step 5: Manual Testing

Terminal 1 - Start server:
```bash
cargo run -p plix-server
```

Terminal 2 - Start first client:
```bash
cargo run -p plix-client
```

Terminal 3 - Start second client:
```bash
cargo run -p plix-client
```

Test checklist:
- [ ] Move players close together (within 2 blocks)
- [ ] Click to attack - attacker sees hit feedback
- [ ] Victim sees damage feedback and HP decrease
- [ ] After 5 hits (100 HP / 20 damage), victim dies
- [ ] Victim disappears from attacker's view
- [ ] After 3 seconds, victim respawns at spawn point
- [ ] Both players can see respawned player

### Step 6: Run Full Test Suite

```bash
# All tests must pass
cargo test --workspace

# Clippy must pass
cargo clippy --workspace

# Format check
cargo fmt --check
```

## Troubleshooting

### "Attack not registering"

- Verify match phase is `Playing` (check countdown finished)
- Verify cooldown expired (0.5s between attacks)
- Verify target is in range (2 blocks)
- Verify facing target (90° cone)

### "No hit feedback"

- Check event is being emitted by server (add logging)
- Check client is receiving events (add logging)
- Verify local player ID matches event's attacker/victim

### "Dead player still visible"

- Verify `is_dead` field is being replicated in snapshot
- Check render loop is checking `is_dead` flag

### "Respawn not happening"

- Verify `respawn_tick` is being set on death
- Verify tick loop is checking respawn condition
- Check spawn manager returns valid spawn point

## Key Files Reference

| Purpose | File |
|---------|------|
| Protocol events | `crates/plix-common/src/protocol/` |
| Combat system | `crates/plix-server/src/sim/combat.rs` |
| Player state | `crates/plix-server/src/session.rs` |
| Server tick | `crates/plix-server/src/lib.rs` |
| Client HUD | `crates/plix-client/src/ui/hud.rs` |
| Player rendering | `crates/plix-client/src/render/players.rs` |
| Combat tests | `crates/plix-server/tests/combat_test.rs` |
