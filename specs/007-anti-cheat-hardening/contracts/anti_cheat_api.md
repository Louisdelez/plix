# Anti-Cheat Module API Contract

**Feature**: 007-anti-cheat-hardening
**Date**: 2025-12-15
**Type**: Internal Rust Module API

## Overview

This document defines the public API contract for the `anti_cheat` module in `plix-server`. Since this is a server-side feature with no external HTTP/RPC interface, these contracts define the Rust trait and struct interfaces that other modules depend on.

---

## Module: `anti_cheat::config`

### `AntiCheatConfig`

```rust
/// Anti-cheat configuration with safe defaults.
/// All fields are public for easy construction in tests.
#[derive(Debug, Clone)]
pub struct AntiCheatConfig {
    /// Maximum movement inputs per second (default: 120)
    pub max_inputs_per_second: u32,
    /// Maximum attacks per second (default: 4)
    pub max_attacks_per_second: u32,
    /// Maximum block edits per second (default: 10)
    pub max_block_edits_per_second: u32,
    /// Maximum ready toggles per second (default: 5)
    pub max_ready_toggles_per_second: u32,
    /// Maximum movement distance per tick in blocks (default: 0.25)
    pub max_speed_per_tick: f32,
    /// Maximum acceleration in blocks/tick^2 (default: 1.5)
    pub max_acceleration: f32,
    /// Strike count to trigger warning (default: 3)
    pub warning_threshold: u32,
    /// Strike count to trigger kick (default: 5)
    pub kick_threshold: u32,
    /// Strike count to trigger temp ban (default: 10)
    pub ban_threshold: u32,
    /// Ban duration in seconds (default: 3600)
    pub ban_duration_seconds: u64,
}

impl Default for AntiCheatConfig {
    fn default() -> Self;
}
```

**Invariants**:
- `warning_threshold < kick_threshold < ban_threshold`
- All rate limits > 0

---

## Module: `anti_cheat::state`

### `AntiCheatState`

```rust
/// Per-player anti-cheat tracking state.
/// Zero-cost default construction for embedding in ServerPlayer.
#[derive(Debug, Clone, Default)]
pub struct AntiCheatState {
    // Fields are private; access via methods
}

impl AntiCheatState {
    /// Create new state for a player connection
    pub fn new(current_tick: Tick) -> Self;

    /// Record an infraction, returns updated strike count
    pub fn record_infraction(&mut self, infraction: InfractionType) -> u32;

    /// Check and update rate limit for an action, returns true if allowed
    pub fn check_rate_limit(
        &mut self,
        action: ActionType,
        current_tick: Tick,
        config: &AntiCheatConfig,
    ) -> bool;

    /// Get current strike count
    pub fn strikes(&self) -> u32;

    /// Reset rate limit window (called each tick or second boundary)
    pub fn maybe_reset_window(&mut self, current_tick: Tick);

    /// Check if input sequence is valid (newer than last seen)
    pub fn check_sequence(&mut self, seq: InputSeq) -> bool;
}
```

**Guarantees**:
- `record_infraction` always increases strike count by 1
- `check_rate_limit` never allocates
- All operations are O(1)

---

## Module: `anti_cheat::ban_list`

### `BanList`

```rust
/// In-memory ban list with IP-based identification.
#[derive(Debug, Default)]
pub struct BanList {
    // Private HashMap<IpAddr, BanEntry>
}

impl BanList {
    /// Create empty ban list
    pub fn new() -> Self;

    /// Check if IP is currently banned, returns ban info if so
    pub fn is_banned(&self, ip: &IpAddr) -> Option<&BanEntry>;

    /// Add or update a ban for an IP
    pub fn add_ban(&mut self, ip: IpAddr, reason: String, duration: Duration);

    /// Remove a specific ban (admin action)
    pub fn unban(&mut self, ip: &IpAddr) -> bool;

    /// Remove all expired bans, returns count removed
    pub fn cleanup_expired(&mut self) -> usize;

    /// Get number of active bans
    pub fn len(&self) -> usize;
}
```

**Guarantees**:
- `is_banned` returns None for expired bans (lazy cleanup)
- `add_ban` with existing IP overwrites (extends ban)
- Thread-safe read access (no interior mutability required for single-threaded server)

---

## Module: `anti_cheat::sanctions`

### `SanctionManager`

```rust
/// Manages sanction escalation logic.
pub struct SanctionManager {
    config: AntiCheatConfig,
}

impl SanctionManager {
    /// Create manager with config
    pub fn new(config: AntiCheatConfig) -> Self;

    /// Determine appropriate sanction based on strike count
    pub fn evaluate(&self, strikes: u32) -> Option<SanctionType>;

    /// Check if warning should be sent (rate-limited)
    pub fn should_warn(&self, state: &AntiCheatState, current_tick: Tick) -> bool;
}
```

---

## Module: `anti_cheat::validation`

### Input Validation Functions

```rust
/// Validate a PlayerInput for NaN/INF and bounds.
/// Returns Ok(()) if valid, Err(InfractionType) if invalid.
pub fn validate_input(input: &PlayerInput) -> Result<(), InfractionType>;

/// Validate a BlockEditRequest for bounds.
/// Returns Ok(()) if valid, Err(InfractionType) if invalid.
pub fn validate_block_edit(request: &BlockEditRequest, arena_bounds: &ArenaBounds) -> Result<(), InfractionType>;

/// Check if all float fields are finite (not NaN or INF).
pub fn floats_are_finite(input: &PlayerInput) -> bool;
```

**Guarantees**:
- `validate_input` never panics regardless of input values
- `floats_are_finite` is a single branch per field (~5ns total)

---

## Enums

### `InfractionType`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InfractionType {
    InvalidFloat,
    OutOfBounds,
    InvalidSequence,
    InputRateExceeded,
    AttackRateExceeded,
    BlockEditRateExceeded,
    ReadyToggleRateExceeded,
    SpeedExceeded,
    AccelerationExceeded,
}
```

### `ActionType`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionType {
    Input,
    Attack,
    BlockEdit,
    ReadyToggle,
}
```

### `SanctionType`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SanctionType {
    Warning,
    Kick { reason: String },
    Ban { reason: String, duration: Duration },
}
```

---

## Integration Contract

### Server Integration Points

The anti-cheat module integrates with the server at these points:

1. **Connection** (`handle_connect`):
   ```rust
   // Before accepting connection
   if let Some(ban) = self.ban_list.is_banned(&addr.ip()) {
       return self.send_rejected(addr, &format!("Banned: {}", ban.reason));
   }
   ```

2. **Message Dispatch** (`handle_message`):
   ```rust
   // For each ClientMessage::Input
   if !validate_input(&input).is_ok() {
       player.anti_cheat.record_infraction(InfractionType::InvalidFloat);
       return; // Drop input
   }
   if !player.anti_cheat.check_rate_limit(ActionType::Input, tick, &config) {
       player.anti_cheat.record_infraction(InfractionType::InputRateExceeded);
       return; // Drop input
   }
   ```

3. **Tick Processing** (`tick`):
   ```rust
   // After processing all players
   for player in sessions.iter_mut() {
       if let Some(sanction) = sanction_manager.evaluate(player.anti_cheat.strikes()) {
           self.apply_sanction(player, sanction);
       }
   }
   ```

---

## Test Contract

All anti-cheat modules must pass these test categories:

1. **Unit Tests**:
   - `validate_input` rejects NaN/INF
   - `check_rate_limit` triggers after exceeding limit
   - `evaluate` returns correct sanction for strike thresholds

2. **Integration Tests**:
   - Normal gameplay produces zero infractions
   - Load test with 16 bots produces zero false positives

3. **Property Tests** (optional):
   - No panic for any f32 bit pattern in input
   - Sanction escalation is monotonic
