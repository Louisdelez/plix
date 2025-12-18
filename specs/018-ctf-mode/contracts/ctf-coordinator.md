# Contract: CTF Coordinator

**Module**: `plix-server/src/ctf/coordinator.rs`
**Date**: 2025-12-16

## Purpose

Orchestrates CTF game flow by handling events, coordinating between rules engine and state, and triggering broadcasts. Acts as the integration point between the game loop and CTF subsystem.

## Interface

### CtfCoordinator

```rust
impl CtfCoordinator {
    /// Create coordinator with initial state
    pub fn new(state: CtfState) -> Self;

    /// Get immutable state reference
    pub fn state(&self) -> &CtfState;

    /// Get CTF config
    pub fn config(&self) -> &CtfConfig;

    /// Process player position update - checks for pickups/captures
    /// Returns events that occurred
    pub fn on_player_position(
        &mut self,
        player_id: PlayerId,
        player_team: TeamId,
        position: Vec3,
        current_tick: Tick,
    ) -> Vec<CtfEvent>;

    /// Process player death - drops flag if carrying
    /// Returns events that occurred
    pub fn on_player_death(
        &mut self,
        player_id: PlayerId,
        death_position: Vec3,
        current_tick: Tick,
    ) -> Vec<CtfEvent>;

    /// Process player disconnect - drops flag if carrying
    /// Returns events that occurred
    pub fn on_player_disconnect(
        &mut self,
        player_id: PlayerId,
        last_position: Vec3,
        current_tick: Tick,
    ) -> Vec<CtfEvent>;

    /// Tick update - processes timers
    /// Returns events that occurred (auto-returns)
    pub fn tick(&mut self, current_tick: Tick) -> Vec<CtfEvent>;

    /// Reset state for new match
    pub fn reset(&mut self);

    /// Check if capture limit reached
    pub fn is_victory(&self) -> Option<TeamId>;

    /// Get current capture scores
    pub fn scores(&self) -> [u32; 2];

    /// Get current flag states for broadcast
    pub fn flag_states(&self) -> [FlagState; 2];
}

/// Events emitted by coordinator
#[derive(Debug, Clone)]
pub enum CtfEvent {
    /// Player picked up enemy flag
    FlagPickup {
        player_id: PlayerId,
        flag_team: TeamId,
    },
    /// Player dropped flag (death/disconnect)
    FlagDrop {
        player_id: PlayerId,
        flag_team: TeamId,
        position: Vec3,
        return_tick: Tick,
    },
    /// Flag returned to base (teammate touch or timeout)
    FlagReturn {
        flag_team: TeamId,
        reason: ReturnReason,
    },
    /// Flag captured - point scored
    FlagCapture {
        capturing_team: TeamId,
        capturing_player: PlayerId,
        new_score: u32,
    },
    /// Team won by reaching capture limit
    Victory {
        winning_team: TeamId,
        final_scores: [u32; 2],
    },
}

#[derive(Debug, Clone, Copy)]
pub enum ReturnReason {
    TeammateTouch,
    Timeout,
    OutOfBounds,
}
```

## Behavior Contracts

### BC-201: Player Position Processing

**Called**: Every tick for each player during Playing phase

**Flow**:
1. Check if player can pick up enemy flag → emit `FlagPickup`
2. Check if player can return own dropped flag → emit `FlagReturn`
3. Check if carrier can capture → emit `FlagCapture`
4. If capture reached limit → emit `Victory`

**Postconditions**:
- Events returned in order they occurred
- State updated before events returned

### BC-202: Player Death Processing

**Called**: When player dies during Playing phase

**Flow**:
1. If player carrying flag → drop flag → emit `FlagDrop`
2. Set flag state to `Dropped` with return timer

**Postconditions**:
- If carrying: exactly one `FlagDrop` event
- If not carrying: empty events

### BC-203: Player Disconnect Processing

**Called**: When player disconnects during Playing phase

**Behavior**: Same as death processing

### BC-204: Tick Processing

**Called**: Every tick during Playing phase

**Flow**:
1. Check dropped flag timers
2. For each expired timer → return flag → emit `FlagReturn(Timeout)`

**Postconditions**:
- Events for each flag that timed out
- No events if no timers expired

### BC-205: Victory Detection

**Called**: After each capture

**Postconditions**:
- Returns `Some(team)` if `team.score >= config.capture_limit`
- Returns `None` if neither team at limit

### BC-206: Reset

**Called**: On match transition to Resetting phase

**Postconditions**:
- State reset (flags at base, scores zero)
- No events returned

## Event Ordering

Events must be processed and broadcast in this order:
1. `FlagDrop` (death triggers drop before other interactions)
2. `FlagReturn` (can happen after drop)
3. `FlagPickup` (after returns processed)
4. `FlagCapture` (requires pickup first)
5. `Victory` (after capture processed)

## Integration Points

### Game Loop Integration

```rust
// In server game loop (pseudo-code)
fn tick(&mut self, current_tick: Tick) {
    // ... input processing, movement, combat ...

    // After combat resolution (deaths processed)
    for (player_id, death_event) in deaths {
        let events = self.ctf.on_player_death(player_id, death_pos, current_tick);
        self.broadcast_ctf_events(events);
    }

    // Position-based interactions
    for player in players {
        let events = self.ctf.on_player_position(
            player.id, player.team, player.position, current_tick
        );
        self.broadcast_ctf_events(events);
    }

    // Timer updates
    let events = self.ctf.tick(current_tick);
    self.broadcast_ctf_events(events);

    // Check victory
    if let Some(winner) = self.ctf.is_victory() {
        self.match_state.end_match_ctf_victory(winner, current_tick);
    }
}
```

### Network Broadcast

Each `CtfEvent` maps to network messages:
- `FlagPickup` → `CtfFlagUpdate` (state = Carried)
- `FlagDrop` → `CtfFlagUpdate` (state = Dropped)
- `FlagReturn` → `CtfFlagUpdate` (state = AtBase)
- `FlagCapture` → `CtfCaptureEvent` + `CtfFlagUpdate` (both AtBase)
- `Victory` → handled by match state transition

## Test Scenarios

### T-COORD-001: Position Update Triggers Pickup
```
Given: player in enemy flag zone, flag AtBase
When: coordinator.on_player_position(player, pos, tick)
Then: returns [FlagPickup { player, flag_team }]
And: state.flag(flag_team).state == Carried(player)
```

### T-COORD-002: Death Triggers Drop
```
Given: player carrying flag
When: coordinator.on_player_death(player, pos, tick)
Then: returns [FlagDrop { player, team, pos, return_tick }]
```

### T-COORD-003: Capture Sequence
```
Given: player carrying enemy flag, in capture zone, own flag at base
When: coordinator.on_player_position(player, capture_zone_pos, tick)
Then: returns [FlagCapture { team, player, score }]
And: both flags AtBase
And: team score incremented
```

### T-COORD-004: Victory After Final Capture
```
Given: team score = capture_limit - 1, player about to capture
When: coordinator.on_player_position triggers capture
Then: returns [FlagCapture, Victory { team }]
```

### T-COORD-005: Timer Expiry Returns Flag
```
Given: flag Dropped with return_tick = 100, current_tick = 100
When: coordinator.tick(100)
Then: returns [FlagReturn { team, Timeout }]
And: flag AtBase
```

### T-COORD-006: Multiple Events Single Tick
```
Given: two dropped flags both timing out
When: coordinator.tick(expiry_tick)
Then: returns [FlagReturn(team0), FlagReturn(team1)]
```
