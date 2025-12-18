# Data Model: BR Lite Mode

**Feature**: 019-br-lite
**Date**: 2025-12-16

## Entity Relationship Diagram

```
┌─────────────────────┐         ┌─────────────────────┐
│    BrLiteConfig     │         │      ZonePhase      │
├─────────────────────┤         ├─────────────────────┤
│ min_players: u8     │ 1    * │ stable_duration: u32│
│ post_match_delay: u32│◄──────┤ shrink_duration: u32│
│ initial_radius: f32 │         │ end_radius: f32     │
│ zone_center: Vec2   │         │ damage_per_tick: u32│
│ bonus_duration: u32 │         └─────────────────────┘
│ phases: Vec<Phase>  │
└─────────────────────┘

┌─────────────────────┐         ┌─────────────────────┐
│     ZoneState       │         │      PhaseMode      │
├─────────────────────┤         ├─────────────────────┤
│ center: Vec2        │         │ Stable              │
│ current_radius: f32 │◄───────┤ Shrinking           │
│ target_radius: f32  │         └─────────────────────┘
│ phase_index: usize  │
│ phase_mode: PhaseMode│
│ phase_timer: u32    │
│ damage_per_tick: u32│
└─────────────────────┘

┌─────────────────────┐         ┌─────────────────────┐
│    BrLiteState      │         │   PlayerBrState     │
├─────────────────────┤         ├─────────────────────┤
│ zone: ZoneState     │ 1    * │ Alive               │
│ alive_players: Set  │◄──────┤ Eliminated          │
│ eliminated: Set     │         │ Spectating(target)  │
│ winner: Option<Id>  │         └─────────────────────┘
│ loot_items: Vec     │
│ effects: HashMap    │
│ total_eliminations  │
└─────────────────────┘
          │
          │ 1
          ▼ *
┌─────────────────────┐         ┌─────────────────────┐
│      LootItem       │         │      LootType       │
├─────────────────────┤         ├─────────────────────┤
│ id: u16             │         │ HealthPack {        │
│ position: Vec3      │◄───────┤   heal_amount: u32  │
│ loot_type: LootType │         │ }                   │
│ collected: bool     │         │ SpeedBoost {        │
└─────────────────────┘         │   multiplier: f32   │
                                │   duration_secs: u32│
                                │ }                   │
                                └─────────────────────┘

┌─────────────────────┐
│    ActiveEffect     │
├─────────────────────┤
│ effect_type: Type   │
│ expires_at: Tick    │
└─────────────────────┘
```

## Entity Definitions

### BrLiteConfig

Configuration for a BR Lite match, loaded from arena TOML.

| Field | Type | Description | Default |
|-------|------|-------------|---------|
| `min_players` | `u8` | Minimum players to start match | 4 |
| `post_match_delay_secs` | `u32` | Seconds to show PostMatch screen | 10 |
| `initial_radius` | `f32` | Starting zone radius (blocks) | arena_size / 2 |
| `zone_center` | `Vec2` | XZ center of zone | arena center |
| `bonus_duration_secs` | `u32` | Duration of speed boost effect | 10 |
| `phases` | `Vec<ZonePhase>` | Zone shrinking phases | 5-phase default |

**Validation Rules**:
- `min_players >= 2`
- `initial_radius > 0`
- `phases.len() >= 1`
- Each phase `end_radius < previous.end_radius` (shrinking)

### ZonePhase

Configuration for a single zone phase.

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `stable_duration_secs` | `u32` | Seconds zone is stable | 60 |
| `shrink_duration_secs` | `u32` | Seconds to shrink to target | 30 |
| `end_radius` | `f32` | Target radius after shrink | 24.0 |
| `damage_per_tick` | `u32` | Damage/sec outside zone | 5 |

**Validation Rules**:
- `stable_duration_secs >= 0`
- `shrink_duration_secs > 0`
- `end_radius >= 0`
- `damage_per_tick > 0`

### ZoneState

Runtime state of the shrinking zone.

| Field | Type | Description |
|-------|------|-------------|
| `center` | `Vec2` | XZ center position |
| `current_radius` | `f32` | Current zone radius |
| `target_radius` | `f32` | Target radius for current phase |
| `phase_index` | `usize` | Current phase (0-indexed) |
| `phase_mode` | `PhaseMode` | Stable or Shrinking |
| `phase_timer` | `u32` | Ticks remaining in current mode |
| `damage_per_tick` | `u32` | Current damage per second outside |

**State Transitions**:
```
[Initial] → Stable(phase 0) → Shrinking(phase 0) → Stable(phase 1) → ... → Final
```

**Invariants**:
- `current_radius >= target_radius` during shrink
- `phase_index < phases.len()`
- `phase_timer` counts down each tick

### PhaseMode

Zone phase mode enumeration.

| Variant | Description |
|---------|-------------|
| `Stable` | Zone radius is fixed |
| `Shrinking` | Zone radius is interpolating toward target |

### BrLiteState

Complete BR Lite match state.

| Field | Type | Description |
|-------|------|-------------|
| `zone` | `ZoneState` | Current zone state |
| `alive_players` | `HashSet<PlayerId>` | Players still alive |
| `eliminated_players` | `HashSet<PlayerId>` | Players eliminated |
| `player_states` | `HashMap<PlayerId, PlayerBrState>` | Per-player state |
| `winner` | `Option<PlayerId>` | Winner (if match ended) |
| `loot_items` | `Vec<LootItem>` | Loot spawns in arena |
| `active_effects` | `HashMap<PlayerId, Vec<ActiveEffect>>` | Active buffs |
| `total_eliminations` | `u32` | Total eliminations this match |

**Invariants**:
- `alive_players ∩ eliminated_players = ∅`
- `winner.is_some() ⟹ alive_players.len() <= 1`
- Player in `alive_players` has `player_states[id] == Alive`

### PlayerBrState

Per-player BR state enumeration.

| Variant | Fields | Description |
|---------|--------|-------------|
| `Alive` | - | Player is alive and playing |
| `Eliminated` | - | Player died, can enter spectator |
| `Spectating` | `target: Option<PlayerId>` | Spectating (optional target) |

**State Transitions**:
```
Alive → Eliminated → Spectating
```

### LootItem

A collectible item in the arena.

| Field | Type | Description |
|-------|------|-------------|
| `id` | `u16` | Unique identifier |
| `position` | `Vec3` | World position |
| `loot_type` | `LootType` | Item type and parameters |
| `collected` | `bool` | Whether item was picked up |

**Invariants**:
- `id` is unique within match
- Once `collected = true`, item is no longer pickupable

### LootType

Loot item type enumeration.

| Variant | Fields | Description |
|---------|--------|-------------|
| `HealthPack` | `heal_amount: u32` | Instant health restore |
| `SpeedBoost` | `multiplier: f32, duration_secs: u32` | Temporary speed buff |

**Validation**:
- `HealthPack.heal_amount > 0`
- `SpeedBoost.multiplier > 1.0`
- `SpeedBoost.duration_secs > 0`

### ActiveEffect

A temporary effect active on a player.

| Field | Type | Description |
|-------|------|-------------|
| `effect_type` | `EffectType` | Type of effect |
| `expires_at` | `Tick` | Server tick when effect expires |

**Lifecycle**:
1. Created on loot pickup (speed boost)
2. Applied during movement processing
3. Removed when `current_tick >= expires_at`
4. Force-removed on player elimination

## Arena Configuration Schema

### TOML Structure

```toml
[metadata]
name = "string"
version = "string"
size = [uint, uint, uint]
game_mode = "br_lite"

[br_lite]
min_players = uint        # optional, default 4
post_match_delay = uint   # optional, default 10
zone_center = [f32, f32]  # optional, defaults to arena center
initial_radius = f32      # optional, defaults to arena_size/2

[[br_lite.phases]]
stable_duration = uint
shrink_duration = uint
end_radius = f32
damage_per_tick = uint

# ... repeat for each phase

[[loot_spawns]]
position = [f32, f32, f32]
type = "health_pack" | "speed_boost"
heal_amount = uint       # for health_pack
multiplier = f32         # for speed_boost
duration = uint          # for speed_boost
```

### Rust Struct (plix-arena)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrLiteArenaConfig {
    #[serde(default = "default_min_players")]
    pub min_players: u8,

    #[serde(default = "default_post_match_delay")]
    pub post_match_delay: u32,

    pub zone_center: Option<[f32; 2]>,
    pub initial_radius: Option<f32>,

    #[serde(default)]
    pub phases: Vec<ZonePhaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonePhaseConfig {
    pub stable_duration: u32,
    pub shrink_duration: u32,
    pub end_radius: f32,
    pub damage_per_tick: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootSpawnConfig {
    pub position: [f32; 3],
    #[serde(rename = "type")]
    pub loot_type: String,
    pub heal_amount: Option<u32>,
    pub multiplier: Option<f32>,
    pub duration: Option<u32>,
}
```

## Default Phase Schedule

When no phases are specified, use this 5-phase default (~8 min match):

| Phase | Stable (s) | Shrink (s) | End Radius (%) | Damage/s |
|-------|------------|------------|----------------|----------|
| 0 | 60 | 30 | 80% | 5 |
| 1 | 45 | 25 | 60% | 10 |
| 2 | 30 | 20 | 40% | 15 |
| 3 | 20 | 15 | 20% | 25 |
| 4 | 15 | 10 | 5% | 50 |

Total time: ~270 seconds (4.5 min) + final phase indefinite.

## Serialization

All entities use `serde` with `bincode` for network protocol and TOML for arena config.

### Protocol Encoding

| Entity | Encoding |
|--------|----------|
| `ZoneState` | bincode, ~32 bytes |
| `LootItem` | bincode, ~24 bytes |
| `PlayerBrState` | bincode, 1-3 bytes |
| `BrLiteConfig` | TOML only (not networked) |

### Network-Safe Types

Protocol messages use fixed-size types:
- `f32` for coordinates/radii
- `u8` for phase index, loot type
- `u16` for loot ID, player count
- `u32` for ticks, damage
