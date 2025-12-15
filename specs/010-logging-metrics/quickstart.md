# Quickstart: Logging & Metrics

**Feature**: 010-logging-metrics
**Date**: 2025-12-15

## Overview

This feature adds observability to Plix:
- **Server**: Tick time metrics (avg/p95/max), player counts, logged every 5 seconds
- **Per-Session**: RTT, jitter, packet loss tracked per connection
- **Client**: F3 debug overlay showing network stats

## Quick Test

### 1. Start Server

```bash
cd /home/louis/Documents/plix
cargo run --release -p plix-server
```

Expected log output every 5 seconds:
```
INFO server_metrics: tick_ms_avg=2.5 tick_ms_p95=4.1 tick_ms_max=8.2 players=0 sessions=0
```

### 2. Connect Client

```bash
cargo run --release -p plix-client
```

### 3. Toggle Debug Overlay

Press **F3** to show/hide the debug overlay.

Expected overlay content:
```
FPS: 60 | RTT: 45ms | Jitter: 5ms | Loss: 0.1%
Tick: S:12345 C:12348 (+3) | Player: 1
```

### 4. Verify Metrics Under Load

Run load test:
```bash
./scripts/run_load_test.sh 8 30 127.0.0.1:7777
```

Monitor server logs for:
- tick_ms_avg staying below 16.67ms (60Hz budget)
- tick_ms_p95 staying reasonable
- No tick overrun warnings

## Log Format Reference

### Server Metrics (every 5s)

```
INFO server_metrics: tick_ms_avg=<f64> tick_ms_p95=<f64> tick_ms_max=<f64> players=<usize> sessions=<usize>
```

| Field | Description | Unit |
|-------|-------------|------|
| tick_ms_avg | Average tick time over 10s window | milliseconds |
| tick_ms_p95 | 95th percentile tick time | milliseconds |
| tick_ms_max | Maximum tick time in window | milliseconds |
| players | Currently connected players | count |
| sessions | Active match sessions | count |

### Per-Session Debug (optional, DEBUG level)

```
DEBUG session_metrics player_id=<u32>: rtt_ms=<f64> jitter_ms=<f64> loss_pct=<f32> pps_in=<f32> pps_out=<f32>
```

## Keybinds

| Key | Action |
|-----|--------|
| F3 | Toggle debug overlay |

Default binding. Can be rebound in config.toml:

```toml
[keybinds]
ToggleDebugOverlay = "F3"
```

## Troubleshooting

### Overlay Not Appearing

1. Ensure you're in-game (connected to server)
2. Press F3 (check keybinds if rebound)
3. Check console for any rendering errors

### High Tick Times

If `tick_ms_avg` exceeds 16.67ms:
1. Check player count (more players = more work)
2. Check for tick overrun warnings in logs
3. Profile server with `perf` or similar

### RTT Shows 0 or N/A

1. Ensure client is connected and sending inputs
2. RTT requires input packets to flow
3. Check network connectivity

## Verification Checklist

- [ ] Server starts and logs metrics every 5 seconds
- [ ] Client connects and F3 shows overlay
- [ ] Overlay updates at 2Hz (every 500ms)
- [ ] RTT/jitter/loss values are reasonable
- [ ] Load test passes without performance regression
- [ ] Headless server works (no client code execution)
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy` clean
- [ ] `cargo fmt --check` clean

## Files Changed

| File | Change |
|------|--------|
| `crates/plix-common/src/metrics.rs` | NEW: RollingWindow, Stats |
| `crates/plix-common/src/protocol/messages.rs` | Add rtt_nonce fields |
| `crates/plix-server/src/metrics.rs` | NEW: ServerMetricsCollector |
| `crates/plix-server/src/tick.rs` | Integrate metrics recording |
| `crates/plix-server/src/session.rs` | Add SessionNetMetrics |
| `crates/plix-client/src/config.rs` | Add F3 keybind, ToggleDebugOverlay |
| `crates/plix-client/src/ui/net_debug.rs` | Implement render() |
