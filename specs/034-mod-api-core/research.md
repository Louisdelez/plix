# Research: Mod API Core

**Feature**: 034-mod-api-core
**Date**: 2025-12-18

## Research Questions

### 1. Event Bus Pattern for Game Mods

**Question**: What is the best pattern for event dispatch in a game mod system?

**Decision**: Phase-based dispatch with FIFO ordering and re-entrancy prevention

**Rationale**:
- Collecting events during tick and dispatching at end-of-tick prevents cascade effects
- FIFO ordering ensures deterministic behavior across game sessions
- Preventing handlers from triggering immediate dispatch avoids stack overflow and unpredictable state
- This pattern is used successfully in Minecraft Forge, Unity, and other mod frameworks

**Alternatives Considered**:
- Immediate dispatch: Rejected - causes re-entrancy issues and unpredictable state
- Priority-based ordering: Rejected - adds complexity, harder to debug
- Async dispatch: Rejected - determinism concerns in game simulation

### 2. Capability/Permission System Design

**Question**: How should mod permissions be structured?

**Decision**: Bitflag-based capability enum with server-side override

**Rationale**:
- Bitflags allow efficient permission checking (single AND operation)
- Enum provides type safety and exhaustive matching
- Server override allows admins to restrict mods beyond manifest declarations
- Follows principle of least privilege - mods must declare all needed capabilities

**Alternatives Considered**:
- String-based permissions: Rejected - no compile-time checking, typo-prone
- Hierarchical permissions: Rejected - over-engineering for MVP
- Runtime capability negotiation: Deferred to future feature

### 3. Mod Manifest Format

**Question**: TOML or JSON for mod manifests?

**Decision**: TOML (`mod.toml`)

**Rationale**:
- TOML is more human-readable for configuration files
- Consistent with Rust ecosystem (Cargo.toml)
- Better support for comments (useful for documentation)
- serde + toml crate is well-maintained and efficient

**Alternatives Considered**:
- JSON: Rejected - no comments, less readable
- YAML: Rejected - whitespace-sensitive, security concerns with arbitrary YAML
- RON: Rejected - less familiar to mod developers

### 4. Error Handling Strategy

**Question**: How should mod API errors be structured?

**Decision**: Typed error enum with error codes (EMOD001-007)

**Rationale**:
- Error codes enable programmatic handling and logging
- Structured errors with context (mod_id, api_call) aid debugging
- No panics in API - all errors return Result
- Consistent with existing plix error patterns (ECHAT, EEMB, etc.)

**Alternatives Considered**:
- String errors: Rejected - no type safety, harder to handle
- anyhow/eyre: Rejected - need stable error codes for mod developers
- Panic on invalid input: Rejected - violates constitution (no panics)

### 5. Timer Implementation

**Question**: How should mod timers integrate with game tick?

**Decision**: Separate timer storage per mod, processed in dedicated tick phase

**Rationale**:
- Per-mod storage enables easy cleanup on mod disable
- Dedicated phase ensures timers don't interfere with game simulation
- Minimum interval (50ms) prevents timer spam
- Maximum count (32) prevents resource exhaustion

**Alternatives Considered**:
- Global timer pool: Rejected - harder to isolate/disable per mod
- OS timers: Rejected - not deterministic, platform-dependent
- Tick-count based: Rejected - harder for mod developers to reason about

### 6. Entity Handle Design

**Question**: How should entities be exposed to mods?

**Decision**: Opaque EntityHandle with generation counter for safety

**Rationale**:
- Opaque handles prevent mods from accessing ECS internals
- Generation counter detects use-after-despawn (EMOD003)
- Consistent with existing plix entity system
- Enables future optimization without API changes

**Alternatives Considered**:
- Direct entity ID: Rejected - no safety against stale IDs
- Full entity proxy objects: Rejected - memory overhead, sync complexity
- ECS component access: Rejected - violates isolation principle

### 7. Network Channel Naming

**Question**: How should mod network channels be identified?

**Decision**: Format `mod:<mod_id>:<channel_name>`

**Rationale**:
- Namespace prevents collision between mods
- Clear ownership (mod_id prefix)
- Allows mods to define multiple channels
- Easy to filter/route server-side

**Alternatives Considered**:
- Numeric channel IDs: Rejected - collision risk, less readable
- Flat namespace: Rejected - mod collision inevitable
- Hierarchical paths: Rejected - over-engineering for MVP

### 8. Rate Limiting Approach

**Question**: How should rate limiting be implemented?

**Decision**: Token bucket per mod with configurable limits

**Rationale**:
- Token bucket allows burst traffic while enforcing average rate
- Per-mod limits prevent one mod from starving others
- Configurable via server settings for different use cases
- Standard pattern, well-understood behavior

**Alternatives Considered**:
- Fixed window: Rejected - allows burst at window boundary
- Sliding window: More complex, similar results to token bucket
- No rate limiting: Rejected - abuse vector

## Technical Findings

### Existing Plix Patterns

Reviewed existing crates for consistency:

1. **Error patterns** (plix-client/ui_cef/bridge/messages.rs):
   - Error codes: EBRG001-003, ECFG001-002, ESRV001-002, etc.
   - BridgeError struct with code + message
   - Result-based returns

2. **Event patterns** (plix-server):
   - Server tick-based processing
   - Event queues for deferred processing

3. **Type patterns** (plix-common):
   - Shared types between crates
   - Serialization with serde + bincode

### Dependencies to Add

```toml
# plix-mod-core/Cargo.toml
[dependencies]
plix-common = { path = "../plix-common" }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
tracing = "0.1"
glam = "0.29"
bitflags = "2.4"

[dev-dependencies]
tempfile = "3.10"
```

### Test Strategy

1. **Unit tests**: Each module tested in isolation
   - manifest.rs: Valid/invalid parsing
   - capabilities.rs: Permission checks
   - errors.rs: Error code generation
   - timers.rs: Limit enforcement

2. **Integration tests**: Dummy mod exercises full API
   - Event subscription and receipt
   - Permission denied scenarios
   - Rate limiting behavior
   - Error threshold and auto-disable

## Resolved Unknowns

All technical context items resolved:

| Item | Resolution |
|------|------------|
| Language/Version | Rust 1.75+ stable |
| Dependencies | serde, toml, tracing, glam, bitflags |
| Storage | In-memory (registry, subscriptions, timers) |
| Testing | cargo test with dummy mod |
| Platform | Linux server (plix-server integration) |
| Performance | Event dispatch <1 tick, permission check <1ms |
