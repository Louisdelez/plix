# Data Model: Matchmaking v1 (Quick Join)

**Feature**: 027-matchmaking-v1
**Date**: 2025-12-17

## Entities

### QuickJoinRequest

Represents a matchmaking request initiated by the player.

```rust
/// A request to quickly join a game server.
#[derive(Debug, Clone)]
pub struct QuickJoinRequest {
    /// Requested game mode (e.g., "tdm", "ffa", "any")
    pub mode: String,
    /// Requested region (e.g., "eu", "us", "any")
    pub region: String,
}

impl QuickJoinRequest {
    /// Valid mode values (case-insensitive)
    pub const VALID_MODES: &'static [&'static str] = &["tdm", "ffa", "ctf", "br", "training", "any"];

    /// Valid region values (case-insensitive)
    pub const VALID_REGIONS: &'static [&'static str] = &["eu", "us", "asia", "any"];
}
```

**Validation Rules**:
- Mode must be one of: tdm, ffa, ctf, br, training, any (case-insensitive)
- Region must be one of: eu, us, asia, any (case-insensitive)
- Both values normalized to lowercase during parsing

### ServerScore

Represents the scoring result for a single server candidate.

```rust
/// Scoring result for a server candidate.
#[derive(Debug, Clone)]
pub struct ServerScore {
    /// Reference to the scored server
    pub server: ServerEntry,
    /// Total computed score
    pub total_score: i32,
    /// Score breakdown for debugging/logging
    pub breakdown: ScoreBreakdown,
}

/// Breakdown of score components.
#[derive(Debug, Clone, Default)]
pub struct ScoreBreakdown {
    /// +50 if region matches request
    pub region_bonus: i32,
    /// +30 if server is 1-80% capacity
    pub capacity_bonus: i32,
    /// +20 if last_seen < 30 seconds ago
    pub freshness_bonus: i32,
    /// +1 per player (up to 80% capacity)
    pub player_bonus: i32,
    /// +40 if ping < 50ms, +20 if ping < 100ms (optional)
    pub ping_bonus: i32,
}
```

**Scoring Rules** (from FR-009 to FR-011):
| Component | Condition | Points |
|-----------|-----------|--------|
| region_bonus | region matches request | +50 |
| capacity_bonus | player_count in 1-80% of max_players | +30 |
| freshness_bonus | last_seen within 30 seconds | +20 |
| player_bonus | per player (capped at 80% capacity) | +1 each |
| ping_bonus | ping < 50ms | +40 |
| ping_bonus | ping 50-100ms | +20 |

### MatchmakingPreferences

User's saved preferences for quick join, persisted to profile.toml.

```rust
/// User preferences for quick join matchmaking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchmakingPreferences {
    /// Preferred game mode (default: "tdm")
    #[serde(default = "default_mode")]
    pub preferred_mode: String,
    /// Preferred region (default: "any")
    #[serde(default = "default_region")]
    pub preferred_region: String,
}

fn default_mode() -> String { "tdm".to_string() }
fn default_region() -> String { "any".to_string() }

impl Default for MatchmakingPreferences {
    fn default() -> Self {
        Self {
            preferred_mode: default_mode(),
            preferred_region: default_region(),
        }
    }
}
```

**Persistence**: Stored in `[matchmaking]` section of `~/.config/plix/profile.toml`

**Example profile.toml**:
```toml
version = 1
display_name = "PlayerOne"

[matchmaking]
preferred_mode = "tdm"
preferred_region = "eu"
```

### QuickJoinResult

Outcome of a quick join attempt.

```rust
/// Result of a quick join attempt.
#[derive(Debug, Clone)]
pub struct QuickJoinResult {
    /// Selected server (if successful)
    pub selected_server: Option<ServerEntry>,
    /// Whether fallback criteria were used
    pub fallback_used: bool,
    /// Reason for fallback (if applicable)
    pub fallback_reason: Option<String>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Number of connection attempts made
    pub attempts: u8,
}

impl QuickJoinResult {
    /// Maximum connection attempts before giving up
    pub const MAX_ATTEMPTS: u8 = 3;
}
```

**State Transitions**:
```
QuickJoinRequest
    ↓
[Fetch server list]
    ↓
[Filter by mode/region/version/capacity]
    ↓ (no servers)        ↓ (servers found)
[Fallback: region=any] → [Score servers]
    ↓ (no servers)              ↓
[Fallback: mode=any]    [Select best (tie-break)]
    ↓ (no servers)              ↓
[Error: No servers]     [Connect attempt]
                              ↓ (success)    ↓ (failure)
                        [QuickJoinResult]  [Retry if < 3 attempts]
                                                 ↓ (exhausted)
                                           [QuickJoinResult with error]
```

## Existing Entities (Extended)

### PlayerProfile (Feature 025)

Extended to include matchmaking preferences.

```rust
/// Player profile stored locally (extended for Feature 027).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerProfile {
    pub version: u32,
    pub display_name: String,
    #[serde(default)]
    pub account_id: Option<u64>,
    #[serde(default)]
    pub auth_token: Option<String>,
    /// NEW: Matchmaking preferences
    #[serde(default)]
    pub matchmaking: MatchmakingPreferences,
}
```

**Backward Compatibility**: The `#[serde(default)]` attribute ensures existing profiles without `[matchmaking]` section load successfully with default values.

### ServerEntry (Feature 026)

Used as-is from plix-common. Relevant fields for scoring:

| Field | Type | Used For |
|-------|------|----------|
| server_id | String | Failed server exclusion |
| region | String | Region matching |
| player_count | u8 | Capacity/player scoring |
| max_players | u8 | Capacity calculation |
| game_modes | Vec<String> | Mode filtering |
| protocol_version | String | Version filtering |
| last_seen | u64 | Freshness scoring |
| host | String | Connection |
| port | u16 | Connection |

## Relationships

```
┌─────────────────────────────────────────────────────────────┐
│                      PlayerProfile                          │
│  (persisted to ~/.config/plix/profile.toml)                │
│                                                             │
│  ├── display_name: String                                   │
│  └── matchmaking: MatchmakingPreferences ◄────────────────┐│
│                                                            ││
└────────────────────────────────────────────────────────────┘│
                                                               │
┌─────────────────────────────────────────────────────────────┐│
│                   QuickJoinRequest                          ││
│  (transient, created per /quickjoin command)               ││
│                                                             ││
│  ├── mode: String   ◄─── defaults from ─────────────────────┘
│  └── region: String ◄─── preferences                        │
└────────────────────────────────────────────────────────────┘
           │
           │ filters & scores
           ▼
┌─────────────────────────────────────────────────────────────┐
│                    ServerEntry[]                            │
│  (from Feature 026 BrowserState)                           │
│                                                             │
│  Filtered by: mode, region, protocol_version, capacity      │
│  Scored into: ServerScore[]                                │
└────────────────────────────────────────────────────────────┘
           │
           │ select best
           ▼
┌─────────────────────────────────────────────────────────────┐
│                    QuickJoinResult                          │
│  (returned to caller)                                       │
│                                                             │
│  ├── selected_server: Option<ServerEntry>                   │
│  ├── fallback_used: bool                                    │
│  ├── fallback_reason: Option<String>                        │
│  ├── error_message: Option<String>                          │
│  └── attempts: u8                                           │
└────────────────────────────────────────────────────────────┘
```
