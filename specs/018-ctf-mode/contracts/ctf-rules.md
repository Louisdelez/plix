# Contract: CTF Rules Engine

**Module**: `plix-server/src/ctf/rules.rs`
**Date**: 2025-12-16

## Purpose

Implements the game rules for CTF flag interactions: pickup, drop, return, and capture logic.

## Interface

### CtfRules

```rust
impl CtfRules {
    /// Check if player can pick up a flag
    /// Returns Some(team_id) of flag that can be picked up, None otherwise
    pub fn can_pickup(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &CtfState,
    ) -> Option<TeamId>;

    /// Execute flag pickup - mutates state
    /// Returns true if pickup succeeded
    pub fn pickup(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &mut CtfState,
    ) -> bool;

    /// Drop flag at position (called on death/disconnect)
    /// Returns true if player was carrying a flag
    pub fn drop(
        player_id: PlayerId,
        drop_pos: Vec3,
        current_tick: Tick,
        state: &mut CtfState,
    ) -> bool;

    /// Check if teammate can return a dropped flag
    pub fn can_return(
        player_team: TeamId,
        player_pos: Vec3,
        state: &CtfState,
    ) -> bool;

    /// Return dropped flag to base (teammate touch)
    pub fn return_flag(
        player_team: TeamId,
        state: &mut CtfState,
    ) -> bool;

    /// Check if carrier can capture (in capture zone, own flag at base)
    pub fn can_capture(
        player_id: PlayerId,
        player_team: TeamId,
        player_pos: Vec3,
        state: &CtfState,
    ) -> bool;

    /// Execute capture - awards point, resets flags
    /// Returns new capture score for team
    pub fn capture(
        player_id: PlayerId,
        player_team: TeamId,
        state: &mut CtfState,
    ) -> u32;

    /// Update dropped flag timers - returns flags that auto-returned
    pub fn update_return_timers(
        current_tick: Tick,
        state: &mut CtfState,
    ) -> Vec<TeamId>;

    /// Check if flag is out of bounds and should auto-return
    pub fn check_out_of_bounds(
        flag_team: TeamId,
        arena_bounds: &ArenaBounds,
        state: &mut CtfState,
    ) -> bool;
}
```

## Behavior Contracts

### BC-101: Flag Pickup

**Preconditions**:
- Player is on opposing team from flag
- Flag state is `AtBase` or `Dropped`
- Player is inside flag's zone (if AtBase) or near flag position (if Dropped)
- Player is not already carrying a flag

**Postconditions**:
- Flag state becomes `Carried { carrier: player_id }`
- Returns `true`

**Rejection cases**:
- Same team as flag → returns `false`
- Flag is being carried → returns `false`
- Player not in pickup range → returns `false`
- Player already carrying flag → returns `false`

### BC-102: Flag Drop

**Preconditions**:
- Player exists
- Current tick is valid

**Postconditions**:
- If player was carrying a flag:
  - Flag state becomes `Dropped { position: drop_pos, return_tick }`
  - `return_tick = current_tick + config.flag_return_delay_ticks`
  - Returns `true`
- If player was not carrying:
  - State unchanged
  - Returns `false`

### BC-103: Flag Return (Teammate)

**Preconditions**:
- Flag belongs to player's team
- Flag state is `Dropped`
- Player is touching dropped flag position

**Postconditions**:
- Flag state becomes `AtBase`
- Returns `true`

### BC-104: Flag Capture

**Preconditions**:
- Player is carrying enemy flag
- Player is inside own team's capture zone
- Own team's flag is `AtBase` (classic rule)

**Postconditions**:
- Own team's capture score increments by 1
- **Both** flags reset to `AtBase`
- Returns new capture score

**Rejection cases**:
- Not carrying flag → returns 0, state unchanged
- Not in capture zone → returns 0, state unchanged
- Own flag not at base → returns 0, state unchanged (classic rule)

### BC-105: Auto-Return Timer

**Preconditions**:
- Called each tick during Playing phase

**Postconditions**:
- For each flag with `Dropped { return_tick }`:
  - If `current_tick >= return_tick`: flag state becomes `AtBase`
- Returns list of flag teams that auto-returned

### BC-106: Out of Bounds Return

**Preconditions**:
- Flag state is `Dropped`
- Flag position outside arena bounds

**Postconditions**:
- Flag state becomes `AtBase`
- Returns `true`

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Invalid team ID | Panic (internal invariant) |
| Missing capture zone | Return false, log warning |
| Missing flag base | Return false, log warning |

## Test Scenarios

### T-RULES-001: Enemy Player Can Pickup Flag At Base
```
Given: flag AtBase, player on opposing team in flag zone
When: CtfRules::pickup(player, state)
Then: flag state = Carried(player), returns true
```

### T-RULES-002: Same Team Cannot Pickup Own Flag
```
Given: flag AtBase, player on same team
When: CtfRules::can_pickup(player, state)
Then: returns None
```

### T-RULES-003: Carrier Already Carrying Cannot Pickup Another
```
Given: player carrying team0 flag, team1 flag AtBase
When: CtfRules::can_pickup(player, team1_zone, state)
Then: returns None
```

### T-RULES-004: Death Drops Flag With Timer
```
Given: player carrying flag, current_tick = 1000
When: CtfRules::drop(player, pos, tick, state)
Then: flag = Dropped(pos, 1600) [assuming 600 tick delay]
```

### T-RULES-005: Teammate Touch Returns Dropped Flag
```
Given: flag Dropped, teammate touching flag
When: CtfRules::return_flag(teammate_team, state)
Then: flag = AtBase, returns true
```

### T-RULES-006: Capture With Own Flag At Base Succeeds
```
Given: player carrying enemy flag, in capture zone, own flag AtBase
When: CtfRules::capture(player, state)
Then: score incremented, both flags AtBase
```

### T-RULES-007: Capture Blocked When Own Flag Not At Base
```
Given: player carrying enemy flag, in capture zone, own flag NOT AtBase
When: CtfRules::can_capture(player, state)
Then: returns false
```

### T-RULES-008: Timer Expiry Returns Flag
```
Given: flag Dropped with return_tick = 1000, current_tick = 1000
When: CtfRules::update_return_timers(tick, state)
Then: flag = AtBase, returns [flag_team]
```

### T-RULES-009: Out Of Bounds Auto-Returns
```
Given: flag Dropped outside arena bounds
When: CtfRules::check_out_of_bounds(team, bounds, state)
Then: flag = AtBase, returns true
```
