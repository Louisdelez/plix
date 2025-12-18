# Research: Account Identity

**Feature**: 025-account-identity
**Date**: 2025-12-17

## Research Tasks

### 1. Display Name Validation Rules

**Question**: What validation rules should apply to display names?

**Decision**: Use spec-defined rules (FR-002 to FR-006):
- Minimum: 1 character after trimming whitespace
- Maximum: 32 characters
- Allowed: alphanumeric (a-z, A-Z, 0-9), underscores, hyphens, spaces
- Trim leading/trailing whitespace
- Fallback: "Player" if empty/invalid after sanitization

**Rationale**:
- 32-char limit matches common game conventions (Steam, Discord)
- Alphanumeric + underscore/hyphen/space covers most reasonable names
- Simple ASCII subset avoids Unicode normalization complexity in v1

**Alternatives Considered**:
- Full Unicode support: Rejected for v1 - adds normalization complexity, homoglyph detection
- Profanity filter: Explicitly out of scope per spec
- Stricter 3-16 char range: User input specified 1-32, so we follow spec

---

### 2. Name Disambiguation Strategy

**Question**: How to handle duplicate display names on same server?

**Decision**: Automatic suffix with `#N` format (lowest available):
- First "Alex" → "Alex"
- Second "Alex" → "Alex#2"
- If "Alex#2" disconnects, next "Alex" gets "Alex#2" again

**Rationale**:
- Common pattern (Discord, Battle.net)
- Non-intrusive - base name preserved
- Easy to implement with HashSet tracking

**Alternatives Considered**:
- Reject duplicate names: Poor UX, user must guess available names
- UUID suffix: Ugly and not memorable
- Random adjective prefix: Inconsistent experience

**Implementation Notes**:
- Track base names in `HashSet<String>` for O(1) lookup
- Track suffixes per base name: `HashMap<String, HashSet<u32>>`
- Max suffix check at 99 (per spec edge case)

---

### 3. Session Identity Design

**Question**: What identifier to use for session correlation?

**Decision**: 64-bit SessionId (monotonic counter per server boot):
```rust
pub struct SessionId(pub u64);
```

**Rationale**:
- Simple monotonic counter, no UUID overhead
- 64-bit sufficient for ~18 quintillion sessions
- Server-local only, no cross-server correlation needed in v1

**Alternatives Considered**:
- UUID v4: Overkill for single-server scope, adds 16 bytes
- Timestamp-based: Monotonic counter is simpler and sufficient
- PlayerId reuse: PlayerId can be reassigned, SessionId should be unique per connection

---

### 4. Profile Storage Format

**Question**: What format and location for client profile?

**Decision**: TOML file at `~/.config/plix/profile.toml`:
```toml
# Plix Player Profile
version = 1

[identity]
display_name = "Player"

# Reserved for future auth
# account_id = ""
# auth_token = ""
```

**Rationale**:
- TOML already used for `config.toml` - consistency
- XDG compliant on Linux
- Human-readable and editable
- Version field for future migrations

**Alternatives Considered**:
- JSON: Valid but TOML already in use for game config
- Binary (bincode): Not human-readable, harder to debug
- Merge with config.toml: Separation of concerns - config = settings, profile = identity

---

### 5. Rate Limiting Strategy

**Question**: How to implement name change rate limiting?

**Decision**: Per-player cooldown tracking in `ServerPlayer`:
```rust
pub struct ServerPlayer {
    // ...
    pub last_rename_tick: Option<Tick>,
}
```
Rate limit: 1 rename per 60 seconds (3600 ticks at 60 TPS)

**Rationale**:
- Simple tick-based cooldown, no additional data structures
- 60 seconds balances user flexibility with spam prevention
- Aligns with existing cooldown patterns (combat, block edit)

**Alternatives Considered**:
- Token bucket: Overkill for simple rename limiting
- Shorter cooldown (10s): Allows name spam in matches
- Longer cooldown (5min): Too restrictive for typo corrections

---

### 6. Protocol Extension Strategy

**Question**: How to extend protocol for identity without breaking existing clients?

**Decision**:
- Add `display_name` field to `PlayerSnapshot` (new field, default "")
- Add `RenameRequest` / `RenameResult` to `ClientMessage` / `ServerMessage`
- Add optional `account_id: Option<u64>` and `auth_token: Option<String>` to `Connect` (unused in v1)

**Rationale**:
- serde with `#[serde(default)]` handles missing fields gracefully
- Protocol version already in Connect message for compatibility checks
- Optional fields allow v1 clients to connect to v2 servers

**Alternatives Considered**:
- New message types only: Loses name in snapshot, requires separate query
- Bump protocol version: Unnecessary for additive changes with defaults

---

### 7. Existing Codebase Integration Points

**Analysis**: Key files to modify

| File | Changes |
|------|---------|
| `plix-common/src/protocol/messages.rs` | Add RenameRequest, RenameResult, display_name to PlayerSnapshot |
| `plix-server/src/session.rs` | Add SessionId, last_rename_tick to ServerPlayer |
| `plix-server/src/netloop.rs` | Handle RenameRequest, log with SessionId |
| `plix-client/src/console.rs` | Add /name command parsing |
| `plix-client/src/config.rs` | Profile is separate file, but follows same pattern |

**Existing Patterns to Follow**:
- `GameConfig` in `plix-client/src/config.rs` - load/save TOML pattern
- `PlayerInput` rate limiting in `session.rs` - cooldown pattern
- `CommandResult` in `console.rs` - command parsing pattern
- `tracing::info!` structured logging throughout server

---

## Summary

All research questions resolved. No blockers identified.

**Key Decisions**:
1. ASCII alphanumeric + underscore/hyphen/space for names (1-32 chars)
2. `#N` suffix disambiguation
3. Monotonic 64-bit SessionId
4. Separate `profile.toml` file (TOML format, versioned)
5. Tick-based 60-second rename cooldown
6. Additive protocol changes with `#[serde(default)]`
