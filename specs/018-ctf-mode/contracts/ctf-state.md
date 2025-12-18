# Contract: CTF State Management

**Module**: `plix-server/src/ctf/state.rs`
**Date**: 2025-12-16

## Purpose

Manages the complete state of a CTF match including flags, zones, and capture scores.

## Interface

### CtfState

```rust
impl CtfState {
    /// Create new CTF state from arena zones and config
    pub fn new(zones: Vec<FlagZone>, config: CtfConfig) -> Self;

    /// Get flag reference for a team (team.0 must be 0 or 1)
    pub fn flag(&self, team: TeamId) -> &Flag;

    /// Get mutable flag reference for a team
    pub fn flag_mut(&mut self, team: TeamId) -> &mut Flag;

    /// Get capture score for a team
    pub fn score(&self, team: TeamId) -> u32;

    /// Get flag base zone for a team
    pub fn flag_base(&self, team: TeamId) -> Option<&FlagZone>;

    /// Get capture zone for a team
    pub fn capture_zone(&self, team: TeamId) -> Option<&FlagZone>;

    /// Reset state for new match (flags to base, scores to 0)
    pub fn reset(&mut self);
}
```

## Behavior Contracts

### BC-001: New State Initialization

**Preconditions**:
- `zones` contains at least one FlagBase zone per team
- `config` has valid values (capture_limit > 0, delays > 0)

**Postconditions**:
- `flags[0].state == FlagState::AtBase`
- `flags[1].state == FlagState::AtBase`
- `flags[0].base_position` equals center of team 0 flag base zone
- `flags[1].base_position` equals center of team 1 flag base zone
- `capture_scores == [0, 0]`

### BC-002: Flag Access

**Preconditions**:
- `team.0` is 0 or 1

**Postconditions**:
- Returns reference to flag at index `team.0`
- Does not modify state

### BC-003: Reset

**Preconditions**:
- State exists

**Postconditions**:
- `flags[0].state == FlagState::AtBase`
- `flags[1].state == FlagState::AtBase`
- `capture_scores == [0, 0]`
- `zones` unchanged
- `config` unchanged
- `flags[*].base_position` unchanged

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Missing flag base for team | Use Vec3::ZERO as base_position |
| Invalid team index | Panic (internal invariant violation) |

## Test Scenarios

### T-STATE-001: Fresh State Has Flags At Base
```
Given: zones with flag bases for both teams
When: CtfState::new(zones, config)
Then: both flags are AtBase
```

### T-STATE-002: Reset Returns Flags To Base
```
Given: state with flags Carried/Dropped, scores > 0
When: state.reset()
Then: both flags AtBase, scores [0, 0]
```

### T-STATE-003: Zone Lookup Returns Correct Zone
```
Given: state with zones for both teams
When: state.flag_base(TeamId::TEAM_0)
Then: returns Some(zone) with team==0, zone_type==FlagBase
```
