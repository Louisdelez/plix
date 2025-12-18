# Research: Server Browser v1

**Feature**: 026-server-browser
**Date**: 2025-12-17

## Research Topics

### 1. HTTP Framework for Master Server

**Decision**: Use `axum` for the master server HTTP API

**Rationale**:
- Native tokio integration (already in workspace)
- Lightweight and fast (no runtime overhead)
- Tower middleware ecosystem (rate limiting, logging)
- First-class async/await support
- Active maintenance by tokio team
- Type-safe routing with extractors

**Alternatives Considered**:
- `actix-web`: More mature but heavier, different async runtime preferences
- `warp`: Good but less ecosystem support than axum
- `hyper` directly: Too low-level for this use case
- `rocket`: Requires nightly Rust (violates constitution)

### 2. HTTP Client for Server/Client

**Decision**: Use `reqwest` for HTTP client operations

**Rationale**:
- De facto standard Rust HTTP client
- Native tokio integration
- Supports timeouts, connection pooling
- JSON serialization built-in
- Well-tested in production

**Alternatives Considered**:
- `hyper` client: Too low-level
- `ureq`: Blocking only, not suitable for async context
- `surf`: Less mature ecosystem

### 3. Server ID Generation Strategy

**Decision**: Hash of `host:port` using `std::hash::DefaultHasher`

**Rationale**:
- Deterministic: same host:port always produces same ID
- No external dependencies
- Fast computation
- Collision-resistant for practical purposes
- Client can compute for favorites matching

**Implementation**:
```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn generate_server_id(host: &str, port: u16) -> String {
    let mut hasher = DefaultHasher::new();
    host.hash(&mut hasher);
    port.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
```

**Alternatives Considered**:
- UUID v4: Not deterministic, requires persistence
- UUID v5 (namespace): More complex than needed
- Sequential IDs: Requires persistence

### 4. Rate Limiting Strategy

**Decision**: In-memory sliding window counter per IP

**Rationale**:
- Simple and effective for MVP
- No external dependencies
- Handles burst + sustained rate limiting
- Memory bounded (HashMap with cleanup)

**Implementation Approach**:
- `HashMap<IpAddr, Vec<Instant>>` for request timestamps
- Max 10 requests per 60 seconds per IP
- Cleanup old entries on each request
- Tower middleware layer for clean integration

**Alternatives Considered**:
- Token bucket: More complex, overkill for MVP
- Redis-based: External dependency, violates simplicity
- No rate limiting: Security risk

### 5. Server Entry TTL and Cleanup

**Decision**: Lazy cleanup on read operations + periodic background task

**Rationale**:
- No stop-the-world cleanup pauses
- Expired entries naturally filtered on GET /servers
- Background task prevents unbounded growth
- Simple implementation

**Implementation**:
- TTL: 60 seconds (configurable)
- Heartbeat interval: 20 seconds (gives 3 attempts before expiration)
- Background cleanup every 30 seconds (removes entries > 2x TTL)

### 6. Favorites Storage Format

**Decision**: TOML file at `~/.config/plix/servers.toml`

**Rationale**:
- Human-readable and editable
- Already in workspace dependencies
- Consistent with existing config patterns (profile.toml)
- Simple structure, no complex queries needed

**Format**:
```toml
[favorites]
servers = [
    { id = "abc123...", name = "My Server", host = "192.168.1.1", port = 7777 },
]

[settings]
master_url = "http://master.plix.game:8080"
```

### 7. Console Command Parsing Extensions

**Decision**: Extend existing `console.rs` with server browser commands

**Rationale**:
- Consistent with existing command pattern
- Reuses `CommandResult` enum
- Single entry point for all console commands
- Test coverage pattern already established

**New Commands**:
- `/servers` - Fetch and display server list
- `/servers <search>` - Search servers
- `/connect <index>` - Connect to server by list index
- `/favorite <index>` - Add server to favorites
- `/unfavorite <index>` - Remove server from favorites
- `/favorites` - List saved favorites

### 8. Async HTTP in Game Client

**Decision**: Spawn dedicated async task for HTTP operations

**Rationale**:
- Game loop must not block on network
- Use tokio::spawn for background fetches
- Channel-based communication to main loop
- Error states shown in console output

**Integration Pattern**:
```rust
// Background task fetches servers
let (tx, rx) = mpsc::channel();
tokio::spawn(async move {
    let result = fetch_servers(&master_url).await;
    tx.send(result).ok();
});
// Main loop polls rx for results
```

### 9. String Sanitization

**Decision**: Whitelist allowed characters + truncation

**Rationale**:
- Prevent display issues in console
- No HTML/ANSI injection possible in console
- Simple and predictable

**Implementation**:
- Allowed: alphanumeric, spaces, hyphens, underscores, dots
- Max lengths enforced (name: 64, region: 32, tag: 32)
- Truncate with ellipsis if needed for display

### 10. Protocol Version Compatibility

**Decision**: String-based version comparison (exact match for v1)

**Rationale**:
- Simple implementation for MVP
- Protocol version already exists in codebase
- Future: can add semver comparison if needed

**Implementation**:
- Client sends its protocol version in filter
- Master returns `protocol_version` field
- Client filters locally for compatibility

## Dependency Summary

### New Dependencies (workspace level)

```toml
# Cargo.toml workspace.dependencies
axum = "0.7"                    # Master server HTTP framework
reqwest = { version = "0.12", features = ["json"] }  # HTTP client
tower = "0.5"                   # Middleware (rate limiting)
tower-http = { version = "0.6", features = ["trace"] }  # HTTP tracing
```

### Crate Dependencies

**plix-master** (new):
- axum, tokio, serde, serde_json, tracing, tower, tower-http, clap

**plix-server** (additions):
- reqwest

**plix-client** (additions):
- reqwest

**plix-common** (no new dependencies):
- serde (already present)

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Master server becomes bottleneck | Low | Medium | In-memory design handles 1000+ servers easily |
| Network failures block game server | Low | High | Heartbeat is fire-and-forget, non-blocking |
| Rate limiting too aggressive | Medium | Low | Configurable limits, logging for tuning |
| Favorites file corruption | Low | Low | Fallback to empty list, warning logged |

## Open Questions Resolved

All technical decisions made. No outstanding unknowns blocking implementation.
