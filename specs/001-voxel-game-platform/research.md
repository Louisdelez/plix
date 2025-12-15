# Research: Plix MVP v0.1 Technical Decisions

**Date**: 2025-12-14
**Status**: Complete

## 1. Network Transport Layer

### Decision: Custom UDP with reliability layer

**Rationale**:
- Full control over packet format and reliability semantics
- No external game networking crate dependency (reduces risk of abandonment)
- Tailored to authoritative server model with snapshot-based replication
- Constitution requires no proprietary dependencies

**Alternatives Considered**:
- **laminar**: Lightweight but limited maintenance activity
- **quinn (QUIC)**: More complex, adds TLS overhead unnecessary for LAN/direct play
- **enet-rs**: C bindings, less idiomatic Rust
- **TCP**: Too high latency for real-time game state

**Implementation Approach**:
```
Channels:
- Unreliable: Player inputs, snapshots (latest-only matters)
- Reliable-unordered: Events (hit, death, score), connection management
- Reliable-ordered: Critical state (round start/end, kick)

Packet structure:
[Header: 4 bytes] [Sequence: 2 bytes] [Ack: 2 bytes] [AckBits: 4 bytes] [Payload]

Reliability:
- Sequence numbers for packet ordering
- Selective ACK bitmap for efficient acknowledgment
- Resend on timeout (RTT * 1.5)
```

---

## 2. Server Tick Architecture

### Decision: Fixed tick at 60 Hz (configurable 20-60)

**Rationale**:
- 60 Hz provides smooth gameplay for competitive PvP
- Fixed tick ensures deterministic simulation
- Constitution requires tick stability without GC pauses
- Configurable allows lower rates for testing or less competitive modes

**Alternatives Considered**:
- **Variable tick**: Harder to reproduce bugs, non-deterministic
- **30 Hz fixed**: Acceptable for casual, but PvP needs 60 Hz
- **120 Hz**: Diminishing returns, higher CPU cost

**Implementation Approach**:
```rust
// Server tick loop (simplified)
let tick_duration = Duration::from_secs_f64(1.0 / 60.0);
loop {
    let tick_start = Instant::now();

    process_incoming_packets();
    apply_player_inputs();
    run_simulation_step();    // Physics, combat
    generate_snapshots();
    send_outgoing_packets();

    let elapsed = tick_start.elapsed();
    if elapsed < tick_duration {
        sleep(tick_duration - elapsed);
    } else {
        warn!("Tick overrun: {:?}", elapsed);
    }
}
```

---

## 3. Client-Side Prediction & Reconciliation

### Decision: Input prediction with server reconciliation

**Rationale**:
- Player feels responsive despite network latency
- Server remains authoritative (constitution requirement)
- Standard approach for competitive FPS games

**Alternatives Considered**:
- **No prediction**: Unplayable above 50ms latency
- **Full client authority with validation**: Opens cheat vectors
- **Lockstep**: Too restrictive for player count, latency-bound

**Implementation Approach**:
```
Client flow:
1. Capture input, apply locally (predicted state)
2. Send input to server with sequence number
3. Store input in pending buffer
4. Receive server snapshot with last-processed input sequence
5. If mismatch: rollback to server state, replay pending inputs

Correction smoothing:
- Small corrections: interpolate over 100ms
- Large corrections (cheater/lag): snap immediately
```

---

## 4. Snapshot Replication

### Decision: Full snapshots at 20-30 Hz with delta encoding

**Rationale**:
- 20-30 Hz is sufficient for interpolation (client renders at 60+ FPS)
- Delta encoding reduces bandwidth
- Simpler than event-sourcing for MVP

**Alternatives Considered**:
- **Event-sourcing only**: Complex rollback, harder debugging
- **60 Hz snapshots**: Bandwidth intensive, unnecessary
- **No delta**: ~3x bandwidth cost

**Implementation Approach**:
```
Snapshot contents (MVP):
- Player states: position, rotation, health, animation state
- Match state: round time, scores
- (Future: block changes, entities)

Delta encoding:
- Track last acknowledged snapshot per client
- Send only changed fields
- Fallback to full snapshot on sequence gap
```

---

## 5. Combat System (MVP)

### Decision: Melee-only with server-authoritative hit detection

**Rationale**:
- Melee is simpler to validate server-side (no projectile prediction)
- Reduces "I shot first" disputes
- MVP focus is network, not combat depth
- Can add projectiles post-MVP

**Alternatives Considered**:
- **Projectiles**: Requires prediction, more complex validation
- **Hitscan**: Lag compensation complexity
- **Client-authoritative**: Constitution violation (anti-cheat)

**Implementation Approach**:
```
Attack flow:
1. Client sends "attack" input
2. Server validates: cooldown, range, line-of-sight
3. Server applies damage if valid
4. Server sends "hit" event to attacker (feedback)
5. Server sends "damaged" event to target
6. Clients play effects based on events

Validation rules:
- Attack range: 2 blocks
- Cooldown: 500ms
- Damage: 20 HP (5 hits to kill from 100 HP)
```

---

## 6. Voxel Rendering (MVP)

### Decision: Simple greedy meshing with wgpu

**Rationale**:
- wgpu is cross-platform, modern, Rust-native
- Greedy meshing reduces triangle count significantly
- MVP arenas are small (< 100x100x50 blocks)
- Can optimize later

**Alternatives Considered**:
- **OpenGL via glow**: Legacy, less maintained
- **Vulkan direct**: Too low-level for MVP
- **rend3 / bevy_render**: Adds dependencies, less control

**Implementation Approach**:
```
Rendering pipeline:
1. Load arena blocks into chunk data (16x16x16)
2. Generate mesh per chunk (greedy meshing)
3. Upload mesh to GPU
4. Render with simple diffuse shading
5. Overlay HUD as 2D layer

MVP simplifications:
- No lighting (flat shading)
- No shadows
- Single texture atlas
- No transparency (solid blocks only)
```

---

## 7. Arena Format

### Decision: TOML-based arena definition files

**Rationale**:
- Human-readable and editable
- Rust has excellent TOML support (serde)
- Easy to version control
- Can generate programmatically

**Alternatives Considered**:
- **JSON**: More verbose, less readable
- **Binary (custom)**: Not human-editable
- **NBT**: Minecraft format, unnecessary compatibility

**Implementation Approach**:
```toml
# assets/arenas/test_arena.toml
[metadata]
name = "Test Arena"
version = "0.1.0"
size = [64, 32, 64]  # x, y, z

[[spawn_points]]
team = 0
position = [10, 5, 10]
rotation = 0.0

[[spawn_points]]
team = 1
position = [54, 5, 54]
rotation = 180.0

[blocks]
# Simple RLE or region-based encoding
floor = { y = 0, block = "stone" }
walls = { border = true, height = 10, block = "brick" }
```

---

## 8. Logging & Observability

### Decision: tracing crate with structured JSON output

**Rationale**:
- Industry standard for Rust structured logging
- Supports spans (tick timing, connection lifecycle)
- Can filter by level at runtime
- JSON output for production, pretty output for dev

**Alternatives Considered**:
- **log + env_logger**: Less structured, no spans
- **slog**: More complex setup
- **Custom**: Unnecessary reinvention

**Implementation Approach**:
```rust
// Server initialization
tracing_subscriber::fmt()
    .with_env_filter("plix=debug,plix_net=trace")
    .json()  // or .pretty() for dev
    .init();

// Usage
#[instrument(skip(self))]
fn process_tick(&mut self) {
    let _span = info_span!("tick", number = self.tick_count);
    // ...
    info!(players = self.players.len(), "Tick complete");
}
```

---

## 9. Cross-Platform Builds

### Decision: GitHub Actions CI with matrix builds

**Rationale**:
- Free for open source
- Native runners for Windows, Linux, macOS
- Constitution requires reproducible builds

**Alternatives Considered**:
- **GitLab CI**: Fewer native runners
- **Local only**: No CI = no reproducibility guarantee

**Implementation Approach**:
```yaml
# .github/workflows/ci.yml
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release
```

---

## 10. Anti-Cheat (MVP Baseline)

### Decision: Server-side validation only, no client integrity checks

**Rationale**:
- Server-authoritative is the primary defense
- Client integrity checks are arms race, MVP doesn't need
- Focus on what server can validate

**Alternatives Considered**:
- **Client checksums**: Easily bypassed, false sense of security
- **Third-party anti-cheat**: Proprietary dependency violation

**Implementation Approach**:
```
Server validates:
- Movement speed: max 10 blocks/sec
- Attack cooldown: min 500ms between attacks
- Attack range: max 2 blocks
- Position consistency: no teleportation > 5 blocks/tick

On violation:
- Log with player ID and details
- Minor: Correct state silently
- Major/repeated: Kick with reason
```

---

## Summary Table

| Topic | Decision | Key Rationale |
|-------|----------|---------------|
| Transport | Custom UDP + reliability | Control, no dependencies |
| Tick | 60 Hz fixed | Competitive PvP, determinism |
| Prediction | Client predict + server reconcile | Responsiveness + authority |
| Replication | Snapshots 20-30 Hz + delta | Bandwidth efficient |
| Combat | Melee only, server-authoritative | Simple validation |
| Rendering | wgpu + greedy meshing | Cross-platform, performant |
| Arena format | TOML files | Human-readable |
| Logging | tracing + JSON | Structured, spans |
| CI | GitHub Actions matrix | Multi-platform builds |
| Anti-cheat | Server validation only | Constitution-compliant |
