# Quickstart: Mod API Core

**Feature**: 034-mod-api-core
**Date**: 2025-12-18

## Overview

This guide covers the implementation order for Feature 034 — Mod API Core.

## Prerequisites

- Rust 1.75+ (stable)
- plix workspace set up
- Understanding of existing crates (plix-common, plix-server)

## Implementation Order

### Phase 1: Foundation (errors, capabilities, manifest)

1. **Create `plix-mod-core` crate**
   ```bash
   cd crates
   cargo new plix-mod-core --lib
   ```

2. **Implement `errors.rs`**
   - ModApiError struct
   - ErrorCode enum (EMOD001-007)
   - ErrorContext struct
   - Helper functions for each error type

3. **Implement `capabilities.rs`**
   - Capability bitflags
   - `require()` helper for permission checks
   - Capability parsing from strings

4. **Implement `manifest.rs`**
   - ModManifest struct with serde derives
   - TOML parsing
   - Validation logic
   - API version checking

### Phase 2: Registry & Event Bus

5. **Implement `registry.rs`**
   - ModContext struct
   - ModState enum
   - ModRegistry (HashMap<ModId, ModContext>)
   - Load/unload/disable operations
   - Error counter tracking

6. **Implement `events.rs`**
   - EventType enum
   - Event payload structs
   - GameEvent struct
   - EventBus struct
   - Subscription management
   - Phase-based dispatch
   - Cancellation handling
   - Error isolation (5 consecutive → disable)

### Phase 3: APIs

7. **Implement `api/world.rs`**
   - get_block with capability check
   - set_block with capability check
   - raycast with bounds (256 max)
   - query_aabb with limit (128 max)
   - Chunk loaded validation

8. **Implement `api/entities.rs`**
   - EntityHandle struct
   - Read functions (transform, velocity, health, owner, team)
   - Write functions (apply_damage, apply_impulse)
   - spawn/despawn with permission checks

9. **Implement `api/net.rs`**
   - ModChannel struct
   - Channel parsing (mod:id:name format)
   - send_message with capability check
   - Rate limiting (20 msg/s)
   - Payload size check (8KB max)

10. **Implement `api/timers.rs`**
    - TimerHandle struct
    - set_timeout with min clamp (50ms)
    - set_interval with min clamp (50ms)
    - clear_timer
    - Max timer enforcement (32)

### Phase 4: Observability & Integration

11. **Implement `observability.rs`**
    - Structured logging with tracing
    - Metrics counters
    - Error logging with context

12. **Implement `lib.rs`**
    - Public exports
    - ModHost trait definition
    - Crate documentation

### Phase 5: Server Integration

13. **Create `plix-server/src/mods/mod.rs`**
    - Mod loading from filesystem
    - Integration with game loop (tick dispatch)
    - Event emission points

14. **Add shared types to `plix-common`**
    - ModId type
    - Shared event types if needed

### Phase 6: Testing

15. **Unit tests** for each module:
    - manifest parsing (valid/invalid)
    - capability checks
    - error generation
    - bounds enforcement
    - rate limiting

16. **Integration tests** with dummy mod:
    - Event subscription and receipt
    - API calls with/without permissions
    - Error threshold and auto-disable
    - Full workflow test

## Key Files to Create

```
crates/plix-mod-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── errors.rs
    ├── capabilities.rs
    ├── manifest.rs
    ├── registry.rs
    ├── events.rs
    ├── observability.rs
    └── api/
        ├── mod.rs
        ├── world.rs
        ├── entities.rs
        ├── net.rs
        └── timers.rs
```

## Cargo.toml Template

```toml
[package]
name = "plix-mod-core"
version = "0.1.0"
edition = "2021"

[dependencies]
plix-common = { path = "../plix-common" }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
glam = "0.29"
bitflags = "2.4"
thiserror = "1.0"

[dev-dependencies]
tempfile = "3.10"
```

## Testing Commands

```bash
# Run all tests
cargo test -p plix-mod-core

# Run specific test module
cargo test -p plix-mod-core manifest

# Run with verbose output
cargo test -p plix-mod-core -- --nocapture

# Check formatting
cargo fmt -p plix-mod-core -- --check

# Run clippy
cargo clippy -p plix-mod-core
```

## Example mod.toml

```toml
id = "example-mod"
name = "Example Mod"
version = "1.0.0"
author = "Developer"
api_version = 1

[capabilities]
world_read = true
world_write = true
entity_read = true
net_send = true
event_cancel_chat = true

[entrypoints]
server = "on_load"
```

## Success Criteria Checklist

- [ ] `plix-mod-core` crate compiles
- [ ] All error codes implemented
- [ ] Manifest parsing works (valid + invalid cases)
- [ ] Capabilities enforce permissions
- [ ] Event bus dispatches in FIFO order
- [ ] Cancellation works for allowed events only
- [ ] World API respects bounds (256/128)
- [ ] Timer API respects limits (50ms/32)
- [ ] Network API enforces rate limit (20 msg/s)
- [ ] Auto-disable after 5 consecutive errors
- [ ] All unit tests pass
- [ ] Integration test with dummy mod passes
- [ ] cargo clippy passes
- [ ] cargo fmt passes
