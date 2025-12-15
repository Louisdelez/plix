# Plix Testing Guide

## Running Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test --package plix-server
cargo test --package plix-client
cargo test --package plix-common
cargo test --package plix-net
cargo test --package plix-arena
cargo test --package plix-tools
```

### Specific Test

```bash
cargo test --package plix-server test_combat_hit_server_validated
```

## Test Categories

### Unit Tests

Located in `src/` files with `#[cfg(test)]` modules.

| Crate | Tests | Coverage |
|-------|-------|----------|
| plix-common | 12 | Types, protocol codec, math |
| plix-net | 18 | Transport, channels, connection |
| plix-server | 22 | Session, simulation, validation |
| plix-client | 1 | Command buffer |
| plix-arena | 10 | Loader, validation, spawns |
| plix-tools | 3 | Network simulator |

### Integration Tests

Located in `tests/` directories.

| File | Purpose |
|------|---------|
| `plix-server/tests/movement_test.rs` | Two players movement visibility |
| `plix-server/tests/combat_test.rs` | Server-validated combat hits |

### Load Tests

Require a running server. Run with `--ignored` flag.

```bash
# Start server first
cargo run --release --bin plix-server &

# Run load tests
cargo test --package plix-tools --test load_test -- --ignored
cargo test --package plix-tools --test stability_test -- --ignored
```

| Test | Description |
|------|-------------|
| `test_8_bots_60_seconds` | 8 bots for 60 seconds |
| `test_16_bots_30_seconds` | 16 bots for 30 seconds |
| `test_2_bots_smoke` | Quick 2 bot smoke test |
| `test_stability_8_bots` | Verify low packet loss |
| `test_no_connection_drops` | Sequential connection test |
| `test_sustained_performance` | 2 minute endurance test |

## Manual Testing

### Connection Test

1. Start server:
   ```bash
   cargo run --bin plix-server -- --bind 127.0.0.1:7777
   ```

2. Start client:
   ```bash
   cargo run --bin plix-client -- --server 127.0.0.1:7777 --name Test
   ```

3. Verify connection message in server logs.

### Headless Client Test

```bash
cargo run --bin plix-client -- --server 127.0.0.1:7777 --name Bot --headless
```

### Multi-Client Test

Terminal 1 (Server):
```bash
cargo run --bin plix-server
```

Terminal 2 (Client 1):
```bash
cargo run --bin plix-client -- --server 127.0.0.1:7777 --name Player1
```

Terminal 3 (Client 2):
```bash
cargo run --bin plix-client -- --server 127.0.0.1:7777 --name Player2
```

### Load Test Script

```bash
./scripts/run_load_test.sh 8 60 127.0.0.1:7777
```

Arguments:
- Bots: Number of bots (default: 8)
- Duration: Seconds (default: 60)
- Server: Address (default: 127.0.0.1:7777)

## Test Assertions

### Movement Tests

- Players can move in all directions
- Position updates are visible to other players
- Movement speed is within limits

### Combat Tests

- Attacks hit targets in range
- Attacks miss targets out of range
- Attack cooldown is enforced
- Damage and death work correctly

### Network Tests

- Packets are delivered reliably when needed
- Duplicate packets are detected
- Connection state machine works
- RTT is measured correctly

### Stability Tests

- 60 Hz tick rate maintained
- No crashes under load
- Packet loss within acceptable limits
- Memory usage stable

## CI Pipeline

Tests run automatically on push:

1. Format check (`cargo fmt --check`)
2. Lint (`cargo clippy`)
3. Build (`cargo build`)
4. Test (`cargo test --workspace`)

Platforms tested:
- Linux (ubuntu-latest)
- macOS (macos-latest)
- Windows (windows-latest)
