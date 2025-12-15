# Plix - Multiplayer Voxel Game Platform

Plix is a multiplayer voxel game platform built in Rust with an authoritative server architecture supporting 8-16 players per match.

## Prerequisites

- Rust 1.75+ (stable)
- Cargo

## Quick Start

### Build

```bash
cargo build --release
```

### Run Server

```bash
cargo run --release --bin plix-server -- --port 7777 --arena test_arena
```

Options:
- `--port <PORT>`: UDP port (default: 7777)
- `--tickrate <RATE>`: Server tick rate 20-60 Hz (default: 60)
- `--max-players <N>`: Max players (default: 16)
- `--arena <NAME>`: Arena name from assets/arenas/ (default: test_arena)
- `--log-level <LEVEL>`: Log level: trace, debug, info, warn, error (default: info)

### Run Client

```bash
cargo run --release --bin plix-client -- --server 127.0.0.1:7777 --name Player1
```

Options:
- `--server <IP:PORT>`: Server address
- `--name <NAME>`: Player name
- `--headless`: Run without graphics (for testing)
- `--log-level <LEVEL>`: Log level

### Run Tests

```bash
cargo test --workspace
```

### Load Testing

```bash
# Run 8 bots for 60 seconds
./scripts/run_load_test.sh 8 60 127.0.0.1:7777

# Or directly with plix-tools
cargo run --release --bin plix-tools -- bot --server 127.0.0.1:7777 --count 8 --duration 60
```

## Project Structure

```
crates/
  plix-common/   # Shared types, protocol, math
  plix-net/      # UDP transport, reliable channels
  plix-server/   # Authoritative game server
  plix-client/   # Game client with prediction
  plix-arena/    # Arena loading and validation
  plix-tools/    # Bot client and load testing
```

## Architecture

- **Authoritative Server**: All game state runs on server
- **Client Prediction**: Local player movement predicted client-side
- **Server Reconciliation**: Corrections applied on misprediction
- **Remote Interpolation**: Other players smoothly interpolated
- **60 Hz Tick Rate**: Server runs at 60 ticks/second

## Documentation

- [Architecture](docs/architecture.md) - System architecture
- [Protocol](docs/protocol.md) - Network protocol specification
- [Testing](docs/testing.md) - Test procedures

## License

See LICENSE file for details.
