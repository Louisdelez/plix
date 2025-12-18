# Research: CTF Mode (Capture The Flag)

**Feature**: 018-ctf-mode | **Date**: 2025-12-16

## Overview

This document captures technical research and decisions for implementing CTF mode in the Plix game platform. The feature extends the existing TDM/FFA game mode infrastructure.

## Existing Infrastructure Analysis

### GameMode Enum (plix-common/src/types.rs)

```rust
pub enum GameMode {
    #[default]
    Tdm,  // Team Deathmatch
    Ffa,  // Free-for-All
    // CTF will be added here
}
```

**Decision**: Add `Ctf` variant to existing enum. No breaking changes required.

### MatchStateMachine (plix-server/src/match_state.rs)

Existing patterns to reuse:
- `MatchConfig` with mode-specific defaults (`tdm_default()`, `ffa_default()`)
- `MatchPhase` state machine (Lobby → Countdown → Playing → EndScreen → Resetting)
- Team scoring via `award_team_kill()` and `check_team_score_limit()`
- Phase transitions with tick-based timing

**Decision**: Add `ctf_default()` to MatchConfig. Extend MatchState with CTF-specific fields (flag states, capture scores).

### Arena Format (plix-arena/src/format.rs)

Current structure:
- `ArenaMetadata` with `game_mode: GameMode`
- `SpawnPoint` with team assignment
- `BlockDefinitions` with regions (AABB boxes)

**Decision**: Add CTF zone definitions using same AABB pattern as regions. New fields in arena TOML:
- `[[ctf.flag_bases]]` - Flag spawn locations per team
- `[[ctf.capture_zones]]` - Capture zone AABBs per team

## Technical Decisions

### Flag State Machine

```
┌──────────┐     pickup      ┌──────────┐     enter zone     ┌──────────┐
│  AtBase  │ ───────────────>│ Carried  │ ──────────────────>│ Captured │
└──────────┘                 └──────────┘                    └──────────┘
     ^                            │                               │
     │                            │ death/disconnect              │
     │                            v                               │
     │       timeout        ┌──────────┐                          │
     └──────────────────────│ Dropped  │                          │
     │                      └──────────┘                          │
     │                            │                               │
     │       teammate touch       │                               │
     └────────────────────────────┘                               │
     │                                                            │
     └────────────────────────────────────────────────────────────┘
                              reset on capture
```

**Decision**: `FlagState` enum with variants:
- `AtBase` - Flag at home position
- `Carried(PlayerId)` - Flag being carried
- `Dropped { position: Vec3, return_tick: Tick }` - Flag on ground with timer

### Zone Collision Detection

Player-zone collision check needed each tick for:
- Flag pickup (player enters enemy flag_base when flag is AtBase)
- Flag capture (carrier enters own capture_zone)

**Decision**: AABB-point collision check. Player position is single point, zones are axis-aligned boxes.

```rust
fn point_in_aabb(point: Vec3, min: Vec3, max: Vec3) -> bool {
    point.x >= min.x && point.x <= max.x &&
    point.y >= min.y && point.y <= max.y &&
    point.z >= min.z && point.z <= max.z
}
```

### Classic Capture Rule

Per spec: "Own flag must be at base to capture enemy flag."

**Decision**: Check `own_flag.state == AtBase` before allowing capture. If not at base, carrier continues holding flag.

### Event-Driven Architecture

Per constitution requirement (II. Performance), avoid polling. CTF events:

| Event | Trigger | Actions |
|-------|---------|---------|
| FlagPickup | Player enters enemy flag zone | Update flag state, broadcast |
| FlagDrop | Carrier dies/disconnects | Drop at position, start timer |
| FlagReturn | Teammate touches dropped flag OR timer expires | Reset to base |
| FlagCapture | Carrier enters capture zone (own flag at base) | Award point, reset flags |

### Network Protocol

New message types for plix-common/src/protocol/messages.rs:

```rust
// Server -> Client
pub struct FlagStateUpdate {
    pub team: TeamId,
    pub state: FlagState,
}

pub struct CtfCaptureEvent {
    pub capturing_team: TeamId,
    pub capturing_player: PlayerId,
    pub team_scores: [u32; 2],
}
```

### Configuration Defaults

| Parameter | Default | Reasoning |
|-----------|---------|-----------|
| capture_limit | 3 | Standard CTF, quick matches |
| flag_return_delay | 10s (600 ticks) | Balance between recovery and defense |
| respawn_delay | 5s (300 ticks) | Longer than TDM for CTF pacing |
| time_limit | 600s (10 min) | Allow multiple captures |
| end_screen_delay | 10s | Match TDM pattern |

## Integration Points

### Server Game Loop

In `plix-server/src/lib.rs`, the main game loop processes:
1. Player input handling
2. Movement/physics simulation
3. Combat resolution
4. **CTF flag interactions** (new)
5. Match state updates
6. State broadcast

CTF processing fits after combat (death triggers flag drop) and before match state (captures affect score).

### Death Handling

Current flow: Player dies → respawn timer starts → player respawns

CTF extension: Player dies → **if carrying flag, drop it** → respawn timer starts

### Disconnect Handling

Current flow: Player disconnects → remove from game state

CTF extension: Player disconnects → **if carrying flag, drop it** → remove from game state

## Risk Assessment

| Risk | Likelihood | Mitigation |
|------|------------|------------|
| Zone collision performance | Low | AABB check is O(1), only check when in playing phase |
| Flag state desync | Medium | Server-authoritative, broadcast on every state change |
| Race condition on capture | Low | Single-threaded tick processing, deterministic order |
| Arena without CTF zones | Medium | Validation at arena load time, reject invalid CTF arenas |

## Open Questions (Resolved)

All questions from spec clarification were resolved:
- Two teams only (no multi-team CTF) ✅
- Classic capture rule always enabled ✅
- No flag physics (instant pickup/drop) ✅
- Client rendering out of scope ✅

## References

- Existing implementation: `crates/plix-server/src/match_state.rs` (TDM/FFA patterns)
- Arena format: `crates/plix-arena/src/format.rs`
- Protocol messages: `crates/plix-common/src/protocol/messages.rs`
- Example arenas: `assets/arenas/test_arena.toml`, `assets/arenas/ffa_arena.toml`
