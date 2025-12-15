# Plix MVP v0.1 - Quickstart Guide

## Prerequisites

- **Rust**: 1.75+ (stable channel)
- **Git**: For cloning the repository
- **OS**: Windows 10+, Linux (Ubuntu 22.04+), or macOS 12+

## Build from Source

```bash
# Clone repository
git clone https://github.com/your-org/plix.git
cd plix

# Build all crates (debug mode)
cargo build

# Build release binaries
cargo build --release
```

## Run Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p plix-net
cargo test -p plix-server

# Run with logging
RUST_LOG=debug cargo test
```

## Start a Server

```bash
# Default settings (port 7777, 60 tick, test_arena)
cargo run --release -p plix-server

# Custom settings
cargo run --release -p plix-server -- \
  --port 7778 \
  --tickrate 60 \
  --max-players 16 \
  --arena test_arena
```

**Server CLI Options**:

| Flag | Default | Description |
|------|---------|-------------|
| `--port` | 7777 | UDP port to listen on |
| `--tickrate` | 60 | Server tick rate (20-60) |
| `--max-players` | 16 | Maximum concurrent players |
| `--arena` | test_arena | Arena name (from assets/arenas/) |
| `--log-level` | info | Log verbosity (trace/debug/info/warn/error) |

## Start a Client

```bash
# Launch client
cargo run --release -p plix-client
```

In the client:
1. Enter server IP:PORT (e.g., `127.0.0.1:7777`)
2. Enter your player name
3. Click "Connect"

**Client Controls**:

| Key | Action |
|-----|--------|
| W/A/S/D | Move |
| Space | Jump |
| Shift | Sprint |
| Ctrl | Crouch |
| Mouse | Look |
| Left Click | Attack |
| Escape | Pause/Disconnect |
| F1 | Toggle debug HUD |

## Local Multiplayer Test

Terminal 1 (Server):
```bash
cargo run --release -p plix-server
```

Terminal 2 (Client 1):
```bash
cargo run --release -p plix-client
# Connect to 127.0.0.1:7777
```

Terminal 3 (Client 2):
```bash
cargo run --release -p plix-client
# Connect to 127.0.0.1:7777
```

## Bot Stress Test

```bash
# Spawn 8 bot clients for testing
cargo run --release -p plix-tools -- bot \
  --server 127.0.0.1:7777 \
  --count 8 \
  --duration 60
```

## Network Simulation

Test with artificial latency and packet loss:

```bash
# Start server with network simulation
cargo run --release -p plix-tools -- net-sim \
  --latency 100 \
  --jitter 20 \
  --loss 5
```

## Creating Arenas

Arenas are defined in `assets/arenas/` as TOML files:

```toml
# assets/arenas/my_arena.toml
[metadata]
name = "My Custom Arena"
version = "0.1.0"
size = [64, 32, 64]

[[spawn_points]]
team = 0
position = [10, 5, 10]
rotation = 0.0

[[spawn_points]]
team = 1
position = [54, 5, 54]
rotation = 180.0

[blocks]
floor = { y = 0, block = "stone" }
walls = { border = true, height = 10, block = "brick" }
```

Validate arena:
```bash
cargo run --release -p plix-tools -- validate-arena my_arena
```

## Logs

**Server logs** (structured JSON in production):
```bash
# Human-readable dev logs
RUST_LOG=plix=debug cargo run -p plix-server

# JSON production logs
cargo run --release -p plix-server 2>&1 | jq
```

**Log locations**:
- Server: stdout (no file by default)
- Client: stdout + `~/.plix/logs/client.log`

## Performance Monitoring

The server exposes metrics in logs:
- `tick_time_ms`: Time to process each tick
- `tps`: Actual ticks per second
- `player_count`: Connected players
- `rtt_avg_ms`: Average round-trip time per player
- `packet_loss_pct`: Estimated packet loss

Example log line:
```json
{"level":"INFO","tick":1234,"tick_time_ms":2.3,"tps":60,"player_count":8}
```

## Troubleshooting

### "Connection refused"
- Verify server is running
- Check firewall allows UDP on port 7777
- Verify correct IP address

### "Protocol mismatch"
- Client and server must be same version
- Rebuild both from same commit

### High ping / lag
- Check network simulation is disabled
- Verify tickrate matches on both ends
- Run `ping` to check base latency

### Client crashes on connect
- Check RUST_LOG=debug for details
- Verify arena file exists and is valid

## Next Steps

After verifying basic functionality:
1. Read [architecture.md](../docs/architecture.md) for system design
2. Read [protocol.md](../docs/protocol.md) for network details
3. Run the full test suite: `cargo test`
4. Try the bot stress test with 16 clients
