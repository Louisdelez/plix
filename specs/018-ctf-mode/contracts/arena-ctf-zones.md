# Contract: Arena CTF Zone Loading

**Module**: `plix-arena/src/format.rs`, `plix-arena/src/loader.rs`
**Date**: 2025-12-16

## Purpose

Defines and validates CTF zone definitions loaded from arena TOML files.

## Interface

### CtfArenaConfig

```rust
/// CTF-specific arena configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CtfArenaConfig {
    pub capture_limit: Option<u16>,
    pub flag_return_delay: Option<u32>,
    pub respawn_delay: Option<u32>,
    pub time_limit: Option<u32>,
    pub flag_bases: Vec<CtfZoneDef>,
    pub capture_zones: Vec<CtfZoneDef>,
}

/// Zone definition from TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfZoneDef {
    pub team: u8,
    pub min: [f32; 3],
    pub max: [f32; 3],
}
```

### Arena Extension

```rust
impl Arena {
    /// Get CTF configuration if arena is CTF mode
    pub fn ctf_config(&self) -> Option<&CtfArenaConfig>;

    /// Convert CTF zone definitions to runtime FlagZone objects
    pub fn ctf_zones(&self) -> Vec<FlagZone>;
}
```

### Loader Extension

```rust
/// Load arena with CTF zone validation
pub fn load_arena(path: &Path) -> Result<LoadedArena, ArenaError>;
```

### Validator Extension

```rust
/// Validate CTF arena has required zones
pub fn validate_ctf_arena(arena: &Arena) -> Result<(), ArenaValidationError>;
```

## Behavior Contracts

### BC-301: Arena Loading

**Preconditions**:
- TOML file exists and is valid TOML
- `[metadata]` section present with `game_mode`

**Postconditions**:
- If `game_mode = "ctf"`:
  - `[ctf]` section parsed into `CtfArenaConfig`
  - Default values used for missing optional fields
- If `game_mode != "ctf"`:
  - `ctf_config()` returns `None`

### BC-302: CTF Arena Validation

**Preconditions**:
- Arena loaded successfully
- `game_mode = "ctf"`

**Validation Rules**:
1. Exactly one `flag_base` zone for team 0
2. Exactly one `flag_base` zone for team 1
3. Exactly one `capture_zone` zone for team 0
4. Exactly one `capture_zone` zone for team 1
5. All zones within arena bounds
6. `min` < `max` for all zones (valid AABB)

**Postconditions**:
- Returns `Ok(())` if all rules pass
- Returns `Err(ArenaValidationError)` with specific failure reason

### BC-303: Zone Conversion

**Preconditions**:
- Arena is valid CTF arena

**Postconditions**:
- Returns `Vec<FlagZone>` with:
  - 2 `FlagBase` zones (one per team)
  - 2 `CaptureZone` zones (one per team)
- Each zone has correct `TeamId` and `FlagZoneType`
- Coordinates converted from `[f32; 3]` to `Vec3`

### BC-304: Config Override

**Preconditions**:
- Arena loaded with optional CTF config values

**Behavior**:
- If field present in TOML: use that value
- If field absent: use server default from `CtfConfig::default()`

## TOML Schema

```toml
[metadata]
name = "string"           # required
version = "string"        # required
size = [u32, u32, u32]    # required
game_mode = "ctf"         # required for CTF

[ctf]
capture_limit = 3           # optional, default: 3
flag_return_delay = 10      # optional, seconds, default: 10
respawn_delay = 5           # optional, seconds, default: 5
time_limit = 600            # optional, seconds, default: 600

[[ctf.flag_bases]]          # required: 2 entries (one per team)
team = 0                    # required: 0 or 1
min = [0.0, 0.0, 0.0]       # required
max = [10.0, 4.0, 10.0]     # required

[[ctf.capture_zones]]       # required: 2 entries (one per team)
team = 0                    # required: 0 or 1
min = [0.0, 0.0, 0.0]       # required
max = [12.0, 4.0, 12.0]     # required
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ArenaValidationError {
    #[error("CTF arena missing flag base for team {0}")]
    MissingFlagBase(u8),

    #[error("CTF arena missing capture zone for team {0}")]
    MissingCaptureZone(u8),

    #[error("CTF arena has multiple flag bases for team {0}")]
    DuplicateFlagBase(u8),

    #[error("CTF arena has multiple capture zones for team {0}")]
    DuplicateCaptureZone(u8),

    #[error("Zone min must be less than max: min={min:?}, max={max:?}")]
    InvalidZoneBounds { min: [f32; 3], max: [f32; 3] },

    #[error("Zone outside arena bounds: {zone_type:?} for team {team}")]
    ZoneOutOfBounds { zone_type: FlagZoneType, team: u8 },
}
```

## Test Scenarios

### T-ARENA-001: Valid CTF Arena Loads
```
Given: valid CTF TOML with all required zones
When: load_arena(path)
Then: Ok(arena) with ctf_config() = Some(_)
```

### T-ARENA-002: Missing Flag Base Fails
```
Given: CTF TOML missing team 1 flag base
When: validate_ctf_arena(arena)
Then: Err(MissingFlagBase(1))
```

### T-ARENA-003: Missing Capture Zone Fails
```
Given: CTF TOML missing team 0 capture zone
When: validate_ctf_arena(arena)
Then: Err(MissingCaptureZone(0))
```

### T-ARENA-004: Duplicate Zone Fails
```
Given: CTF TOML with two flag bases for team 0
When: validate_ctf_arena(arena)
Then: Err(DuplicateFlagBase(0))
```

### T-ARENA-005: Invalid AABB Fails
```
Given: zone with min > max
When: validate_ctf_arena(arena)
Then: Err(InvalidZoneBounds { ... })
```

### T-ARENA-006: Default Config Applied
```
Given: CTF TOML without capture_limit specified
When: load_arena and build CtfConfig
Then: config.capture_limit == 3 (default)
```

### T-ARENA-007: Override Config Applied
```
Given: CTF TOML with capture_limit = 5
When: load_arena and build CtfConfig
Then: config.capture_limit == 5
```

### T-ARENA-008: Non-CTF Arena Has No CTF Config
```
Given: arena with game_mode = "tdm"
When: arena.ctf_config()
Then: returns None
```
