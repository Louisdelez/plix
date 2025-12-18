# Quickstart: Training Mode

**Feature**: 020-training-mode | **Date**: 2025-12-17

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix workspace cloned and building
- cargo fmt, cargo clippy passing

## Quick Test

```bash
# Run all tests
cargo test -p plix-server

# Run training-specific tests
cargo test -p plix-server training

# Run server with training arena
cargo run -p plix-server -- --arena training_arena
```

## Implementation Checklist

### Phase 1: Core Types

1. **Add GameMode::Training variant**
   ```rust
   // crates/plix-common/src/types.rs
   pub enum GameMode {
       // ... existing
       Training,
   }
   ```

2. **Add BotId type**
   ```rust
   // crates/plix-common/src/types.rs
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
   pub struct BotId(pub u8);
   ```

3. **Extend protocol messages**
   ```rust
   // crates/plix-common/src/protocol/messages.rs
   pub enum ClientMessage {
       // ... existing
       TrainingReset,
       TrainingStatsRequest,
   }

   pub enum ServerMessage {
       // ... existing
       TrainingStats { hits: u32, kills: u32, attacks: u32, accuracy_pct: f32, session_duration_secs: f32 },
   }

   pub enum GameEvent {
       // ... existing
       TrainingReset { player_id: PlayerId },
       BotHit { bot_id: BotId, damage: u8, killed: bool },
       BotRespawned { bot_id: BotId, position: Vec3 },
   }

   pub struct WorldSnapshot {
       // ... existing fields
       #[serde(default)]
       pub bots: Vec<BotSnapshot>,
   }
   ```

### Phase 2: Training Module

1. **Create module structure**
   ```
   crates/plix-server/src/training/
   ├── mod.rs
   ├── config.rs
   ├── bot.rs
   ├── stats.rs
   └── coordinator.rs
   ```

2. **Add to lib.rs**
   ```rust
   // crates/plix-server/src/lib.rs
   pub mod training;
   ```

3. **Implement TrainingConfig** (see data-model.md)

4. **Implement TrainingBot + BotBehavior** (see data-model.md)

5. **Implement TrainingStats** (see data-model.md)

6. **Implement TrainingCoordinator** (see data-model.md)

### Phase 3: Server Integration

1. **Add match config factory**
   ```rust
   // crates/plix-server/src/match_state.rs
   impl MatchConfig {
       pub fn training_default() -> Self {
           Self {
               min_players: 1,
               max_players: 1,
               countdown_seconds: 0,
               time_limit_seconds: 0,      // No time limit
               score_limit: 0,              // No score limit
               respawn_delay_ticks: 60,     // 1 second
               end_screen_seconds: 0,
               reset_delay_ticks: 0,
               arena_rotation: Vec::new(),
           }
       }
   }
   ```

2. **Create TrainingCoordinator in Server::new()**
   ```rust
   // When game_mode == Training
   let training_coordinator = Some(TrainingCoordinator::new(
       TrainingConfig::from_arena(&loaded_arena),
       spawn_points.clone(),
       Tick::ZERO,
   ));
   ```

3. **Call coordinator.tick() in server tick loop**
   ```rust
   // In Server::tick()
   if let Some(ref mut training) = self.training_coordinator {
       let events = training.tick(self.current_tick);
       // Handle events (broadcast respawns, etc.)
   }
   ```

4. **Handle new client messages**
   ```rust
   // In Server::handle_message()
   ClientMessage::TrainingReset => { /* ... */ }
   ClientMessage::TrainingStatsRequest => { /* ... */ }
   ```

5. **Include bots in combat target list**
   ```rust
   // When building attack targets, add bots if training mode
   let bot_targets: Vec<_> = training_coordinator
       .map(|t| t.bots.iter()
           .filter(|b| !b.is_dead)
           .map(|b| (/* pseudo-PlayerId or BotId */, b.position, b.health))
           .collect())
       .unwrap_or_default();
   ```

### Phase 4: Arena Definition

1. **Create training_arena.toml**
   ```toml
   [metadata]
   name = "Training Arena"
   version = "1.0"
   size = [64, 16, 64]
   game_mode = "training"

   [[spawn_points]]
   team = 0
   position = [32.0, 2.0, 32.0]
   rotation = 0.0

   [training]
   bot_count = 5
   bot_behavior = "dummy"
   ```

### Phase 5: Tests

1. **Bot spawn/respawn test**
   ```rust
   #[test]
   fn test_bot_spawn_count() {
       let config = TrainingConfig { bot_count: 5, ..Default::default() };
       let coord = TrainingCoordinator::new(config, spawn_points, Tick(0));
       assert_eq!(coord.bots.len(), 5);
   }
   ```

2. **Stats tracking test**
   ```rust
   #[test]
   fn test_accuracy_calculation() {
       let mut stats = TrainingStats::new(Tick(0));
       stats.attacks = 10;
       stats.hits = 7;
       assert!((stats.accuracy() - 70.0).abs() < 0.01);
   }
   ```

3. **Reset test**
   ```rust
   #[test]
   fn test_session_reset_clears_stats() {
       let mut coord = TrainingCoordinator::new(config, spawns, Tick(0));
       coord.stats.hits = 10;
       coord.stats.kills = 5;
       coord.reset(Tick(100));
       assert_eq!(coord.stats.hits, 0);
       assert_eq!(coord.stats.kills, 0);
   }
   ```

## Testing Locally

```bash
# Terminal 1: Start server with training arena
cargo run -p plix-server -- --arena training_arena

# Terminal 2: Connect client
cargo run -p plix-client -- --connect 127.0.0.1:7777

# In client:
# - Move around and attack bots
# - Press reset key to reset session
# - Press stats key to print stats to console
```

## Common Issues

| Issue | Solution |
|-------|----------|
| "Unknown game mode: training" | Ensure GameMode::Training is added with serde rename |
| Bots not appearing | Check WorldSnapshot includes bots, client renders BotSnapshot |
| Hits not registering | Verify bots are added to combat target list |
| Stats always zero | Check record_hit/record_attack are called |
| Reset not working | Verify TrainingReset message is handled |

## File Checklist

- [ ] `crates/plix-common/src/types.rs` - GameMode::Training, BotId
- [ ] `crates/plix-common/src/protocol/messages.rs` - New messages
- [ ] `crates/plix-server/src/training/mod.rs` - Module exports
- [ ] `crates/plix-server/src/training/config.rs` - TrainingConfig
- [ ] `crates/plix-server/src/training/bot.rs` - TrainingBot, BotBehavior
- [ ] `crates/plix-server/src/training/stats.rs` - TrainingStats
- [ ] `crates/plix-server/src/training/coordinator.rs` - TrainingCoordinator
- [ ] `crates/plix-server/src/lib.rs` - Add training module
- [ ] `crates/plix-server/src/match_state.rs` - training_default()
- [ ] `crates/plix-arena/src/format.rs` - TrainingArenaConfig
- [ ] `assets/arenas/training_arena.toml` - Sample arena
- [ ] `crates/plix-server/tests/training_bot_test.rs`
- [ ] `crates/plix-server/tests/training_stats_test.rs`
- [ ] `crates/plix-server/tests/training_reset_test.rs`
