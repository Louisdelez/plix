# Data Model: CTF Mode (Capture The Flag)

**Feature**: 018-ctf-mode | **Date**: 2025-12-16

## Overview

This document defines the data structures for CTF mode, extending the existing plix type system.

## Core Types

### FlagState

Represents the current state of a flag in the game.

```rust
// Location: crates/plix-common/src/types.rs

/// State of a team's flag in CTF mode
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlagState {
    /// Flag is at its home base
    AtBase,
    /// Flag is being carried by a player
    Carried {
        carrier: PlayerId,
    },
    /// Flag was dropped and is on the ground
    Dropped {
        position: Vec3,
        return_tick: Tick,
    },
}

impl Default for FlagState {
    fn default() -> Self {
        FlagState::AtBase
    }
}
```

### Flag

Complete flag entity with team ownership and state.

```rust
// Location: crates/plix-common/src/types.rs

/// A team's flag in CTF mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flag {
    /// Team that owns this flag
    pub team: TeamId,
    /// Current state of the flag
    pub state: FlagState,
    /// Base position where flag spawns/returns
    pub base_position: Vec3,
}

impl Flag {
    pub fn new(team: TeamId, base_position: Vec3) -> Self {
        Self {
            team,
            state: FlagState::AtBase,
            base_position,
        }
    }

    /// Get current position of the flag
    pub fn position(&self) -> Vec3 {
        match &self.state {
            FlagState::AtBase => self.base_position,
            FlagState::Carried { .. } => Vec3::ZERO, // Position comes from carrier
            FlagState::Dropped { position, .. } => *position,
        }
    }

    /// Check if flag is at its base
    pub fn is_at_base(&self) -> bool {
        matches!(self.state, FlagState::AtBase)
    }

    /// Check if flag is being carried
    pub fn is_carried(&self) -> bool {
        matches!(self.state, FlagState::Carried { .. })
    }

    /// Get carrier ID if flag is being carried
    pub fn carrier(&self) -> Option<PlayerId> {
        match &self.state {
            FlagState::Carried { carrier } => Some(*carrier),
            _ => None,
        }
    }
}
```

### FlagZone

Defines spatial zones for flag interactions.

```rust
// Location: crates/plix-common/src/types.rs

/// Type of CTF zone
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlagZoneType {
    /// Where the flag spawns and can be picked up
    FlagBase,
    /// Where enemy flag must be brought to capture
    CaptureZone,
}

/// A spatial zone for CTF flag interactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagZone {
    /// Team that owns this zone
    pub team: TeamId,
    /// Type of zone (flag base or capture zone)
    pub zone_type: FlagZoneType,
    /// Minimum corner of bounding box
    pub min: Vec3,
    /// Maximum corner of bounding box
    pub max: Vec3,
}

impl FlagZone {
    /// Check if a point is inside this zone
    pub fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    /// Get the center of the zone
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
}
```

### GameMode Extension

```rust
// Location: crates/plix-common/src/types.rs (modify existing)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum GameMode {
    #[default]
    Tdm,
    Ffa,
    Ctf,  // NEW
}

impl std::fmt::Display for GameMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameMode::Tdm => write!(f, "tdm"),
            GameMode::Ffa => write!(f, "ffa"),
            GameMode::Ctf => write!(f, "ctf"),  // NEW
        }
    }
}
```

## Configuration Types

### CtfConfig

Server-side configuration for CTF matches.

```rust
// Location: crates/plix-server/src/ctf/mod.rs

/// Configuration for CTF game mode
#[derive(Debug, Clone)]
pub struct CtfConfig {
    /// Number of captures to win (default: 3)
    pub capture_limit: u16,
    /// Ticks before dropped flag returns to base (default: 600 = 10s at 60Hz)
    pub flag_return_delay_ticks: u32,
    /// Ticks before respawning after death (default: 300 = 5s at 60Hz)
    pub respawn_delay_ticks: u32,
    /// Match time limit in seconds (default: 600 = 10 minutes)
    pub time_limit_seconds: u32,
    /// End screen duration in ticks (default: 600 = 10s at 60Hz)
    pub end_screen_ticks: u32,
}

impl Default for CtfConfig {
    fn default() -> Self {
        Self {
            capture_limit: 3,
            flag_return_delay_ticks: 600,  // 10 seconds at 60Hz
            respawn_delay_ticks: 300,      // 5 seconds at 60Hz
            time_limit_seconds: 600,       // 10 minutes
            end_screen_ticks: 600,         // 10 seconds at 60Hz
        }
    }
}
```

## Server State Types

### CtfState

Complete CTF game state managed by server.

```rust
// Location: crates/plix-server/src/ctf/state.rs

/// Complete CTF game state
#[derive(Debug, Clone)]
pub struct CtfState {
    /// Flags for each team (indexed by TeamId)
    pub flags: [Flag; 2],
    /// Capture scores for each team
    pub capture_scores: [u32; 2],
    /// Flag zones loaded from arena
    pub zones: Vec<FlagZone>,
    /// Configuration
    pub config: CtfConfig,
}

impl CtfState {
    pub fn new(zones: Vec<FlagZone>, config: CtfConfig) -> Self {
        // Find flag base positions from zones
        let team0_base = zones.iter()
            .find(|z| z.team == TeamId::TEAM_0 && z.zone_type == FlagZoneType::FlagBase)
            .map(|z| z.center())
            .unwrap_or(Vec3::ZERO);

        let team1_base = zones.iter()
            .find(|z| z.team == TeamId::TEAM_1 && z.zone_type == FlagZoneType::FlagBase)
            .map(|z| z.center())
            .unwrap_or(Vec3::ZERO);

        Self {
            flags: [
                Flag::new(TeamId::TEAM_0, team0_base),
                Flag::new(TeamId::TEAM_1, team1_base),
            ],
            capture_scores: [0, 0],
            zones,
            config,
        }
    }

    /// Get flag for a team
    pub fn flag(&self, team: TeamId) -> &Flag {
        &self.flags[team.0 as usize]
    }

    /// Get mutable flag for a team
    pub fn flag_mut(&mut self, team: TeamId) -> &mut Flag {
        &mut self.flags[team.0 as usize]
    }

    /// Get capture score for a team
    pub fn score(&self, team: TeamId) -> u32 {
        self.capture_scores[team.0 as usize]
    }

    /// Get flag base zone for a team
    pub fn flag_base(&self, team: TeamId) -> Option<&FlagZone> {
        self.zones.iter()
            .find(|z| z.team == team && z.zone_type == FlagZoneType::FlagBase)
    }

    /// Get capture zone for a team
    pub fn capture_zone(&self, team: TeamId) -> Option<&FlagZone> {
        self.zones.iter()
            .find(|z| z.team == team && z.zone_type == FlagZoneType::CaptureZone)
    }

    /// Reset state for new match
    pub fn reset(&mut self) {
        for flag in &mut self.flags {
            flag.state = FlagState::AtBase;
        }
        self.capture_scores = [0, 0];
    }
}
```

## Protocol Messages

### CTF-Specific Messages

```rust
// Location: crates/plix-common/src/protocol/messages.rs

/// CTF flag state update (Server -> Client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfFlagUpdate {
    /// Team whose flag state changed
    pub team: TeamId,
    /// New flag state
    pub state: FlagState,
    /// Base position of the flag
    pub base_position: Vec3,
}

/// CTF capture event (Server -> Client)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfCaptureEvent {
    /// Team that captured the flag
    pub capturing_team: TeamId,
    /// Player who captured the flag
    pub capturing_player: PlayerId,
    /// Current capture scores [team0, team1]
    pub scores: [u32; 2],
}

/// CTF match state (included in MatchState broadcast)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfMatchInfo {
    /// Flag states for both teams
    pub flags: [CtfFlagUpdate; 2],
    /// Capture scores
    pub scores: [u32; 2],
    /// Capture limit to win
    pub capture_limit: u16,
}
```

## Arena Format Extension

### CTF Zone Definitions in TOML

```toml
# Example: assets/arenas/ctf_arena.toml

[metadata]
name = "CTF Arena"
version = "1.0.0"
size = [64, 32, 64]
game_mode = "ctf"

# CTF-specific configuration
[ctf]
capture_limit = 3
flag_return_delay = 10  # seconds
respawn_delay = 5       # seconds
time_limit = 600        # seconds

# Team 0 (Red) flag base - where flag spawns
[[ctf.flag_bases]]
team = 0
min = [8, 0, 28]
max = [16, 4, 36]

# Team 1 (Blue) flag base
[[ctf.flag_bases]]
team = 1
min = [48, 0, 28]
max = [56, 4, 36]

# Team 0 capture zone - where team 0 brings enemy flag
[[ctf.capture_zones]]
team = 0
min = [4, 0, 24]
max = [12, 4, 40]

# Team 1 capture zone
[[ctf.capture_zones]]
team = 1
min = [52, 0, 24]
max = [60, 4, 40]
```

### Arena Format Structs

```rust
// Location: crates/plix-arena/src/format.rs

/// CTF-specific arena configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CtfArenaConfig {
    /// Capture limit override (None = use default)
    #[serde(default)]
    pub capture_limit: Option<u16>,
    /// Flag return delay in seconds (None = use default)
    #[serde(default)]
    pub flag_return_delay: Option<u32>,
    /// Respawn delay in seconds (None = use default)
    #[serde(default)]
    pub respawn_delay: Option<u32>,
    /// Time limit in seconds (None = use default)
    #[serde(default)]
    pub time_limit: Option<u32>,
    /// Flag base zones
    #[serde(default)]
    pub flag_bases: Vec<CtfZoneDef>,
    /// Capture zones
    #[serde(default)]
    pub capture_zones: Vec<CtfZoneDef>,
}

/// CTF zone definition in arena TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtfZoneDef {
    /// Team that owns this zone
    pub team: u8,
    /// Minimum corner
    pub min: [f32; 3],
    /// Maximum corner
    pub max: [f32; 3],
}

impl CtfZoneDef {
    /// Convert to FlagZone
    pub fn to_flag_zone(&self, zone_type: FlagZoneType) -> FlagZone {
        FlagZone {
            team: TeamId(self.team),
            zone_type,
            min: Vec3::new(self.min[0], self.min[1], self.min[2]),
            max: Vec3::new(self.max[0], self.max[1], self.max[2]),
        }
    }
}
```

## Type Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                        plix-common                               │
│  ┌─────────┐  ┌───────────┐  ┌──────────┐  ┌─────────────────┐ │
│  │GameMode │  │ FlagState │  │   Flag   │  │    FlagZone     │ │
│  │  ::Ctf  │  │  AtBase   │  │  team    │  │  team           │ │
│  └─────────┘  │  Carried  │  │  state   │  │  zone_type      │ │
│               │  Dropped  │  │  base_pos│  │  min/max        │ │
│               └───────────┘  └──────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        plix-server                               │
│  ┌─────────────┐  ┌───────────────┐  ┌───────────────────────┐ │
│  │  CtfConfig  │  │   CtfState    │  │ MatchStateMachine     │ │
│  │capture_limit│  │  flags[2]     │  │  ctf_default()        │ │
│  │return_delay │  │  scores[2]    │  │  check_ctf_victory()  │ │
│  │respawn_delay│  │  zones        │  └───────────────────────┘ │
│  └─────────────┘  └───────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        plix-arena                                │
│  ┌─────────────────┐  ┌─────────────┐                           │
│  │ CtfArenaConfig  │  │ CtfZoneDef  │                           │
│  │  flag_bases[]   │  │  team       │                           │
│  │  capture_zones[]│  │  min/max    │                           │
│  └─────────────────┘  └─────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

## Invariants

1. **Flag State Consistency**: A flag can only be in one state at a time
2. **Carrier Uniqueness**: A player can carry at most one flag
3. **Team Ownership**: Players can only pick up enemy flags, not their own
4. **Zone Validity**: Each team must have exactly one flag_base and one capture_zone in CTF mode
5. **Score Monotonicity**: Capture scores only increase during a match (reset on match reset)
6. **Return Timer**: Dropped flags must have a valid return_tick in the future
