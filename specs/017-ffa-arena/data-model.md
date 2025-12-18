# Data Model: FFA Arena Mode

**Feature**: 017-ffa-arena | **Date**: 2025-12-16

## Entity Overview

This feature introduces one new type (`GameMode`) and modifies two existing entities (`ArenaMetadata`, `MatchState`). All other entities are reused from TDM implementation.

## New Types

### GameMode (NEW)

**Location**: `plix-common/src/types.rs`

```rust
/// Game mode for arena matches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GameMode {
    /// Team Deathmatch - teams score points for kills
    #[default]
    Tdm,
    /// Free-for-All - individual players score points for kills
    Ffa,
}
```

**Fields**:
| Variant | Description | Default |
|---------|-------------|---------|
| `Tdm` | Team Deathmatch mode | Yes (backward compat) |
| `Ffa` | Free-for-All mode | No |

**Validation Rules**:
- Must be one of the defined variants
- Unknown values in TOML fail arena load with clear error

## Modified Types

### ArenaMetadata (MODIFIED)

**Location**: `plix-arena/src/format.rs`

**Changes**: Add `game_mode` field

```rust
/// Arena metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArenaMetadata {
    /// Arena name
    pub name: String,
    /// Arena version
    pub version: String,
    /// Arena dimensions (x, y, z)
    pub size: [u32; 3],
    /// Game mode for this arena (NEW)
    #[serde(default)]
    pub game_mode: GameMode,
}
```

**New Field**:
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `game_mode` | `GameMode` | No | `Tdm` | Determines scoring rules |

**Validation Rules**:
- Field is optional (defaults to TDM for backward compatibility)
- If present, must be valid GameMode variant
- Arena validation logs info-level message about detected mode

### MatchState (MODIFIED)

**Location**: `plix-common/src/protocol/messages.rs`

**Changes**: Add `game_mode` field for client awareness

```rust
/// Match state broadcast to all clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: MatchPhase,
    pub countdown_remaining: u8,
    pub time_remaining: u32,
    pub score_limit: u16,
    pub player_scores: Vec<PlayerScore>,
    pub winner: Option<PlayerId>,
    pub arena_name: String,
    pub scores: Vec<TeamScore>,
    pub team_winner: Option<TeamId>,
    /// Game mode for this match (NEW)
    pub game_mode: GameMode,
}
```

**New Field**:
| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `game_mode` | `GameMode` | Yes | N/A | Set from arena config on match init |

### MatchConfig (MODIFIED)

**Location**: `plix-server/src/match_state.rs`

**Changes**: Add constructor for FFA defaults

```rust
impl MatchConfig {
    /// Create FFA-specific default configuration
    pub fn ffa_default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,      // 3 seconds at 60 Hz
            time_limit_seconds: 300,   // 5 minutes
            score_limit: 15,           // 15 kills to win (FFA standard per FR-020)
            end_screen_ticks: 600,     // 10 seconds at 60 Hz (FR-022)
            respawn_delay_ticks: 180,  // 3 seconds at 60 Hz (FR-021)
            arena_rotation: Vec::new(),
            team_size: 0,              // Not applicable for FFA
        }
    }
}
```

## Existing Types (Reused)

These types are used as-is from existing implementation:

### PlayerScore

**Location**: `plix-common/src/protocol/messages.rs`

```rust
/// Individual player score (used by both TDM and FFA)
pub struct PlayerScore {
    pub player_id: PlayerId,
    pub name: String,
    pub kills: u16,
    pub deaths: u16,
}
```

**Usage in FFA**: Primary scoring entity. `kills` field determines winner.

### MatchPhase

**Location**: `plix-common/src/protocol/messages.rs`

```rust
pub enum MatchPhase {
    Lobby,
    Countdown,
    Playing,
    EndScreen,
    Resetting,
}
```

**Usage in FFA**: Identical state machine to TDM.

### SpawnPoint

**Location**: `plix-arena/src/format.rs`

```rust
pub struct SpawnPoint {
    pub team: u8,
    pub position: [f32; 3],
    pub rotation: f32,
}
```

**Usage in FFA**: `team` field is ignored. All spawns are treated as neutral.

## State Transitions

### FFA Match State Machine

```
┌─────────┐
│  Lobby  │◄──────────────────────────────────────┐
└────┬────┘                                       │
     │ min_players ready                          │
     ▼                                            │
┌─────────┐                                       │
│Countdown│                                       │
└────┬────┘                                       │
     │ countdown_ticks == 0                       │
     ▼                                            │
┌─────────┐                                       │
│ Playing │                                       │
└────┬────┘                                       │
     │ player.kills >= score_limit                │
     │    OR time_remaining == 0                  │
     ▼                                            │
┌──────────┐                                      │
│EndScreen │                                      │
└────┬─────┘                                      │
     │ end_screen_ticks == 0                      │
     ▼                                            │
┌──────────┐                                      │
│Resetting │──────────────────────────────────────┘
└──────────┘    reset scores, clear winner
```

### Kill Event Processing (FFA Branch)

```
Kill Event Received
        │
        ▼
┌───────────────────┐
│ phase == Playing? │──No──► Ignore kill
└────────┬──────────┘
         │ Yes
         ▼
┌───────────────────┐
│ attacker == victim?│──Yes──► No score (suicide)
└────────┬──────────┘
         │ No
         ▼
┌───────────────────┐
│ game_mode == FFA? │──No──► TDM scoring path
└────────┬──────────┘
         │ Yes
         ▼
┌───────────────────────────┐
│ update_player_score(+1)   │
│ check_score_limit()       │
└────────────┬──────────────┘
             │
             ▼
┌───────────────────────────┐
│ kills >= score_limit?     │──Yes──► transition EndScreen
└────────────┬──────────────┘         set winner = attacker
             │ No
             ▼
         Continue
```

## Data Flow

### Arena Load → Match Init

```
test_arena.toml          plix-arena             plix-server
      │                      │                       │
      │   [metadata]         │                       │
      │   game_mode = "ffa"  │                       │
      │──────────────────────►                       │
      │                      │  Arena {             │
      │                      │    metadata: {       │
      │                      │      game_mode: Ffa  │
      │                      │    }                 │
      │                      │  }                   │
      │                      │───────────────────────►
      │                      │                       │ MatchStateMachine::new()
      │                      │                       │   state.game_mode = arena.game_mode
      │                      │                       │   config = MatchConfig::ffa_default()
```

### Score Broadcast

```
Kill Event                                  WorldSnapshot
    │                                            │
    ▼                                            │
update_player_score()                            │
    │                                            │
    ▼                                            │
player_scores: [                                 │
  { player_id: 1, kills: 5, deaths: 2 },        │
  { player_id: 2, kills: 3, deaths: 4 },        │
]                                                │
    │                                            │
    ▼                                            │
MatchState {                                     │
  game_mode: Ffa,                                │
  player_scores: [...],                          │
  winner: None,                                  │
}──────────────────────────────────────────────► Client
```

## Relationships

```
┌─────────────────┐       ┌─────────────────┐
│   GameMode      │◄──────│ ArenaMetadata   │
│   (enum)        │       │   game_mode     │
└────────┬────────┘       └─────────────────┘
         │
         │ copied to
         ▼
┌─────────────────┐       ┌─────────────────┐
│   MatchState    │───────│ MatchConfig     │
│   game_mode     │       │   (defaults)    │
└────────┬────────┘       └─────────────────┘
         │
         │ determines
         ▼
┌─────────────────┐
│ Scoring Logic   │
│ (branch point)  │
└─────────────────┘
```

## TOML Schema

### FFA Arena Example

```toml
[metadata]
name = "FFA Arena"
version = "1.0.0"
size = [64, 32, 64]
game_mode = "ffa"      # NEW FIELD

# All spawn points are neutral in FFA (team field ignored)
[[spawn_points]]
team = 0               # Ignored for FFA
position = [10.0, 1.0, 10.0]
rotation = 0.0

[[spawn_points]]
team = 0
position = [32.0, 1.0, 10.0]
rotation = 90.0

[[spawn_points]]
team = 0
position = [54.0, 1.0, 10.0]
rotation = 180.0

# ... more spawns distributed across arena

[blocks]
floor = { y = 0, block = "stone" }
walls = { border = true, height = 8, block = "brick" }
```

### TDM Arena Example (Backward Compatible)

```toml
[metadata]
name = "Test Arena"
version = "0.1.0"
size = [64, 32, 64]
# game_mode omitted = defaults to "tdm"

[[spawn_points]]
team = 0               # Team 0 spawn
position = [10.0, 1.0, 10.0]
rotation = 45.0

[[spawn_points]]
team = 1               # Team 1 spawn
position = [50.0, 1.0, 50.0]
rotation = 225.0

# ... existing arena content
```
