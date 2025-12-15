# Research: Logging & Metrics

**Feature**: 010-logging-metrics
**Date**: 2025-12-15

## Existing Infrastructure Analysis

### 1. Network Metrics (`plix-net/src/metrics.rs`)

**Decision**: Extend existing `NetworkMetrics` rather than replace

**Findings**:
- Already has `NetworkMetrics` struct with RTT samples, packet counting, loss tracking
- Uses `VecDeque<u32>` with `MAX_SAMPLES = 64` (sample-based, not time-based)
- Provides `rtt_avg()`, `jitter()` (std dev), `packet_loss_pct()`
- Has `rtt_min()`, `rtt_max()` methods

**Gap**:
- Current implementation is sample-count based (64 samples), not time-based (10s window)
- No p95 computation
- Need to add time-based windowing or increase sample count for 10s coverage

**Recommendation**:
- At 60Hz with ~1 RTT sample per tick, 600 samples = 10 seconds
- Add generic `RollingWindow<T>` in plix-common for reuse
- Keep existing `NetworkMetrics` API, add p95 method

### 2. Debug Overlay (`plix-client/src/ui/net_debug.rs`)

**Decision**: Implement the existing placeholder

**Findings**:
- `NetDebugOverlay` struct exists with `visible` flag and `toggle()` method
- `NetDebugData` struct has all needed fields: rtt, jitter, packet_loss, bytes_sent/recv, server_tick, client_tick
- `render()` is a TODO placeholder with comment showing intended format

**Gap**:
- render() not implemented
- F3 keybind not in config.rs Action enum

**Recommendation**:
- Add `ToggleDebugOverlay` to `Action` enum in config.rs
- Implement render using text rendering (check existing HUD patterns)

### 3. HUD Rendering (`plix-client/src/ui/hud.rs`)

**Decision**: Follow existing HUD patterns

**Findings**:
- `Hud` struct uses `HudData` for display state
- `render()` is also a placeholder (TODO: actual rendering by render engine)
- Event log pattern with timestamps for expiry
- Update method takes fps, ping_ms, health, etc.

**Gap**:
- Both HUD and overlay render() are placeholders
- Actual text rendering must be in render engine

**Recommendation**:
- Check render/engine.rs for actual text rendering
- Overlay should follow same pattern as HUD

### 4. Protocol Messages (`plix-common/src/protocol/messages.rs`)

**Decision**: Add `rtt_nonce` field to PlayerInput and echo in WorldSnapshot

**Findings**:
- `PlayerInput` has: seq, tick, move_forward, move_right, jump, crouch, attack, yaw, pitch
- `WorldSnapshot` has: tick, last_input_seq, players, match_state
- No existing RTT mechanism

**Gap**:
- Need to add `rtt_nonce: u64` to `PlayerInput`
- Need to add `rtt_nonce_echo: u64` to `WorldSnapshot`

**Recommendation**:
- Use client timestamp in milliseconds as nonce (simple, no collision)
- Echo in snapshot alongside `last_input_seq`

### 5. Tick Loop (`plix-server/src/tick.rs`)

**Decision**: Integrate metrics collection into existing loop

**Findings**:
- `TickLoop` already measures elapsed time per tick
- Returns `Duration` from `tick()` method
- Has `overrun_count` tracking
- Warns on tick overruns

**Gap**:
- No metrics collection/aggregation
- No periodic logging

**Recommendation**:
- Create `ServerMetricsCollector` that receives tick durations
- Add log output every 300 ticks (5s at 60Hz)

### 6. Config System (`plix-client/src/config.rs`)

**Decision**: Add ToggleDebugOverlay action

**Findings**:
- `Action` enum has 9 actions (Forward, Backward, Left, Right, Jump, Attack, PlaceBlock, RemoveBlock, Pause)
- `Key` enum supports F keys via `from_keycode` (but F3 mapping not explicit)
- Keybinds stored in HashMap<Action, Key>

**Gap**:
- No F key variants in Key enum (F1-F12 missing)
- No ToggleDebugOverlay action

**Recommendation**:
- Add `F3` variant to `Key` enum
- Add `ToggleDebugOverlay` to `Action` enum
- Default binding: F3 → ToggleDebugOverlay

### 7. Input Handling (`plix-client/src/input.rs`)

**Decision**: Handle F3 separately from gameplay input

**Findings**:
- `InputManager` handles WASD, Space, Ctrl, mouse buttons
- Uses local `Key` enum (W, A, S, D, Space, Ctrl, LeftClick, RightClick)
- Separate from config.rs Key enum

**Gap**:
- F3 should be handled at app level, not InputManager
- Need to check main.rs for key event handling

**Recommendation**:
- F3 toggle should be in event loop (main.rs), not InputManager
- Toggle overlay visibility directly

## Research Questions Resolved

### Q1: Rolling Window Implementation

**Decision**: Fixed-size array ring buffer with timestamp tracking

**Rationale**:
- VecDeque reallocates, ring buffer is O(1) push
- Store (timestamp, value) pairs for time-based expiry
- Pre-allocate 600 slots (10s at 60Hz)

**Alternatives Considered**:
- Sample-count based (current): Simple but doesn't guarantee time coverage
- Time-based VecDeque: Allocates, not suitable for hot path

### Q2: P95 Computation

**Decision**: Sort on read, cache result

**Rationale**:
- Computing p95 requires sorting
- Cache computed stats, invalidate on push
- Only recompute when stats requested and cache invalid

**Alternatives Considered**:
- Streaming quantiles (t-digest): Complex, overkill for 600 samples
- Always sort on read: Acceptable for 5s log intervals

### Q3: RTT Nonce Format

**Decision**: Use `Instant::now().elapsed().as_micros() as u64` as nonce

**Rationale**:
- Monotonic, no collisions
- Client stores HashMap<u64, Instant> for pending RTT measurements
- On echo, lookup and compute RTT

**Alternatives Considered**:
- Sequence number: Requires additional mapping
- Wall clock time: Not monotonic, can go backwards

### Q4: Headless Mode Compatibility

**Decision**: Conditional compilation not needed; overlay code path is safe

**Rationale**:
- `NetDebugOverlay::render()` returns early if `!self.visible`
- Server crate doesn't depend on client crate
- Headless runs server binary only

**Alternatives Considered**:
- `#[cfg(not(headless))]`: Unnecessary complexity
- Feature flags: Overkill for this feature

## Summary of Decisions

| Topic | Decision | Impact |
|-------|----------|--------|
| Rolling Window | Fixed-size ring buffer in plix-common | New file: metrics.rs |
| P95 | Sort on read, cache stats | Add method to RollingWindow |
| RTT Nonce | Microsecond timestamp as u64 | Extend PlayerInput, WorldSnapshot |
| Overlay | Implement existing placeholder | Update net_debug.rs |
| F3 Keybind | Add to Action + Key enums | Update config.rs |
| Server Metrics | New ServerMetricsCollector | New file: metrics.rs in plix-server |
| Log Interval | Every 300 ticks (5s) | Configurable constant |
