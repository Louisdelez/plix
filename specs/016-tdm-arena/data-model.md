# Data Model: TDM Arena Mode

**Feature**: 016-tdm-arena
**Date**: 2025-12-16
**Purpose**: Define entities, fields, relationships, and state transitions for TDM mode

## Entity Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                      TDM Match System                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│   TdmMatchConfig ──────► MatchStateMachine                      │
│        │                       │                                │
│        │                       ├── MatchState                   │
│        │                       │      ├── phase: MatchPhase     │
│        │                       │      ├── scores: [TeamScore]   │
│        │                       │      └── team_winner: TeamId?  │
│        │                       │                                │
│        └──► score_limit        ├── player_scores: [PlayerScore] │
│        └──► respawn_delay      │                                │
│        └──► reset_delay        └── team scoring methods         │
│                                                                 │
│   PlayerSession ───────────────────────────────────────────────│
│        ├── team: TeamId                                         │
│        ├── is_dead: bool                                        │
│        ├── respawn_tick: Option<Tick>                          │
│        └── spectate_target: Option<PlayerId>  [NEW]            │
│                                                                 │
│   Team ────────────────────────────────────────────────────────│
│        ├── Red (TeamId::TEAM_0)                                 │
│        └── Blue (TeamId::TEAM_1)                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Entities

### Team (Existing - `plix_common::types::TeamId`)

Represents the two competing teams in TDM.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TeamId(pub u8);

impl TeamId {
    pub const NONE: Self = Self(0xFF);   // Spectator / no team
    pub const TEAM_0: Self = Self(0);    // Red team
    pub const TEAM_1: Self = Self(1);    // Blue team
}
```

**Relationships**:
- Players belong to exactly one Team
- Each Team has a TeamScore
- SpawnPoints are assigned to Teams

**Validation**:
- Only TEAM_0 (Red) and TEAM_1 (Blue) valid for TDM
- NONE used for spectators (out of scope for MVP)

---

### TeamScore (Existing - `plix_common::protocol::TeamScore`)

Tracks cumulative score for a team.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamScore {
    pub team: TeamId,    // Which team
    pub score: u32,      // Total kills by team members
}
```

**Relationships**:
- One per Team in MatchState
- Incremented on enemy kills

**Validation**:
- score >= 0
- score <= score_limit triggers match end

**State Transitions**:
- `0` → Lobby/Countdown (reset)
- Increment +1 on each enemy kill during Playing
- Frozen on match end (EndScreen/Resetting)

---

### TdmMatchConfig (New - extends MatchConfig)

Configuration for TDM-specific parameters.

```rust
#[derive(Debug, Clone)]
pub struct MatchConfig {
    // Existing fields
    pub min_players: u8,           // Min players to start (default: 2)
    pub countdown_ticks: u32,      // Countdown duration (default: 180 = 3s)
    pub time_limit_seconds: u32,   // Match time limit (default: 300 = 5min)
    pub score_limit: u16,          // Kills to win (default: 25 for TDM)
    pub end_screen_ticks: u32,     // End screen duration (default: 900 = 15s)
    pub respawn_delay_ticks: u32,  // Respawn delay (default: 180 = 3s)
    pub arena_rotation: Vec<String>,

    // TDM-specific (new or repurposed)
    pub team_size: u8,             // Max players per team (default: 8)
}
```

**Validation Rules**:
- min_players >= 2
- score_limit >= 1 (TDM default: 25)
- respawn_delay_ticks >= 0 (0 = instant respawn)
- end_screen_ticks >= 60 (min 1 second)
- team_size >= 1

**Defaults** (TDM-specific):
```rust
impl MatchConfig {
    pub fn tdm_default() -> Self {
        Self {
            min_players: 2,
            countdown_ticks: 180,      // 3 seconds
            time_limit_seconds: 300,   // 5 minutes
            score_limit: 25,           // 25 kills to win (TDM standard)
            end_screen_ticks: 900,     // 15 seconds (per clarification Q2)
            respawn_delay_ticks: 180,  // 3 seconds
            arena_rotation: Vec::new(),
            team_size: 8,              // 8v8 max
        }
    }
}
```

---

### MatchState (Existing - `plix_common::protocol::MatchState`)

Broadcast state visible to all clients.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: MatchPhase,
    pub countdown_remaining: u8,      // Seconds in Countdown
    pub time_remaining: u32,          // Seconds in Playing
    pub score_limit: u16,             // Target score
    pub player_scores: Vec<PlayerScore>,  // Individual K/D for scoreboard
    pub winner: Option<PlayerId>,     // Individual winner (legacy, keep for compat)
    pub arena_name: String,
    pub scores: Vec<TeamScore>,       // Team scores [Red, Blue]

    // NEW for TDM
    pub team_winner: Option<TeamId>,  // Winning team (None = tie)
}
```

**State Transitions by Phase**:

| Phase | scores | team_winner | time_remaining | Actions Allowed |
|-------|--------|-------------|----------------|-----------------|
| Lobby | [0, 0] | None | time_limit | None (ready up only) |
| Countdown | [0, 0] | None | time_limit | None (wait) |
| Playing | Incrementing | None | Decrementing | Combat, kills score |
| EndScreen | Frozen | Set | Frozen | None |
| Resetting | [0, 0] | None | time_limit | None |

---

### PlayerSession (Existing - `plix_server::session::PlayerSession`)

Server-side player state.

```rust
pub struct PlayerSession {
    // Existing fields
    pub id: PlayerId,
    pub name: String,
    pub addr: SocketAddr,
    pub team: TeamId,           // Already exists
    pub position: Vec3,
    pub rotation: Rotation,
    pub health: u8,
    pub is_dead: bool,          // Already exists
    pub respawn_tick: Option<Tick>,  // Already exists
    pub kills: u16,             // Individual kills (for scoreboard)
    pub deaths: u16,            // Individual deaths (for scoreboard)

    // NEW for TDM spectate
    pub spectate_target: Option<PlayerId>,  // Who to spectate when dead
}
```

**Player Lifecycle State Machine**:

```text
              ┌──────────────┐
              │    Alive     │
              │  is_dead=F   │
              │  spectate=N  │
              └──────┬───────┘
                     │ killed by enemy
                     ▼
              ┌──────────────┐
              │    Dead      │
              │  is_dead=T   │
              │  spectate=K  │ ◄─── K = killer ID
              │  respawn=T+D │ ◄─── T = death tick, D = delay
              └──────┬───────┘
                     │ current_tick >= respawn_tick
                     ▼
              ┌──────────────┐
              │    Alive     │
              │  is_dead=F   │
              │  spectate=N  │ ◄─── cleared on respawn
              └──────────────┘
```

**Validation**:
- spectate_target is only set when is_dead = true
- spectate_target must be a valid player ID (or None if killer disconnected)
- respawn_tick > current_tick when dead

---

### PlayerScore (Existing - `plix_common::protocol::PlayerScore`)

Per-player stats for scoreboard display.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    pub player_id: PlayerId,
    pub name: String,
    pub kills: u16,
    pub deaths: u16,
}
```

**Note**: In TDM, individual kills still tracked for scoreboard, but team score determines winner.

---

### KillEvent (Implicit - processed in Server)

Not a stored entity, but important for data flow.

```rust
// Conceptual - not stored
struct KillEvent {
    killer_id: PlayerId,
    killer_team: TeamId,
    victim_id: PlayerId,
    victim_team: TeamId,
    tick: Tick,
}
```

**Validation** (before awarding team point):
- killer_id != victim_id (no self-kill points)
- killer_team != victim_team (no friendly fire points)
- match_state.phase == Playing (no points outside match)

---

## Relationships Diagram

```text
┌──────────────┐         ┌──────────────┐
│ MatchConfig  │────────►│MatchState    │
│              │         │Machine       │
│ score_limit  │         │              │
│ respawn_delay│         │ - state      │
│ reset_delay  │         │ - config     │
│ team_size    │         │              │
└──────────────┘         └───────┬──────┘
                                 │
                                 │ contains
                                 ▼
                         ┌──────────────┐
                         │ MatchState   │
                         │              │
                         │ - phase      │
                         │ - scores[]   │◄──────┐
                         │ - team_winner│       │
                         │ - player_    │       │
                         │   scores[]   │       │
                         └──────────────┘       │
                                                │
┌──────────────┐         ┌──────────────┐       │
│ PlayerSession│────────►│   TeamId     │───────┘
│              │  team   │              │ TeamScore.team
│ - spectate_  │         │ TEAM_0 (Red) │
│   target     │         │ TEAM_1 (Blue)│
│ - is_dead    │         └──────────────┘
│ - respawn_   │
│   tick       │
│ - kills      │─────────► PlayerScore
│ - deaths     │
└──────────────┘
```

## State Transitions

### Match Phase Transitions

```text
                    min_players ready
        ┌─────────────────────────────────────────┐
        │                                         │
        ▼              countdown_complete         │
    ┌───────┐      ┌───────────┐      ┌───────┐  │
    │ Lobby │─────►│ Countdown │─────►│Playing│  │
    └───┬───┘      └─────┬─────┘      └───┬───┘  │
        │                │                │       │
        │    cancel      │                │       │
        │◄───────────────┘                │       │
        │                                 │       │
        │         score_limit OR time_out │       │
        │                                 ▼       │
        │                          ┌───────────┐  │
        │                          │ EndScreen │  │
        │                          └─────┬─────┘  │
        │                                │        │
        │                 reset_delay    │        │
        │                                ▼        │
        │                          ┌───────────┐  │
        └──────────────────────────│ Resetting │──┘
                                   └───────────┘
```

### Team Score State

```text
State: scores[team].score

Lobby/Countdown:  [0, 0]
                    │
                    │ match starts
                    ▼
Playing:          [n, m] where n,m incrementing
                    │
                    │ score_limit reached
                    ▼
EndScreen:        [n, m] frozen, team_winner set
                    │
                    │ reset_delay
                    ▼
Resetting→Lobby:  [0, 0] reset
```

### Player Death/Respawn State

```text
State: (is_dead, spectate_target, respawn_tick)

Alive:     (false, None, None)
              │
              │ killed by player K
              ▼
Dead:      (true, Some(K), Some(T+D))
              │
              │ T+D ticks elapsed
              ▼
Respawned: (false, None, None)
```

## Validation Rules Summary

| Entity | Rule | Error if Violated |
|--------|------|-------------------|
| TeamScore | score >= 0 | Panic (internal) |
| TeamScore | increment only during Playing | Ignored |
| MatchConfig | score_limit >= 1 | Use default |
| MatchConfig | team_size >= 1 | Use default |
| PlayerSession | spectate_target valid player | Set to None |
| KillEvent | killer_team != victim_team for points | No team point |
| KillEvent | phase == Playing for points | No team point |
