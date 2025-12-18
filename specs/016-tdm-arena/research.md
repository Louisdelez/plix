# Research: TDM Arena Mode

**Feature**: 016-tdm-arena
**Date**: 2025-12-16
**Purpose**: Resolve unknowns and document technical decisions before implementation

## Research Questions

### R1: How to integrate team scoring with existing MatchStateMachine?

**Context**: The existing `MatchStateMachine` tracks individual player kills via `player_scores: Vec<PlayerScore>` and uses `score_limit` as individual kill count. TDM needs team-based scoring.

**Research Findings**:

The existing codebase already has infrastructure for team scoring:
- `TeamScore` struct in `protocol/messages.rs` with `team: TeamId` and `score: u32`
- `MatchState.scores: Vec<TeamScore>` initialized with two teams (TEAM_0, TEAM_1)
- Team scores are already broadcast in `WorldSnapshot.match_state`

Current gap: Team scores are initialized to 0 but never incremented. Individual `player_scores` drives win condition.

**Decision**: Add `award_team_kill(team: TeamId)` method to `MatchStateMachine` that:
1. Increments `state.scores[team_index].score += 1`
2. Calls `check_team_score_limit()` to potentially end match

**Rationale**: Minimal change to existing structure. Team scores already exist but unused.

**Alternatives Considered**:
- Create separate `TdmMatchStateMachine`: Rejected (code duplication, existing state machine is flexible)
- Replace individual scores with team scores: Rejected (both useful - team for win condition, individual for scoreboard)

---

### R2: How to implement spectate-killer during death?

**Context**: Clarification Q1 specified players spectate their killer during respawn delay.

**Research Findings**:

Current death handling in `lib.rs`:
```rust
if killed {
    let event = GameEvent::PlayerDied { victim: target_id, killer: Some(attacker_id) };
    self.broadcast_event(event).await;
    // ... score updates
}
```

Current player state in `session.rs`:
- `is_dead: bool`
- `respawn_tick: Option<Tick>`
- No spectate target field

Client side:
- Uses `PlayerSnapshot` for camera target
- No spectate mode currently implemented

**Decision**:
1. Add `spectate_target: Option<PlayerId>` to player session
2. Set `spectate_target = killer_id` on death
3. Include `spectate_target` in `PlayerSnapshot` for client
4. Client camera follows `spectate_target` position/rotation when dead
5. Clear `spectate_target` on respawn

**Rationale**: Simple addition; client can interpolate killer's position from existing snapshot data.

**Alternatives Considered**:
- Server-side camera state: Rejected (unnecessary complexity; client has all data from snapshots)
- Spectate random teammate: Rejected (clarification Q1 specified killer)

---

### R3: How to handle team-based win condition?

**Context**: TDM ends when a team reaches `score_limit`, not an individual player.

**Research Findings**:

Current win condition check in `match_state.rs`:
```rust
pub fn check_score_limit(&mut self, player_id: PlayerId, kills: u16, current_tick: Tick) -> bool {
    if kills >= self.config.score_limit {
        self.end_match_score_limit(player_id, current_tick);
        true
    }
}
```

This checks individual player's kill count.

**Decision**: Add `check_team_score_limit(&mut self, team: TeamId, current_tick: Tick) -> bool`:
```rust
pub fn check_team_score_limit(&mut self, team: TeamId, current_tick: Tick) -> bool {
    let team_score = self.state.scores.iter()
        .find(|s| s.team == team)
        .map(|s| s.score)
        .unwrap_or(0);

    if team_score >= self.config.score_limit as u32 {
        self.end_match_team_score_limit(team, current_tick);
        true
    } else {
        false
    }
}
```

**Rationale**: Parallel to existing individual check; clean separation.

**Alternatives Considered**:
- Modify existing `check_score_limit` to handle both: Rejected (makes method complex, less clear intent)
- Remove individual scoring entirely: Rejected (per-player stats still useful for scoreboard)

---

### R4: How to broadcast team score updates?

**Context**: Clients need to know when team scores change.

**Research Findings**:

Existing events in `GameEvent`:
- `ScoreUpdate { player_id, kills, deaths }` - per-player scores
- No team-specific score update event

Options:
1. Add new `TeamScoreUpdate { team: TeamId, new_score: u32 }` event
2. Reuse `MatchState.scores` in regular snapshots (already broadcast every tick)
3. Both

**Decision**: Option 2 (rely on snapshot) with optional event for immediate feedback.

Since `WorldSnapshot.match_state.scores` already contains team scores and is sent every tick (16ms), clients already receive updated team scores without a separate event.

For immediate kill feedback, the existing `PlayerDied` event is sufficient. Client can infer team score change from killer's team.

**Rationale**: No new event type needed. Existing infrastructure sufficient.

**Alternatives Considered**:
- Add `TeamScoreUpdate` event: Not rejected, but deferred. Can add later if snapshot latency is problematic.

---

### R5: How to handle simultaneous kills in same tick?

**Context**: Edge case where both players kill each other in the same tick.

**Research Findings**:

Current combat processing in `simulate_tick()`:
```rust
for (attacker_id, attacker_pos, attacker_forward, last_attack_tick) in attack_requests {
    if let Some((target_id, hit_result)) = self.combat.try_attack(...) {
        // Process kill sequentially
    }
}
```

Kills are processed sequentially within a single tick. If Player A kills Player B, then Player B kills Player A, both kills register but in order.

**Decision**: Accept sequential processing. First kill that reaches score limit wins. If both teams reach limit in same tick, first processed team wins.

**Rationale**:
- True simultaneity is impossible in sequential processing
- 16ms tick granularity makes perception of "same time" reasonable
- No gameplay issue since kills are extremely unlikely to be exactly simultaneous

**Alternatives Considered**:
- Detect simultaneous and award to neither: Rejected (penalizes both players unfairly)
- Track all kills first, then award: Over-engineering for rare edge case

---

### R6: How to validate TDM spawn points in arena?

**Context**: TDM requires spawn points for both teams.

**Research Findings**:

Current arena spawn point structure (from plix-arena):
```rust
pub struct SpawnPoint {
    pub position: [f32; 3],
    pub rotation: f32,
    pub team: Option<TeamId>,  // Already exists!
}
```

`SpawnManager` already filters by team:
```rust
pub fn get_spawn_point(&self, team: TeamId) -> Option<&SpawnPoint> {
    // Returns spawn point matching team
}
```

**Decision**: No code changes needed for arena integration. Existing spawn system already supports team-filtered spawns.

**Validation**: On server startup, log warning if either team has zero spawn points configured.

**Rationale**: Infrastructure already exists and works.

---

## Technical Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Team scoring storage | Use existing `MatchState.scores` | Already present but unused |
| Win condition | New `check_team_score_limit()` | Parallel to individual check |
| Score broadcast | Via existing snapshots | No new event type needed |
| Spectate target | New field in player session | Simple addition |
| Spectate camera | Client-side from snapshot | Server already broadcasts positions |
| Simultaneous kills | Sequential processing | First processed wins |
| Arena spawns | Existing team field | No changes needed |

## No Remaining Unknowns

All NEEDS CLARIFICATION items resolved. Ready for Phase 1 design.
