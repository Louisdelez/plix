# Research: Anti-Cheat Hardening

**Feature**: 007-anti-cheat-hardening
**Date**: 2025-12-15
**Status**: Complete

## Codebase Analysis

### Existing Anti-Cheat Foundation

The plix-server crate already has a basic anti-cheat foundation in `validation.rs`:

```rust
pub struct InputValidator {
    violations: u32,  // Escalating system
}

impl InputValidator {
    pub fn validate(&mut self, input: &mut PlayerInput) -> ValidationResult;
    pub fn validate_movement(&mut self, old_pos: Vec3, new_pos: Vec3) -> ValidationResult;
    pub fn should_kick(&self) -> bool { self.violations >= 100 }
}
```

**Decision**: Extend existing `InputValidator` rather than replace it. Add new `anticheat/` module for additional functionality.
**Rationale**: Preserves existing tests and behavior, follows constitution principle of no unnecessary complexity.
**Alternatives considered**: New crate (rejected - overkill for MVP), replace entirely (rejected - would break existing tests).

### Client Message Types

All 6 client message types identified for validation:

| Message | Frequency | Current Validation | Anti-Cheat Priority |
|---------|-----------|-------------------|---------------------|
| `Input(PlayerInput)` | 60/sec | Input clamping | HIGH - rate limit + NaN check |
| `BlockEdit(BlockEditRequest)` | Variable | Cooldown, range, phase | MEDIUM - already validated |
| `ReadyToggle` | Rare | Phase check only | LOW - add rate limit |
| `Connect` | Once | Version, capacity | HIGH - ban check |
| `Disconnect` | Once | None | N/A |
| `SnapshotAck` | Variable | None | LOW - timing only |

**Decision**: Focus on `Input`, `Connect`, and add rate limiting to all action messages.
**Rationale**: Input is highest frequency and most exploitable. Connect is entry point for ban enforcement.

### Rate Limiting Approach

**Decision**: Fixed window counters (1-second windows)
**Rationale**:
- Simpler than token bucket
- Predictable behavior for testing
- Aligns with existing `last_edit_tick` pattern
- O(1) space and time per check

**Alternatives considered**:
- Token bucket (rejected - more complex, not needed for MVP)
- Sliding window (rejected - more memory, marginal benefit)

### Protocol Messages

Existing kick/rejection messages:

```rust
pub enum ServerMessage {
    Rejected { reason: String },  // Pre-connection
    Kicked { reason: String },    // In-game
    // ...
}
```

**Decision**: Add `Warning` message for strike notifications
**Rationale**: Players should know they're accumulating strikes before being kicked.

### Movement Validation

Server is already authoritative - client positions are suggestions only:

```rust
// In simulate_tick() - line 350-354 of lib.rs
let new_pos = self.movement.move_player(player.position, player.velocity, &input, dt);
player.position = new_pos;  // Server calculates position
```

**Decision**: Movement sanity checks are optional/low priority
**Rationale**: Server already ignores client positions. Only need to detect if client *claims* impossible positions (which we already reject via `validate_movement()`).

### Ban Identification

**Decision**: Use IP address for MVP ban identification
**Rationale**:
- No account system exists
- IP bans are simple and effective for casual cheaters
- Can be bypassed (VPN) but acceptable for MVP scope
- Socket address already available: `player.addr`

**Alternatives considered**:
- HWID bans (out of scope per spec)
- Account bans (no account system)

## Performance Analysis

### Per-Tick Budget

At 60 Hz with 32 players:
- Total budget per tick: 16.67ms
- Per-player anti-cheat budget: < 1µs (spec requirement)
- Total anti-cheat overhead: < 32µs per tick (0.2% of budget)

**Decision**: All checks must be O(1) with no heap allocations
**Rationale**: Meets spec requirement, leaves headroom for future expansion.

### Memory Budget

Per-player anti-cheat state:
- Strike counter: 4 bytes
- Rate limit counters (4 actions × 4 bytes): 16 bytes
- Last action timestamps (4 × 4 bytes): 16 bytes
- **Total per player**: ~36 bytes

Global state:
- Ban list: ~100 bytes per entry (IP + timestamp + reason)
- **Estimated**: < 1KB for typical ban list

**Decision**: Inline anti-cheat state in `ServerPlayer` struct
**Rationale**: Avoids HashMap lookup per message, cache-friendly.

## Integration Strategy

### Phase 1: Strict Validation (FR-001, FR-002, FR-003)
Extend `InputValidator` with:
- `is_finite()` checks for all floats
- Bounds validation for position/rotation
- Sequence number monotonicity check

### Phase 2: Rate Limiting (FR-004, FR-005)
Add `RateLimiter` struct with:
- Per-action counters
- Configurable limits via `AntiCheatConfig`
- Integration at message dispatch

### Phase 3: Sanctions (FR-008 through FR-014)
Add `SanctionManager` with:
- Per-player strike tracking
- Warning/kick/ban escalation
- In-memory ban list with expiry

### Phase 4: Observability
Add structured logging:
- `info!()` for kicks/bans
- `warn!()` for rate limit violations
- `debug!()` for individual rejections (disabled by default)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| False positives | Low | High | 2x rate limits, extensive testing |
| Performance regression | Low | Medium | Benchmark tests, O(1) algorithms |
| Bypass via IP change | Medium | Low | Acceptable for MVP |
| Complex edge cases | Medium | Medium | Thorough edge case testing |

## Conclusion

The existing codebase provides a solid foundation:
- `InputValidator` already tracks violations
- Server is already authoritative for positions
- Block edits already have rate limiting
- Protocol already has kick/reject messages

The anti-cheat feature is primarily about:
1. Adding NaN/INF validation (new)
2. Extending rate limiting to all actions (extend existing pattern)
3. Adding sanction escalation (new)
4. Adding ban list (new)

Estimated implementation: 500-800 lines of new code, primarily in `anticheat/` module.
