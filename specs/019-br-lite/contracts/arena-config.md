# Arena Configuration Contract: BR Lite Mode

**Feature**: 019-br-lite
**Date**: 2025-12-16

## Overview

This document defines the TOML configuration schema for BR Lite arenas. The configuration extends the existing arena format with BR-specific sections.

## Schema

### Complete Example

```toml
[metadata]
name = "BR Arena Alpha"
version = "1.0"
size = [64, 32, 64]
game_mode = "br_lite"

[br_lite]
min_players = 4
post_match_delay = 10
zone_center = [32.0, 32.0]
initial_radius = 30.0

[[br_lite.phases]]
stable_duration = 60
shrink_duration = 30
end_radius = 24.0
damage_per_tick = 5

[[br_lite.phases]]
stable_duration = 45
shrink_duration = 25
end_radius = 18.0
damage_per_tick = 10

[[br_lite.phases]]
stable_duration = 30
shrink_duration = 20
end_radius = 12.0
damage_per_tick = 15

[[br_lite.phases]]
stable_duration = 20
shrink_duration = 15
end_radius = 6.0
damage_per_tick = 25

[[br_lite.phases]]
stable_duration = 15
shrink_duration = 10
end_radius = 2.0
damage_per_tick = 50

[[spawn_points]]
team = 0
position = [10.0, 2.0, 10.0]
rotation = 45.0

[[spawn_points]]
team = 0
position = [54.0, 2.0, 10.0]
rotation = 135.0

[[spawn_points]]
team = 0
position = [10.0, 2.0, 54.0]
rotation = 315.0

[[spawn_points]]
team = 0
position = [54.0, 2.0, 54.0]
rotation = 225.0

[[loot_spawns]]
position = [16.0, 2.0, 16.0]
type = "health_pack"
heal_amount = 25

[[loot_spawns]]
position = [48.0, 2.0, 16.0]
type = "health_pack"
heal_amount = 25

[[loot_spawns]]
position = [16.0, 2.0, 48.0]
type = "speed_boost"
multiplier = 1.5
duration = 10

[[loot_spawns]]
position = [48.0, 2.0, 48.0]
type = "speed_boost"
multiplier = 1.5
duration = 10

[[loot_spawns]]
position = [32.0, 2.0, 32.0]
type = "health_pack"
heal_amount = 50

[blocks]
# ... standard block definitions
```

## Field Reference

### [metadata]

Standard arena metadata fields.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Arena display name |
| `version` | string | Yes | Arena version |
| `size` | [u32; 3] | Yes | Arena dimensions [x, y, z] |
| `game_mode` | string | Yes | Must be `"br_lite"` for BR Lite |

### [br_lite]

BR Lite specific configuration.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `min_players` | u8 | No | 4 | Minimum players to start |
| `post_match_delay` | u32 | No | 10 | PostMatch screen duration (seconds) |
| `zone_center` | [f32; 2] | No | arena center | Zone center [x, z] |
| `initial_radius` | f32 | No | min(size.x, size.z) / 2 | Starting zone radius |

**Validation**:
- `min_players >= 2`
- `min_players <= 16`
- `initial_radius > 0`
- `zone_center` must be within arena bounds

### [[br_lite.phases]]

Zone phase configuration (array, at least 1 required).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `stable_duration` | u32 | Yes | Seconds zone is stable |
| `shrink_duration` | u32 | Yes | Seconds to shrink to target |
| `end_radius` | f32 | Yes | Target radius after shrink |
| `damage_per_tick` | u32 | Yes | Damage per second outside zone |

**Validation**:
- `stable_duration >= 0`
- `shrink_duration > 0`
- `end_radius >= 0`
- `damage_per_tick > 0`
- Each phase `end_radius` must be smaller than previous (zone shrinks)
- First phase `end_radius` must be smaller than `initial_radius`

**Ordering**: Phases are processed in array order (first = earliest).

### [[spawn_points]]

Player spawn points (FFA mode, team = 0 for all).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `team` | u8 | Yes | Team ID (use 0 for BR Lite) |
| `position` | [f32; 3] | Yes | Spawn position [x, y, z] |
| `rotation` | f32 | Yes | Initial yaw rotation (degrees) |

**BR Lite Specifics**:
- All spawns should use `team = 0` (FFA mode)
- Spawns should be spread around the arena perimeter
- Minimum `min_players` spawn points required
- Recommended: 2x `min_players` spawn points for variety

### [[loot_spawns]]

Loot item spawn definitions.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `position` | [f32; 3] | Yes | World position [x, y, z] |
| `type` | string | Yes | `"health_pack"` or `"speed_boost"` |
| `heal_amount` | u32 | For health_pack | HP restored |
| `multiplier` | f32 | For speed_boost | Speed multiplier (e.g., 1.5) |
| `duration` | u32 | For speed_boost | Effect duration (seconds) |

**Validation**:
- `type` must be `"health_pack"` or `"speed_boost"`
- `health_pack` requires `heal_amount > 0`
- `speed_boost` requires `multiplier > 1.0` and `duration > 0`
- Position must be within arena bounds

## Defaults

### Default Phase Schedule

When `[[br_lite.phases]]` is empty, use:

```toml
[[br_lite.phases]]
stable_duration = 60
shrink_duration = 30
end_radius = 0.8  # 80% of initial
damage_per_tick = 5

[[br_lite.phases]]
stable_duration = 45
shrink_duration = 25
end_radius = 0.6  # 60% of initial
damage_per_tick = 10

[[br_lite.phases]]
stable_duration = 30
shrink_duration = 20
end_radius = 0.4  # 40% of initial
damage_per_tick = 15

[[br_lite.phases]]
stable_duration = 20
shrink_duration = 15
end_radius = 0.2  # 20% of initial
damage_per_tick = 25

[[br_lite.phases]]
stable_duration = 15
shrink_duration = 10
end_radius = 0.05  # 5% of initial
damage_per_tick = 50
```

**Note**: When using percentage-based defaults, `end_radius` is computed as `initial_radius * percentage` at load time.

### Default Loot Layout

When `[[loot_spawns]]` is empty, no loot spawns (loot is optional).

## Rust Parsing Structs

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

fn default_min_players() -> u8 { 4 }
fn default_post_match_delay() -> u32 { 10 }

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

## Validation Errors

| Error | Condition | Message |
|-------|-----------|---------|
| `InvalidMinPlayers` | `min_players < 2` or `> 16` | "min_players must be between 2 and 16" |
| `InvalidRadius` | `initial_radius <= 0` | "initial_radius must be positive" |
| `NoPhases` | `phases.is_empty()` after defaults | "at least one zone phase required" |
| `InvalidPhaseShrink` | phase[n].end_radius >= phase[n-1].end_radius | "zone phases must shrink" |
| `InvalidLootType` | unknown loot type | "unknown loot type: {type}" |
| `MissingLootParam` | health_pack without heal_amount | "health_pack requires heal_amount" |
| `InsufficientSpawns` | spawn_points.len() < min_players | "need at least {min_players} spawn points" |

## Migration

Existing arenas with `game_mode = "tdm"` or `"ffa"` are unaffected. The `[br_lite]` section is only parsed when `game_mode = "br_lite"`.
