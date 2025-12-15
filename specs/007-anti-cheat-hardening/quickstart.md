# Quickstart: Anti-Cheat Hardening

**Feature**: 007-anti-cheat-hardening
**Date**: 2025-12-15

## Prerequisites

- Rust 1.75+ (stable)
- Existing plix workspace builds (`cargo build --workspace`)
- Understanding of server tick loop (`crates/plix-server/src/lib.rs`)

## Development Setup

```bash
# Ensure workspace builds
cargo build --workspace

# Run existing tests (should all pass)
cargo test --workspace

# Run clippy (should have no warnings)
cargo clippy --workspace
```

## Key Files to Understand First

1. **Message Protocol** - `crates/plix-common/src/protocol/messages.rs`
   - `ClientMessage` enum - all client-to-server messages
   - `ServerMessage` enum - includes `Kicked`, `Rejected`
   - `PlayerInput` struct - movement/attack input with f32 fields

2. **Session Management** - `crates/plix-server/src/session.rs`
   - `ServerPlayer` struct - per-player state (will add `anti_cheat` field)
   - `SessionManager` - player lifecycle management

3. **Server Loop** - `crates/plix-server/src/lib.rs`
   - `Server::handle_message()` - message dispatch (add validation here)
   - `Server::tick()` - simulation tick (add sanction checks here)

4. **Existing Validation** - `crates/plix-server/src/validation.rs`
   - `InputValidator` - existing violation tracking (expand this)
   - Constants: `MAX_MOVE_SPEED`, `ATTACK_COOLDOWN_TICKS`, etc.

## Implementation Order

### Step 1: Add Anti-Cheat Module Structure

```bash
# Create module directory
mkdir -p crates/plix-server/src/anti_cheat

# Create module files
touch crates/plix-server/src/anti_cheat/mod.rs
touch crates/plix-server/src/anti_cheat/config.rs
touch crates/plix-server/src/anti_cheat/state.rs
touch crates/plix-server/src/anti_cheat/ban_list.rs
touch crates/plix-server/src/anti_cheat/sanctions.rs
touch crates/plix-server/src/anti_cheat/validation.rs
```

### Step 2: Implement Config (simplest)

```rust
// crates/plix-server/src/anti_cheat/config.rs
pub struct AntiCheatConfig {
    pub max_inputs_per_second: u32,
    // ... see data-model.md for full struct
}

impl Default for AntiCheatConfig {
    fn default() -> Self {
        Self {
            max_inputs_per_second: 120,
            max_attacks_per_second: 4,
            max_block_edits_per_second: 10,
            max_ready_toggles_per_second: 5,
            max_speed_per_tick: 0.25,
            max_acceleration: 1.5,
            warning_threshold: 3,
            kick_threshold: 5,
            ban_threshold: 10,
            ban_duration_seconds: 3600,
        }
    }
}
```

### Step 3: Implement Validation Functions

```rust
// crates/plix-server/src/anti_cheat/validation.rs
use plix_common::protocol::PlayerInput;

pub fn validate_input(input: &PlayerInput) -> Result<(), InfractionType> {
    // Check all f32 fields for NaN/INF
    if !input.move_forward.is_finite() { return Err(InfractionType::InvalidFloat); }
    if !input.move_right.is_finite() { return Err(InfractionType::InvalidFloat); }
    if !input.yaw.is_finite() { return Err(InfractionType::InvalidFloat); }
    if !input.pitch.is_finite() { return Err(InfractionType::InvalidFloat); }
    Ok(())
}
```

### Step 4: Implement Rate Limiter State

```rust
// crates/plix-server/src/anti_cheat/state.rs
pub struct AntiCheatState {
    pub strikes: u32,
    input_count: u32,
    attack_count: u32,
    edit_count: u32,
    ready_count: u32,
    window_start: Tick,
}

impl AntiCheatState {
    pub fn check_rate_limit(&mut self, action: ActionType, tick: Tick, config: &AntiCheatConfig) -> bool {
        // Reset window if >60 ticks (1 second at 60Hz)
        if tick.0.wrapping_sub(self.window_start.0) >= 60 {
            self.reset_window(tick);
        }

        let (count, limit) = match action {
            ActionType::Input => (&mut self.input_count, config.max_inputs_per_second),
            // ... other actions
        };

        if *count >= limit {
            false
        } else {
            *count += 1;
            true
        }
    }
}
```

### Step 5: Integrate into Server

```rust
// In crates/plix-server/src/lib.rs, handle_message()

// Before queuing input:
if let Err(infraction) = validate_input(&input) {
    player.anti_cheat.record_infraction(infraction);
    return; // Drop invalid input
}

if !player.anti_cheat.check_rate_limit(ActionType::Input, self.tick, &self.config) {
    player.anti_cheat.record_infraction(InfractionType::InputRateExceeded);
    return; // Rate limited
}

// Queue valid input
player.queue_input(input);
```

## Testing Commands

```bash
# Run all tests
cargo test --workspace

# Run only anti-cheat tests
cargo test --package plix-server anti_cheat

# Run with logging
RUST_LOG=debug cargo test --package plix-server anti_cheat -- --nocapture

# Run load test (requires running server)
cargo test --package plix-tools --test load_test -- --ignored --nocapture
```

## Verification Checklist

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes (no regressions)
- [ ] `cargo clippy --workspace` has no warnings
- [ ] Manual test: send NaN input, verify rejection
- [ ] Manual test: spam inputs, verify rate limiting
- [ ] Load test: 16 bots, verify no false positives

## Common Issues

### Issue: "trait bound not satisfied" for Default
Add `#[derive(Default)]` to `AntiCheatState` or implement manually.

### Issue: "cannot borrow as mutable" in handle_message
The server borrows sessions mutably. Make sure anti-cheat checks happen before any other mutable borrow.

### Issue: Load test shows false positives
Check rate limits - default 120 inputs/sec should allow 2 seconds of lag burst.
Check window reset logic - ensure it resets at 60 tick intervals.

## Architecture Overview

```
handle_packet()
    │
    ▼
handle_message() ─────────────────────────────────────┐
    │                                                 │
    ├─ ClientMessage::Connect                         │
    │   └─ ban_list.is_banned(ip) → Reject if banned │
    │                                                 │
    ├─ ClientMessage::Input(input)                    │
    │   ├─ validate_input() → Reject if NaN/INF      │
    │   ├─ check_rate_limit() → Reject if over limit │
    │   └─ check_sequence() → Reject if duplicate    │
    │                                                 │
    └─ ClientMessage::Attack/BlockEdit/ReadyToggle    │
        └─ check_rate_limit() → Reject if over limit │
                                                      │
tick() ◄──────────────────────────────────────────────┘
    │
    ├─ Process inputs (already validated)
    │
    └─ For each player:
        └─ evaluate(strikes) → Apply sanction if threshold
            ├─ Warning → Send ServerMessage::Warning
            ├─ Kick → Disconnect, allow reconnect
            └─ Ban → Add to ban_list, disconnect
```
