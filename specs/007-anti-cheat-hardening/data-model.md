# Data Model: Anti-Cheat Hardening

**Feature**: 007-anti-cheat-hardening
**Date**: 2025-12-15
**Status**: Complete

## Entities

### AntiCheatConfig

Configuration for all anti-cheat thresholds and limits.

```rust
pub struct AntiCheatConfig {
    // Rate limits (per second)
    pub max_inputs_per_second: u32,       // Default: 120
    pub max_attacks_per_second: u32,      // Default: 4
    pub max_block_edits_per_second: u32,  // Default: 10
    pub max_ready_toggles_per_second: u32, // Default: 2

    // Physics limits (per tick at 60Hz)
    pub max_speed_per_tick: f32,          // Default: 0.25 (15 blocks/sec)
    pub max_acceleration: f32,            // Default: 1.5

    // Sanction thresholds
    pub warning_threshold: u32,           // Default: 3
    pub kick_threshold: u32,              // Default: 5
    pub ban_threshold: u32,               // Default: 10

    // Ban duration
    pub ban_duration_seconds: u64,        // Default: 3600 (1 hour)
}
```

**Validation Rules**:
- All numeric fields must be > 0
- `warning_threshold < kick_threshold < ban_threshold`
- `max_speed_per_tick` should be >= movement system max speed

**Lifecycle**: Created at server startup, immutable during runtime.

---

### AntiCheatState

Per-player anti-cheat tracking state. Embedded in `ServerPlayer`.

```rust
pub struct AntiCheatState {
    // Infraction tracking
    pub strike_count: u32,
    pub last_warning_tick: Option<Tick>,

    // Rate limiting (fixed window counters)
    pub input_count: u32,
    pub attack_count: u32,
    pub block_edit_count: u32,
    pub ready_toggle_count: u32,
    pub window_start_tick: Tick,

    // Sequence validation
    pub last_input_seq: InputSeq,
    pub last_input_tick: Tick,
}
```

**Validation Rules**:
- `strike_count` resets on kick (not on warning)
- Rate counters reset when `current_tick - window_start_tick >= 60` (1 second)

**Lifecycle**: Created when player connects, destroyed when player disconnects.

**State Transitions**:
```
[Normal] -> (infraction) -> [Warning Pending] -> (more infractions) -> [Kick Pending] -> [Ban Pending]
     ^                            |                                           |
     +------- (window reset) -----+                                           |
                                                                              v
                                                                        [Disconnected]
```

---

### BanEntry

Represents a temporary ban in the ban list.

```rust
pub struct BanEntry {
    pub ip: IpAddr,
    pub reason: String,
    pub expires_at: Instant,
    pub strike_count: u32,  // For logging/analytics
}
```

**Validation Rules**:
- `reason` max length: 256 characters
- `expires_at` must be in the future when created

**Lifecycle**: Created when ban threshold reached, removed when expired or server restarts.

---

### BanList

Global ban list storage.

```rust
pub struct BanList {
    bans: HashMap<IpAddr, BanEntry>,
}

impl BanList {
    pub fn is_banned(&self, ip: &IpAddr) -> Option<&BanEntry>;
    pub fn add_ban(&mut self, entry: BanEntry);
    pub fn remove_expired(&mut self);
    pub fn unban(&mut self, ip: &IpAddr) -> bool;
}
```

**Validation Rules**:
- Duplicate IPs overwrite existing ban (extends duration)
- `remove_expired()` called periodically (every 60 seconds)

**Lifecycle**: Created at server startup, persists until server shutdown.

---

### InfractionType

Enum for categorizing infractions (for logging/analytics).

```rust
pub enum InfractionType {
    // Input validation
    InvalidFloat,        // NaN or INF
    OutOfBounds,         // Position/rotation outside valid range
    InvalidSequence,     // Out-of-order or duplicate input

    // Rate limiting
    InputRateExceeded,
    AttackRateExceeded,
    BlockEditRateExceeded,
    ReadyToggleRateExceeded,

    // Physics sanity
    SpeedExceeded,
    AccelerationExceeded,
    TeleportAttempt,
}
```

---

### SanctionType

Enum for sanction actions.

```rust
pub enum SanctionType {
    Warning,
    Kick,
    Ban { duration_seconds: u64 },
}
```

---

## Protocol Extensions

### New Server Message

```rust
pub enum ServerMessage {
    // ... existing variants ...

    /// Warning sent to client (anti-cheat)
    Warning {
        reason: String,
        strike_count: u32,
        kick_threshold: u32,
    },
}
```

---

## Relationships

```
┌─────────────────┐     1:1     ┌──────────────────┐
│  ServerPlayer   │────────────▶│  AntiCheatState  │
└─────────────────┘             └──────────────────┘
        │
        │ addr.ip()
        ▼
┌─────────────────┐     1:*     ┌──────────────────┐
│    BanList      │────────────▶│    BanEntry      │
└─────────────────┘             └──────────────────┘
        │
        │ uses
        ▼
┌─────────────────┐
│ AntiCheatConfig │
└─────────────────┘
```

---

## Integration with Existing Types

### ServerPlayer Extension

Add `anti_cheat: AntiCheatState` field to `ServerPlayer` struct in `session.rs`:

```rust
pub struct ServerPlayer {
    // ... existing fields ...

    /// Anti-cheat state
    pub anti_cheat: AntiCheatState,
}
```

### PlixServer Extension

Add `ban_list: BanList` and `anti_cheat_config: AntiCheatConfig` to `PlixServer`:

```rust
pub struct PlixServer {
    // ... existing fields ...

    /// Anti-cheat configuration
    anti_cheat_config: AntiCheatConfig,

    /// Global ban list
    ban_list: BanList,
}
```

---

## Invariants

1. **No panics**: All anti-cheat code must handle all inputs without panicking
2. **Bounded memory**: Ban list size bounded by unique IPs (realistic max ~1000)
3. **Deterministic**: Same inputs always produce same sanction decisions
4. **No heap per tick**: Rate limit checks use pre-allocated counters only
5. **Time monotonic**: Tick-based timestamps always increase
