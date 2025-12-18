# Quickstart: Account Identity

**Feature**: 025-account-identity
**Date**: 2025-12-17

## Overview

This feature adds simple player identity with:
1. **Display names** - Configurable, validated, and disambiguated
2. **Local profile** - Persisted in `~/.config/plix/profile.toml`
3. **Session identity** - Unique SessionId for logging/metrics
4. **Name changes** - `/name` command with rate limiting

## Implementation Order

### Phase 1: Core Types (plix-common)

```bash
# Create identity module
mkdir -p crates/plix-common/src/identity
```

1. **DisplayName type** (`identity/display_name.rs`)
   - Validation rules (1-32 chars, alphanumeric + `_- `)
   - Sanitization helper
   - Base name extraction

2. **SessionId type** (`identity/session.rs`)
   - Simple wrapper struct
   - Display implementation

3. **Protocol extensions** (`protocol/messages.rs`)
   - Add `RenameRequest` to `ClientMessage`
   - Add `RenameResult`, `PlayerRenamed` to `GameEvent`
   - Add `display_name` to `PlayerSnapshot`
   - Add `display_name`, `session_id` to `Connected`

### Phase 2: Server Implementation (plix-server)

```bash
# Create identity module
mkdir -p crates/plix-server/src/identity
```

1. **NameRegistry** (`identity/name_registry.rs`)
   - Registration with auto-disambiguation
   - Unregistration on disconnect
   - Rename support

2. **Session extensions** (`session.rs`)
   - Add `session_id: SessionId` to `ServerPlayer`
   - Add `last_rename_tick: Option<Tick>` to `ServerPlayer`
   - Add rename cooldown check helper

3. **Netloop integration** (`netloop.rs`)
   - Handle `RenameRequest`
   - Log with SessionId on connect/disconnect/rename
   - Broadcast `PlayerRenamed` events

4. **Replication** (`replication/state.rs`, `snapshot.rs`)
   - Include `display_name` in `ReplicatedPlayer`
   - Include `display_name` in snapshot generation

### Phase 3: Client Implementation (plix-client)

```bash
# Create profile module
mkdir -p crates/plix-client/src/profile
```

1. **PlayerProfile** (`profile/player_profile.rs`)
   - TOML load/save
   - Default creation
   - Corruption handling

2. **Console command** (`console.rs`)
   - Add `/name <new_name>` parsing
   - Update help text

3. **Connection flow** (`net.rs` or main)
   - Load profile on startup
   - Send preferred name in Connect
   - Handle assigned name in Connected response
   - Save profile on successful rename

## Key Files to Modify

| File | Changes |
|------|---------|
| `plix-common/src/lib.rs` | Add `pub mod identity;` |
| `plix-common/src/protocol/messages.rs` | New messages, extended structs |
| `plix-server/src/lib.rs` | Add `pub mod identity;` |
| `plix-server/src/session.rs` | SessionId, rename cooldown |
| `plix-server/src/netloop.rs` | Handle RenameRequest |
| `plix-server/src/replication/state.rs` | Add display_name to ReplicatedPlayer |
| `plix-server/src/replication/snapshot.rs` | Include display_name |
| `plix-client/src/lib.rs` | Add `pub mod profile;` |
| `plix-client/src/console.rs` | /name command |

## Testing Strategy

### Unit Tests (plix-common)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_name_valid() {
        assert!(DisplayName::new("Alice").is_ok());
        assert!(DisplayName::new("Player_123").is_ok());
        assert!(DisplayName::new("John-Doe").is_ok());
    }

    #[test]
    fn test_display_name_invalid() {
        assert!(DisplayName::new("").is_err());
        assert!(DisplayName::new("   ").is_err());
        assert!(DisplayName::new("a".repeat(33).as_str()).is_err());
        assert!(DisplayName::new("Invalid@Name!").is_err());
    }

    #[test]
    fn test_display_name_sanitize() {
        assert_eq!(DisplayName::sanitize("  Alice  ").as_str(), "Alice");
        assert_eq!(DisplayName::sanitize("Invalid@Name!").as_str(), "Player");
        assert_eq!(DisplayName::sanitize("").as_str(), "Player");
    }
}
```

### Unit Tests (plix-server)

```rust
#[test]
fn test_name_registry_unique() {
    let mut registry = NameRegistry::default();
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);

    let name1 = registry.register(p1, "Alex");
    let name2 = registry.register(p2, "Alex");

    assert_eq!(name1, "Alex");
    assert_eq!(name2, "Alex#2");
}

#[test]
fn test_name_registry_reuse_suffix() {
    let mut registry = NameRegistry::default();
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let p3 = PlayerId(3);

    registry.register(p1, "Alex");
    registry.register(p2, "Alex");
    registry.unregister(p2); // Free up "Alex#2"

    let name3 = registry.register(p3, "Alex");
    assert_eq!(name3, "Alex#2"); // Reuses lowest suffix
}

#[test]
fn test_rename_cooldown() {
    let mut player = ServerPlayer::new(/* ... */);
    let current = Tick(1000);
    let cooldown = 3600; // 60 seconds at 60 TPS

    assert!(player.can_rename(current, cooldown));
    player.record_rename(current);
    assert!(!player.can_rename(current, cooldown));
    assert!(!player.can_rename(Tick(1000 + 3599), cooldown));
    assert!(player.can_rename(Tick(1000 + 3600), cooldown));
}
```

### Integration Tests

```rust
// tests/identity_test.rs

#[test]
fn test_connect_with_valid_name() {
    // Connect client with preferred name
    // Verify Connected response has same name
    // Verify PlayerJoined event has correct name
}

#[test]
fn test_connect_with_invalid_name() {
    // Connect with invalid name (e.g., empty)
    // Verify server assigns fallback "Player"
}

#[test]
fn test_duplicate_name_disambiguation() {
    // Connect two clients with same name
    // Verify second gets #2 suffix
}

#[test]
fn test_rename_success() {
    // Connect, then send RenameRequest
    // Verify RenameResult success
    // Verify PlayerRenamed broadcast
}

#[test]
fn test_rename_rate_limited() {
    // Connect, rename, immediately try again
    // Verify RenameResult failure with RateLimited reason
}
```

## Profile File Example

**Location**: `~/.config/plix/profile.toml`

```toml
version = 1
display_name = "MyAwesomeName"
```

## Console Commands

```
/name <new_name>  - Change display name (60s cooldown)
/help             - Updated to include /name
```

## Success Criteria Verification

| Criterion | How to Verify |
|-----------|---------------|
| SC-001: Name visible < 1s | Time from Connect to UI update |
| SC-002: Persist across restarts | Close/reopen client, check profile loaded |
| SC-003: Disambiguation < 10ms | Benchmark NameRegistry::register with 100 players |
| SC-004: Validation < 1ms | Benchmark DisplayName::new |
| SC-005: 100 concurrent players | Load test with 100 connections |
| SC-006: 100% traceability | Verify all logs include SessionId |
| SC-007: Rate limit enforcement | Try rapid renames, verify rejection |
| SC-008: Backward compatible | Connect old client to new server |
