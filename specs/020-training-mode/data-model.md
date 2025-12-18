# Data Model: Training Mode

**Feature**: 020-training-mode | **Date**: 2025-12-17

## Entity Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                    TrainingCoordinator                          │
│  - Orchestrates training session                                │
│  - Manages bots, stats, reset                                   │
└───────────────┬────────────────────────┬───────────────────────┘
                │                        │
                ▼                        ▼
┌───────────────────────┐    ┌─────────────────────────┐
│   TrainingConfig      │    │     TrainingStats       │
│  - bot_count          │    │  - hits, kills          │
│  - bot_behavior       │    │  - attacks              │
│  - respawn_delays     │    │  - session_start        │
│  - invincibility      │    │  + accuracy()           │
└───────────────────────┘    │  + duration()           │
                             └─────────────────────────┘
                ▼
┌───────────────────────────────────────────────────────┐
│              Vec<TrainingBot>                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │  TrainingBot                                    │  │
│  │  - id: BotId                                    │  │
│  │  - position: Vec3                               │  │
│  │  - health: u8                                   │  │
│  │  - is_dead: bool                                │  │
│  │  - respawn_timer: Option<Tick>                  │  │
│  │  - spawn_point: Vec3                            │  │
│  │  - behavior: BotBehavior                        │  │
│  └─────────────────────────────────────────────────┘  │
└───────────────────────────────────────────────────────┘
```

---

## Core Entities

### GameMode (Extend Existing)

**Location**: `crates/plix-common/src/types.rs`

```rust
/// Game mode for arena matches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    #[default]
    Tdm,
    Ffa,
    Ctf,
    BrLite,
    Training,  // NEW
}
```

**State Transitions**: N/A (enum discriminant only)

---

### TrainingConfig

**Location**: `crates/plix-server/src/training/config.rs`

| Field | Type | Default | Validation | Description |
|-------|------|---------|------------|-------------|
| bot_count | u8 | 5 | 0-20 | Number of bots to spawn |
| bot_behavior | BotBehaviorType | Dummy | enum | Behavior for all bots |
| bot_respawn_delay_ticks | u32 | 180 (3s @ 60Hz) | >= 0 | Ticks before bot respawns |
| player_respawn_delay_ticks | u32 | 60 (1s @ 60Hz) | >= 0 | Ticks before player respawns |
| invincibility_player | bool | false | - | Player immune to damage |
| invincibility_bots | bool | false | - | Bots immune to damage (hits still count) |

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub bot_count: u8,
    pub bot_behavior: BotBehaviorType,
    pub bot_respawn_delay_ticks: u32,
    pub player_respawn_delay_ticks: u32,
    pub invincibility_player: bool,
    pub invincibility_bots: bool,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            bot_count: 5,
            bot_behavior: BotBehaviorType::Dummy,
            bot_respawn_delay_ticks: 180,      // 3 seconds at 60Hz
            player_respawn_delay_ticks: 60,    // 1 second at 60Hz
            invincibility_player: false,
            invincibility_bots: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BotBehaviorType {
    #[default]
    Dummy,   // Stationary
    Roam,    // Random slow movement
    Strafe,  // Side-to-side oscillation
}
```

**State Transitions**: Immutable during session (loaded from arena config)

---

### BotId

**Location**: `crates/plix-server/src/training/bot.rs`

```rust
/// Unique identifier for training bots (server-local only)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BotId(pub u8);

impl BotId {
    pub const NONE: Self = Self(0xFF);
}
```

**Validation**: 0-254 valid, 255 = NONE

---

### TrainingBot

**Location**: `crates/plix-server/src/training/bot.rs`

| Field | Type | Description |
|-------|------|-------------|
| id | BotId | Unique identifier (0-254) |
| position | Vec3 | Current world position |
| health | u8 | Current health (0-100) |
| is_dead | bool | Dead flag |
| respawn_tick | Option<Tick> | When bot can respawn (None if alive) |
| spawn_point | Vec3 | Initial spawn position (for reset) |
| behavior | BotBehavior | Runtime behavior state |

```rust
#[derive(Debug, Clone)]
pub struct TrainingBot {
    pub id: BotId,
    pub position: Vec3,
    pub health: u8,
    pub is_dead: bool,
    pub respawn_tick: Option<Tick>,
    pub spawn_point: Vec3,
    pub behavior: BotBehavior,
}

impl TrainingBot {
    pub fn new(id: BotId, spawn_point: Vec3, behavior_type: BotBehaviorType) -> Self {
        Self {
            id,
            position: spawn_point,
            health: 100,
            is_dead: false,
            respawn_tick: None,
            spawn_point,
            behavior: BotBehavior::new(behavior_type, spawn_point),
        }
    }

    /// Take damage, returns true if killed
    pub fn take_damage(&mut self, amount: u8, respawn_tick: Tick) -> bool {
        if self.is_dead {
            return false;
        }
        self.health = self.health.saturating_sub(amount);
        if self.health == 0 {
            self.is_dead = true;
            self.respawn_tick = Some(respawn_tick);
            true
        } else {
            false
        }
    }

    /// Respawn bot at spawn point
    pub fn respawn(&mut self) {
        self.position = self.spawn_point;
        self.health = 100;
        self.is_dead = false;
        self.respawn_tick = None;
        self.behavior.reset(self.spawn_point);
    }
}
```

**State Transitions**:
```text
                    ┌───────────────────────┐
                    │                       │
      spawn/        ▼                       │ respawn_tick reached
      reset    ┌─────────┐    take_damage   │
    ─────────► │  Alive  │ ─────────────────┤
               │ is_dead │   (health -> 0)  │
               │ = false │                  ▼
               └─────────┘             ┌─────────┐
                    ▲                  │  Dead   │
                    │   respawn()      │ is_dead │
                    └──────────────────│ = true  │
                                       └─────────┘
```

---

### BotBehavior

**Location**: `crates/plix-server/src/training/bot.rs`

```rust
/// Runtime behavior state (varies by behavior type)
#[derive(Debug, Clone)]
pub enum BotBehavior {
    /// Stationary - no movement
    Dummy,

    /// Random slow movement around spawn point
    Roam {
        direction: Vec3,        // Current movement direction (normalized)
        change_tick: Tick,      // Next direction change
        move_speed: f32,        // Units per tick (default: 0.05)
        max_radius: f32,        // Max distance from spawn (default: 5.0)
    },

    /// Side-to-side oscillation around spawn point
    Strafe {
        phase: f32,             // Current oscillation phase (0..2π)
        center: Vec3,           // Center point (spawn position)
        radius: f32,            // Oscillation amplitude (default: 3.0)
        speed: f32,             // Phase increment per tick (default: 0.05)
    },
}

impl BotBehavior {
    pub fn new(behavior_type: BotBehaviorType, spawn_point: Vec3) -> Self {
        match behavior_type {
            BotBehaviorType::Dummy => BotBehavior::Dummy,
            BotBehaviorType::Roam => BotBehavior::Roam {
                direction: Vec3::ZERO,
                change_tick: Tick(0),
                move_speed: 0.05,
                max_radius: 5.0,
            },
            BotBehaviorType::Strafe => BotBehavior::Strafe {
                phase: 0.0,
                center: spawn_point,
                radius: 3.0,
                speed: 0.05,
            },
        }
    }

    /// Reset behavior state (on respawn/session reset)
    pub fn reset(&mut self, spawn_point: Vec3) {
        match self {
            BotBehavior::Dummy => {}
            BotBehavior::Roam { direction, change_tick, .. } => {
                *direction = Vec3::ZERO;
                *change_tick = Tick(0);
            }
            BotBehavior::Strafe { phase, center, .. } => {
                *phase = 0.0;
                *center = spawn_point;
            }
        }
    }

    /// Update position based on behavior, returns new position
    pub fn update(&mut self, current_pos: Vec3, spawn_point: Vec3, current_tick: Tick) -> Vec3 {
        match self {
            BotBehavior::Dummy => current_pos,

            BotBehavior::Roam { direction, change_tick, move_speed, max_radius } => {
                // Change direction periodically
                if current_tick.0 >= change_tick.0 {
                    // Random direction (simplified: use tick as seed)
                    let angle = (current_tick.0 as f32 * 0.1) % (2.0 * std::f32::consts::PI);
                    *direction = Vec3::new(angle.cos(), 0.0, angle.sin()).normalize_or_zero();
                    *change_tick = Tick(current_tick.0 + 60); // Change every second
                }

                let new_pos = current_pos + *direction * *move_speed;

                // Clamp to max radius from spawn
                let delta = new_pos - spawn_point;
                if delta.length() > *max_radius {
                    spawn_point + delta.normalize_or_zero() * *max_radius
                } else {
                    new_pos
                }
            }

            BotBehavior::Strafe { phase, center, radius, speed } => {
                *phase += *speed;
                if *phase > 2.0 * std::f32::consts::PI {
                    *phase -= 2.0 * std::f32::consts::PI;
                }

                // Oscillate left-right (X axis) around center
                let offset = phase.sin() * *radius;
                Vec3::new(center.x + offset, center.y, center.z)
            }
        }
    }
}
```

---

### TrainingStats

**Location**: `crates/plix-server/src/training/stats.rs`

| Field | Type | Description |
|-------|------|-------------|
| hits | u32 | Total hits on bots |
| kills | u32 | Total bot kills |
| attacks | u32 | Total attack attempts |
| session_start | Tick | When session started |

```rust
#[derive(Debug, Clone, Default)]
pub struct TrainingStats {
    pub hits: u32,
    pub kills: u32,
    pub attacks: u32,
    pub session_start: Tick,
}

impl TrainingStats {
    pub fn new(start_tick: Tick) -> Self {
        Self {
            hits: 0,
            kills: 0,
            attacks: 0,
            session_start: start_tick,
        }
    }

    /// Record an attack attempt
    pub fn record_attack(&mut self) {
        self.attacks += 1;
    }

    /// Record a hit on a bot
    pub fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record a bot kill
    pub fn record_kill(&mut self) {
        self.kills += 1;
    }

    /// Calculate accuracy percentage (0.0 - 100.0)
    pub fn accuracy(&self) -> f32 {
        if self.attacks == 0 {
            0.0
        } else {
            (self.hits as f32 / self.attacks as f32) * 100.0
        }
    }

    /// Calculate session duration in seconds
    pub fn duration_secs(&self, current_tick: Tick, tick_rate: u8) -> f32 {
        let elapsed_ticks = current_tick.0.saturating_sub(self.session_start.0);
        elapsed_ticks as f32 / tick_rate as f32
    }

    /// Reset all stats
    pub fn reset(&mut self, start_tick: Tick) {
        self.hits = 0;
        self.kills = 0;
        self.attacks = 0;
        self.session_start = start_tick;
    }
}
```

---

### TrainingCoordinator

**Location**: `crates/plix-server/src/training/coordinator.rs`

```rust
#[derive(Debug)]
pub struct TrainingCoordinator {
    pub config: TrainingConfig,
    pub bots: Vec<TrainingBot>,
    pub stats: TrainingStats,
    spawn_points: Vec<Vec3>,
}

impl TrainingCoordinator {
    pub fn new(config: TrainingConfig, spawn_points: Vec<Vec3>, start_tick: Tick) -> Self {
        let mut coordinator = Self {
            config,
            bots: Vec::new(),
            stats: TrainingStats::new(start_tick),
            spawn_points,
        };
        coordinator.spawn_all_bots();
        coordinator
    }

    /// Spawn all bots at initial positions
    fn spawn_all_bots(&mut self) {
        self.bots.clear();
        for i in 0..self.config.bot_count {
            let spawn_idx = i as usize % self.spawn_points.len().max(1);
            let spawn_point = self.spawn_points.get(spawn_idx)
                .copied()
                .unwrap_or(Vec3::new(0.0, 1.0, 0.0));
            let bot = TrainingBot::new(
                BotId(i),
                spawn_point,
                self.config.bot_behavior,
            );
            self.bots.push(bot);
        }
    }

    /// Tick update - returns events (hits, kills, respawns)
    pub fn tick(&mut self, current_tick: Tick) -> Vec<TrainingEvent> {
        let mut events = Vec::new();

        for bot in &mut self.bots {
            if bot.is_dead {
                // Check for respawn
                if let Some(respawn_tick) = bot.respawn_tick {
                    if current_tick.0 >= respawn_tick.0 {
                        bot.respawn();
                        events.push(TrainingEvent::BotRespawned { bot_id: bot.id });
                    }
                }
            } else {
                // Update behavior
                let new_pos = bot.behavior.update(
                    bot.position,
                    bot.spawn_point,
                    current_tick,
                );
                bot.position = new_pos;
            }
        }

        events
    }

    /// Process hit on a bot
    pub fn process_hit(&mut self, bot_id: BotId, damage: u8, current_tick: Tick) -> bool {
        self.stats.record_hit();

        if self.config.invincibility_bots {
            return false; // Hit counted but no damage
        }

        if let Some(bot) = self.bots.iter_mut().find(|b| b.id == bot_id) {
            let respawn_tick = Tick(current_tick.0 + self.config.bot_respawn_delay_ticks);
            let killed = bot.take_damage(damage, respawn_tick);
            if killed {
                self.stats.record_kill();
            }
            killed
        } else {
            false
        }
    }

    /// Reset session - reposition player, respawn bots, clear stats
    pub fn reset(&mut self, current_tick: Tick) {
        self.spawn_all_bots();
        self.stats.reset(current_tick);
    }

    /// Get bot positions for snapshot
    pub fn bot_snapshots(&self) -> Vec<BotSnapshot> {
        self.bots.iter()
            .map(|b| BotSnapshot {
                id: b.id,
                position: b.position,
                health: b.health,
                is_dead: b.is_dead,
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub enum TrainingEvent {
    BotRespawned { bot_id: BotId },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSnapshot {
    pub id: BotId,
    pub position: Vec3,
    pub health: u8,
    pub is_dead: bool,
}
```

---

## Relationships

```text
Arena Config
    │
    │ loads
    ▼
TrainingConfig ◄────────── TrainingCoordinator
                                  │
                           ┌──────┼──────┐
                           │      │      │
                           ▼      ▼      ▼
                      Vec<Bot>  Stats  SpawnPts

TrainingCoordinator ─────► Server.tick()
        │                        │
        │ events                 │ snapshots
        ▼                        ▼
   GameEvent            WorldSnapshot + BotSnapshots
```

---

## Arena Config Extension

**Location**: `crates/plix-arena/src/format.rs`

```rust
/// Training mode arena configuration (optional)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrainingArenaConfig {
    /// Number of bots (overrides server default)
    #[serde(default)]
    pub bot_count: Option<u8>,

    /// Bot behavior type (overrides server default)
    #[serde(default)]
    pub bot_behavior: Option<String>,

    /// Bot respawn delay in seconds
    #[serde(default)]
    pub bot_respawn_delay_secs: Option<f32>,

    /// Player respawn delay in seconds
    #[serde(default)]
    pub player_respawn_delay_secs: Option<f32>,

    /// Dedicated bot spawn points (if different from player spawns)
    #[serde(default)]
    pub bot_spawns: Vec<BotSpawnPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSpawnPoint {
    pub position: [f32; 3],
}
```

**Sample Arena TOML**:
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
bot_respawn_delay_secs = 3.0
player_respawn_delay_secs = 1.0

[[training.bot_spawns]]
position = [20.0, 2.0, 32.0]

[[training.bot_spawns]]
position = [44.0, 2.0, 32.0]

[[training.bot_spawns]]
position = [32.0, 2.0, 20.0]

[[training.bot_spawns]]
position = [32.0, 2.0, 44.0]

[[training.bot_spawns]]
position = [26.0, 2.0, 26.0]
```

---

## Validation Rules

| Entity | Rule | Error |
|--------|------|-------|
| TrainingConfig.bot_count | 0-20 | "bot_count must be 0-20" |
| TrainingConfig.respawn_delay | >= 0 | "respawn_delay cannot be negative" |
| BotId | 0-254 | "BotId 255 is reserved" |
| TrainingStats.accuracy | attacks > 0 for calculation | Returns 0.0 if no attacks |
| BotBehavior.max_radius | > 0 | "max_radius must be positive" |
