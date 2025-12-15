# Data Model: Logging & Metrics

**Feature**: 010-logging-metrics
**Date**: 2025-12-15

## Entities

### 1. RollingWindow<T> (plix-common)

Time-based fixed-size ring buffer for metric samples.

```rust
/// Fixed-size ring buffer for time-based metrics
pub struct RollingWindow<T> {
    /// Storage for samples with timestamps
    samples: Vec<(Instant, T)>,
    /// Current write position (ring buffer index)
    head: usize,
    /// Number of valid samples (up to capacity)
    len: usize,
    /// Window duration (samples older than this are excluded)
    window_duration: Duration,
    /// Cached statistics (invalidated on push)
    cached_stats: Option<Stats>,
}

impl<T> RollingWindow<T> {
    pub fn new(capacity: usize, window_duration: Duration) -> Self;
    pub fn push(&mut self, value: T);
    pub fn samples_in_window(&self) -> impl Iterator<Item = &T>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

**Constraints**:
- Capacity: 600 samples (10s at 60Hz)
- Window duration: 10 seconds
- No heap allocation in `push()` after initial construction

### 2. Stats (plix-common)

Statistical summary of rolling window samples.

```rust
/// Statistical summary
#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    /// Average value
    pub avg: f64,
    /// 95th percentile
    pub p95: f64,
    /// Maximum value
    pub max: f64,
    /// Minimum value
    pub min: f64,
    /// Sample count
    pub count: usize,
}

impl RollingWindow<f64> {
    pub fn stats(&mut self) -> Stats;  // Computes and caches
}

impl RollingWindow<Duration> {
    pub fn stats_ms(&mut self) -> Stats;  // Converts to milliseconds
}
```

### 3. ServerMetricsCollector (plix-server)

Collects and logs server-wide metrics.

```rust
/// Server metrics collector
pub struct ServerMetricsCollector {
    /// Tick time rolling window
    tick_times: RollingWindow<Duration>,
    /// Last log timestamp
    last_log: Instant,
    /// Log interval (default: 5 seconds)
    log_interval: Duration,
    /// Total ticks processed
    total_ticks: u64,
}

impl ServerMetricsCollector {
    pub fn new() -> Self;
    pub fn record_tick(&mut self, duration: Duration);
    pub fn maybe_log(&mut self, players_connected: usize, sessions_active: usize);
}
```

**Log Format** (structured via tracing):
```
INFO server_metrics: tick_ms_avg=2.5 tick_ms_p95=4.1 tick_ms_max=8.2 players=4 sessions=1
```

### 4. SessionNetMetrics (plix-server)

Per-connection network quality metrics.

```rust
/// Per-session network metrics
pub struct SessionNetMetrics {
    /// RTT rolling window (from ack timing)
    rtt_window: RollingWindow<Duration>,
    /// Last received sequence number (for loss detection)
    last_seq: u32,
    /// Expected next sequence number
    expected_seq: u32,
    /// Packets received in window
    packets_received: u32,
    /// Packets lost (gaps) in window
    packets_lost: u32,
    /// Bytes received since last reset
    bytes_in: u64,
    /// Bytes sent since last reset
    bytes_out: u64,
    /// Last reset timestamp (for PPS calculation)
    last_reset: Instant,
}

impl SessionNetMetrics {
    pub fn new() -> Self;
    pub fn record_rtt(&mut self, rtt: Duration);
    pub fn record_packet(&mut self, seq: u32, bytes: usize);
    pub fn record_send(&mut self, bytes: usize);
    pub fn jitter(&mut self) -> Duration;  // std dev of RTT
    pub fn loss_pct(&self) -> f32;
    pub fn pps_in(&self) -> f32;
    pub fn pps_out(&self) -> f32;
}
```

### 5. Protocol Extensions (plix-common)

Extensions to existing protocol messages.

```rust
// In PlayerInput (existing struct)
pub struct PlayerInput {
    pub seq: InputSeq,
    pub tick: Tick,
    pub move_forward: f32,
    pub move_right: f32,
    pub jump: bool,
    pub crouch: bool,
    pub attack: bool,
    pub yaw: f32,
    pub pitch: f32,
    // NEW: RTT measurement nonce (client timestamp in microseconds)
    pub rtt_nonce: u64,
}

// In WorldSnapshot (existing struct)
pub struct WorldSnapshot {
    pub tick: Tick,
    pub last_input_seq: InputSeq,
    pub players: Vec<PlayerSnapshot>,
    pub match_state: MatchState,
    // NEW: Echo of client's rtt_nonce for RTT calculation
    pub rtt_nonce_echo: u64,
}
```

### 6. ClientOverlayState (plix-client)

Extended overlay state for F3 debug display.

```rust
// Already exists in net_debug.rs, minor extensions
pub struct NetDebugOverlay {
    pub visible: bool,
    pub data: NetDebugData,
    // NEW: Last update timestamp for 2Hz refresh
    last_update: Instant,
    // NEW: Cached text lines for rendering
    cached_lines: Vec<String>,
}

pub struct NetDebugData {
    pub rtt: Duration,
    pub jitter: Duration,
    pub packet_loss: f32,
    pub pending_inputs: usize,
    pub bytes_sent_per_sec: u64,
    pub bytes_recv_per_sec: u64,
    pub server_tick: u32,
    pub client_tick: u32,
    pub tick_offset: i32,
    // NEW: FPS for overlay display
    pub fps: u32,
    // NEW: Player ID
    pub player_id: Option<PlayerId>,
}
```

### 7. Config Extensions (plix-client)

Keybind additions for overlay toggle.

```rust
// In config.rs Action enum
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Attack,
    PlaceBlock,
    RemoveBlock,
    Pause,
    // NEW
    ToggleDebugOverlay,
}

// In config.rs Key enum
pub enum Key {
    // ... existing keys ...
    // NEW: Function keys for debug
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}
```

## Relationships

```
┌─────────────────┐     ┌──────────────────┐
│ ServerMetrics   │────▶│ RollingWindow    │
│ Collector       │     │ <Duration>       │
└─────────────────┘     └──────────────────┘
                              │
                              ▼
                        ┌──────────┐
                        │  Stats   │
                        └──────────┘

┌─────────────────┐     ┌──────────────────┐
│ SessionNet      │────▶│ RollingWindow    │
│ Metrics         │     │ <Duration>       │
└─────────────────┘     └──────────────────┘

┌─────────────────┐     ┌──────────────────┐
│ PlayerInput     │────▶│ rtt_nonce (u64)  │
└─────────────────┘     └──────────────────┘
        │                       │
        ▼                       ▼
┌─────────────────┐     ┌──────────────────┐
│ WorldSnapshot   │────▶│ rtt_nonce_echo   │
└─────────────────┘     └──────────────────┘
        │
        ▼
┌─────────────────┐
│ Client RTT      │
│ Calculation     │
└─────────────────┘
```

## State Transitions

### RTT Measurement Flow

```
1. Client generates input
   └── Set rtt_nonce = current_time_micros()
   └── Store (nonce, Instant::now()) in pending map

2. Server receives input
   └── Store latest rtt_nonce per client

3. Server sends snapshot
   └── Include rtt_nonce_echo = stored nonce

4. Client receives snapshot
   └── Lookup nonce in pending map
   └── RTT = Instant::now() - stored_instant
   └── Push RTT to metrics window
   └── Remove nonce from pending map
```

### Metrics Logging Flow

```
Every tick:
1. Tick executes
2. Record tick_duration in RollingWindow
3. Check if log_interval elapsed
4. If yes:
   └── Compute stats()
   └── Log structured output
   └── Reset log timer
```

## Validation Rules

| Field | Rule | Error |
|-------|------|-------|
| RollingWindow capacity | >= 1 | Panic on construction |
| Stats.count | >= 1 for valid stats | Return default if empty |
| rtt_nonce | Non-zero | Zero means no RTT tracking |
| packet_loss_pct | 0.0 - 100.0 | Clamp to range |
| log_interval | >= 1 second | Clamp minimum |
