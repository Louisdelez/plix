# Quickstart Guide: Server Browser v1

**Feature**: 026-server-browser
**Date**: 2025-12-17

## Overview

This guide covers development setup and testing for the server browser feature.

## Prerequisites

- Rust 1.75+ (stable)
- Cargo
- A terminal with multiple tabs/panes (or tmux)

## Building

```bash
# Build all crates (including new plix-master)
cargo build --workspace

# Build specific components
cargo build -p plix-master
cargo build -p plix-server
cargo build -p plix-client
```

## Running Locally

### 1. Start Master Server

```bash
# Terminal 1: Master server
cargo run -p plix-master -- --port 8080

# With verbose logging
RUST_LOG=debug cargo run -p plix-master -- --port 8080
```

Expected output:
```
Master server listening on 0.0.0.0:8080
```

### 2. Start Game Server (with master announcement)

```bash
# Terminal 2: Game server announcing to master
cargo run -p plix-server -- \
  --master-url http://localhost:8080 \
  --server-name "Local Test Server" \
  --region "local" \
  --tags "test,dev"

# Or using environment variables
PLIX_MASTER_URL=http://localhost:8080 \
PLIX_SERVER_NAME="Local Test Server" \
PLIX_REGION="local" \
cargo run -p plix-server
```

Expected output:
```
Server starting on 0.0.0.0:7777
Registered with master server (id: a1b2c3d4...)
Heartbeat sent successfully
```

### 3. Start Client and Browse

```bash
# Terminal 3: Client
cargo run -p plix-client -- --master-url http://localhost:8080
```

In the client console:
```
> /servers
Server List (1 servers):
[1] Local Test Server (0/32) [local] [test, dev] ping: 1ms

> /connect 1
Connecting to Local Test Server (127.0.0.1:7777)...
Connected!
```

## Testing Commands

### Server Browser Commands

```bash
# List all servers
/servers

# Search by name/tag/region
/servers local
/servers ctf

# Filter by players
/servers --players

# Filter by compatible version
/servers --compatible

# Sort options
/servers --sort=players
/servers --sort=recent

# Connect to server
/connect 1

# Manage favorites
/favorite 1
/favorites
/unfavorite 1
```

## Running Tests

```bash
# All tests
cargo test --workspace

# Master server tests
cargo test -p plix-master

# Server browser client tests
cargo test -p plix-client -- server_browser

# Integration tests
cargo test -p plix-master -- --test integration
```

## Test Scenarios

### Scenario 1: Basic Server Discovery

1. Start master server
2. Start game server with master URL
3. Verify heartbeat logs on master
4. Start client
5. Run `/servers` - should see game server
6. Run `/connect 1` - should connect

### Scenario 2: Server Expiration

1. Start master and game server
2. Verify server appears in list
3. Kill game server (Ctrl+C)
4. Wait 60+ seconds
5. Run `/servers` - server should be gone

### Scenario 3: Rate Limiting

```bash
# Test rate limiting with curl
for i in {1..15}; do
  curl -X POST http://localhost:8080/heartbeat \
    -H "Content-Type: application/json" \
    -d '{"name":"Test'$i'","host":"127.0.0.1","port":'$((7777+i))',"region":"test","tags":[],"player_count":0,"max_players":32,"game_modes":[],"protocol_version":"0.1.0"}'
  echo ""
done
# Requests 11-15 should return 429
```

### Scenario 4: Favorites Persistence

1. Start master, server, client
2. `/servers` then `/favorite 1`
3. Exit client
4. Check `~/.config/plix/servers.toml` - should contain favorite
5. Restart client
6. `/favorites` - should show saved favorite

### Scenario 5: Search and Filter

1. Start master
2. Start multiple game servers with different names/tags
3. Test search: `/servers ctf`
4. Test filter: `/servers --players`
5. Verify correct filtering

## Configuration Files

### servers.toml (Client Favorites)

Location: `~/.config/plix/servers.toml`

```toml
[[favorites]]
server_id = "a1b2c3d4e5f67890"
name = "Local Test Server"
host = "127.0.0.1"
port = 7777
added_at = 1702828800

[settings]
master_url = "http://localhost:8080"
```

### Server Configuration (Game Server)

Via CLI args:
```bash
plix-server \
  --master-url http://localhost:8080 \
  --server-name "My Server" \
  --region "eu-west" \
  --tags "ctf,competitive"
```

Via environment:
```bash
export PLIX_MASTER_URL=http://localhost:8080
export PLIX_SERVER_NAME="My Server"
export PLIX_REGION=eu-west
export PLIX_TAGS=ctf,competitive
```

## Troubleshooting

### "Could not reach master server"
- Check master server is running
- Verify master URL is correct
- Check firewall/network connectivity

### "No servers found"
- Verify game server is running with `--master-url`
- Check game server logs for heartbeat errors
- Ensure master server received heartbeats

### "Rate limit exceeded"
- Wait 60 seconds before retrying
- This is expected for excessive requests from same IP

### Favorites not persisting
- Check `~/.config/plix/` directory permissions
- Look for errors in client logs

## API Testing with curl

```bash
# List servers
curl http://localhost:8080/servers | jq

# Send heartbeat
curl -X POST http://localhost:8080/heartbeat \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Server",
    "host": "127.0.0.1",
    "port": 7777,
    "region": "local",
    "tags": ["test"],
    "player_count": 0,
    "max_players": 32,
    "game_modes": ["ffa"],
    "protocol_version": "0.1.0"
  }' | jq

# Health check
curl http://localhost:8080/health | jq
```
