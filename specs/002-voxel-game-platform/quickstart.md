# Quickstart: Voxel Game Platform - Visual Multiplayer

**Feature**: 002-voxel-game-platform
**Date**: 2025-12-14

## Prerequisites

- Rust 1.75+ (stable)
- Linux with Vulkan support (or other wgpu-compatible platform)
- Terminal access

## Build

```bash
# Build all crates
cargo build --workspace

# Build release (for better performance)
cargo build --workspace --release
```

## Run Server

```bash
# Default: port 7777, tick rate 60 Hz
cargo run --bin plix-server

# Custom port
cargo run --bin plix-server -- --port 8888

# With debug logging
RUST_LOG=debug cargo run --bin plix-server
```

## Run Client (Windowed)

```bash
# Connect to local server
cargo run --bin plix-client

# Connect to specific server
cargo run --bin plix-client -- --server 192.168.1.100:7777

# With player name
cargo run --bin plix-client -- --name "Player1"
```

## Run Client (Headless)

```bash
# For testing without graphics
cargo run --bin plix-client -- --headless --server 127.0.0.1:7777
```

## Multi-Client Test

Open 3 terminals:

```bash
# Terminal 1: Server
cargo run --bin plix-server

# Terminal 2: Client 1
cargo run --bin plix-client -- --name "Alice"

# Terminal 3: Client 2
cargo run --bin plix-client -- --name "Bob"
```

## Controls

| Key | Action |
|-----|--------|
| W/A/S/D | Move forward/left/back/right |
| Mouse | Look around |
| Space | Jump |
| Ctrl | Crouch |
| ESC | Release mouse cursor |

## Expected Behavior

After completing this feature:

1. **Server starts**: Loads arena, waits for connections
2. **Client connects**: Window opens, arena visible
3. **Arena rendered**: Blocks visible with distinct colors
4. **Players visible**: Capsules represent players
5. **HUD active**: Window title shows FPS, ping, player ID, round state
6. **Movement smooth**: Remote players interpolate smoothly

## Verification Checklist

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] Server starts without errors
- [ ] Client window opens with arena visible
- [ ] Two clients see each other
- [ ] Remote player movement is smooth (no teleporting)
- [ ] HUD displays: FPS > 30, Ping < 100ms, Player ID, Round state
- [ ] Headless mode still works
- [ ] Load test script runs successfully

## Troubleshooting

### "No suitable GPU adapter found"
- Ensure Vulkan/Metal/DX12 drivers are installed
- Try running with `WGPU_BACKEND=vulkan` or `WGPU_BACKEND=gl`

### "Connection refused"
- Verify server is running
- Check port is correct (default 7777)
- Firewall may be blocking UDP

### Low FPS
- Use release build: `cargo run --release`
- Check GPU utilization
- Arena may be too large (test arena is 32x16x32)

### "Disconnected" in title
- Server may have crashed
- Network timeout (check firewall)
- Protocol version mismatch

## Development

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p plix-client

# With output
cargo test --workspace -- --nocapture
```

### Formatting & Linting

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace

# Both (CI requirement)
cargo fmt --all -- --check && cargo clippy --workspace
```

### Load Testing

```bash
# Run load test with bots
./scripts/run_load_test.sh

# Or manually
cargo run --bin plix-tools -- load-test --bots 8 --server 127.0.0.1:7777
```
