# Plix Dedicated Server Guide

This guide covers deploying and configuring a Plix dedicated game server.

## Table of Contents

- [Quick Start](#quick-start)
- [Deployment Options](#deployment-options)
- [Configuration Reference](#configuration-reference)
- [Game Modes](#game-modes)
- [Log Management](#log-management)
- [Troubleshooting](#troubleshooting)
- [Reproducible Builds](#reproducible-builds)

---

## Quick Start

### Docker (Recommended)

Deploy a Plix server in 2 commands:

```bash
# Build the image
./deploy/scripts/build.sh

# Run the server
./deploy/scripts/run.sh
```

The server starts with default settings:
- **Game Mode**: FFA (Free-For-All)
- **Port**: 7777/udp
- **Max Players**: 16

### Docker Compose (Multi-Service)

Deploy server with master (server browser):

```bash
# Start full stack
./deploy/scripts/compose.sh --with-master --detach

# View logs
./deploy/scripts/compose.sh logs

# Stop
./deploy/scripts/compose.sh down
```

### Non-Docker Deployment

For servers without Docker:

```bash
# Create release archive
./deploy/scripts/release-local.sh

# Deploy to remote server
scp release/plix-server-*.tar.gz user@server:/opt/
ssh user@server 'cd /opt && tar -xzf plix-server-*.tar.gz && cd plix-server && ./plix-server'
```

---

## Deployment Options

### Option 1: Docker (Single Container)

Best for: Quick deployment, isolated environment

```bash
# Build
./deploy/scripts/build.sh

# Run with defaults
./deploy/scripts/run.sh

# Run with custom settings
./deploy/scripts/run.sh \
  --name "My Server" \
  --mode tdm \
  --port 7778 \
  --players 32 \
  --detach
```

### Option 2: Docker Compose (Full Stack)

Best for: Production deployment with server browser

```bash
# Copy environment template
cp deploy/docker/.env.example deploy/docker/.env

# Edit configuration
vim deploy/docker/.env

# Start services
./deploy/scripts/compose.sh --with-master --detach
```

Compose profiles:
- Default (no flag): Game server only
- `--with-master`: Game server + master server
- `--master-only`: Master server only

### Option 3: Binary Release

Best for: Non-Docker environments, custom init systems

```bash
# Create archive
./deploy/scripts/release-local.sh --version 1.0.0

# Extract and run
tar -xzf release/plix-server-1.0.0-linux-x86_64.tar.gz
cd plix-server
./plix-server
```

---

## Configuration Reference

Configuration priority: **CLI flags > Environment variables > Defaults**

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PLIX_PORT` | `7777` | UDP port for game traffic |
| `PLIX_SERVER_NAME` | `Plix Server` | Display name in server browser |
| `PLIX_REGION` | `unknown` | Geographic region (e.g., `eu-west`) |
| `PLIX_TAGS` | (empty) | Comma-separated tags for filtering |
| `PLIX_GAME_MODE` | `ffa` | Game mode: `ffa`, `tdm`, `ctf`, `br_lite` |
| `PLIX_ARENA` | `test_arena` | Arena name from `assets/arenas/` |
| `PLIX_MAX_PLAYERS` | `16` | Maximum concurrent players |
| `PLIX_TICKRATE` | `60` | Server tick rate (20-60 Hz) |
| `PLIX_MASTER_URL` | (none) | Master server URL for registration |
| `RUST_LOG` | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

### CLI Flags

```bash
./plix-server [OPTIONS]

Options:
  --port <PORT>              UDP port [default: 7777]
  --server-name <NAME>       Server display name [default: "Plix Server"]
  --region <REGION>          Geographic region [default: "unknown"]
  --tags <TAGS>              Comma-separated tags
  --mode <MODE>              Game mode: ffa, tdm, ctf, br_lite [default: ffa]
  --arena <ARENA>            Arena name [default: test_arena]
  --max-players <N>          Max players [default: 16]
  --tickrate <HZ>            Tick rate [default: 60]
  --master-url <URL>         Master server URL
  --assets-dir <PATH>        Assets directory [default: ./assets]
  -h, --help                 Show help
```

### Configuration File

Create `server.toml` for persistent configuration:

```toml
# Server Identity
server_name = "My Plix Server"
region = "eu-west"
tags = ["competitive", "vanilla"]

# Network
port = 7777
max_players = 24

# Gameplay
game_mode = "tdm"
arena = "test_arena"
tickrate = 60

# Logging
log_level = "info"
```

Example configs are in `deploy/config/`:
- `server.toml.example` - Full reference
- `server-ffa.toml` - Free-For-All preset
- `server-tdm.toml` - Team Deathmatch preset
- `server-ctf.toml` - Capture The Flag preset
- `server-br.toml` - Battle Royale Lite preset

---

## Game Modes

### FFA (Free-For-All)

Every player for themselves. First to score limit wins.

```bash
./plix-server --mode ffa --max-players 16
```

### TDM (Team Deathmatch)

Two teams compete for kills. Team with most kills wins.

```bash
./plix-server --mode tdm --max-players 24
```

### CTF (Capture The Flag)

Teams capture enemy flags. First team to capture limit wins.

```bash
./plix-server --mode ctf --max-players 32
```

### BR Lite (Battle Royale)

Last player/team standing wins. Shrinking play zone.

```bash
./plix-server --mode br_lite --max-players 50
```

---

## Log Management

### Log Levels

Control verbosity with `RUST_LOG`:

```bash
# Production (default)
RUST_LOG=info ./plix-server

# Debugging
RUST_LOG=debug ./plix-server

# Verbose tracing
RUST_LOG=trace ./plix-server

# Quiet (errors only)
RUST_LOG=error ./plix-server

# Module-specific
RUST_LOG=plix_server=debug,plix_common=warn ./plix-server
```

### Docker Logging

Plix outputs all logs to stdout/stderr. Docker handles log management.

View logs:
```bash
docker logs plix-server
docker logs -f plix-server  # Follow
docker logs --tail 100 plix-server  # Last 100 lines
```

Configure Docker logging driver in `docker-compose.yml`:
```yaml
services:
  plix-server:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Systemd Integration

For non-Docker deployments, use systemd for log management:

```ini
# /etc/systemd/system/plix-server.service
[Unit]
Description=Plix Game Server
After=network.target

[Service]
Type=simple
User=plix
WorkingDirectory=/opt/plix-server
ExecStart=/opt/plix-server/plix-server
Restart=on-failure
Environment="RUST_LOG=info"

[Install]
WantedBy=multi-user.target
```

View logs:
```bash
journalctl -u plix-server -f
```

---

## Troubleshooting

### Port Issues

**Symptom**: Server starts but players cannot connect.

1. Check firewall:
   ```bash
   sudo ufw allow 7777/udp
   # or
   sudo firewall-cmd --add-port=7777/udp --permanent
   sudo firewall-cmd --reload
   ```

2. Check port is listening:
   ```bash
   ss -ulnp | grep 7777
   ```

3. Check NAT/port forwarding if behind router.

### Container Won't Start

**Symptom**: Docker container exits immediately.

1. Check logs:
   ```bash
   docker logs plix-server
   ```

2. Run interactively:
   ```bash
   docker run -it --rm plix-server:latest
   ```

3. Verify image built correctly:
   ```bash
   ./deploy/scripts/build.sh --no-cache
   ```

### Assets Not Found

**Symptom**: Server logs "arena not found" errors.

1. Check assets are included:
   ```bash
   docker run --rm plix-server:latest ls -la /app/assets/arenas/
   ```

2. Verify arena name matches file:
   ```bash
   ls assets/arenas/
   # Use name without .toml extension
   ```

### Master Server Connection

**Symptom**: Server doesn't appear in server browser.

1. Check master URL is correct:
   ```bash
   docker logs plix-server | grep master
   ```

2. Verify master server is reachable:
   ```bash
   curl http://master-server:8080/servers
   ```

3. Check network connectivity between containers:
   ```bash
   docker exec plix-server ping plix-master
   ```

### Performance Issues

**Symptom**: High latency, stuttering gameplay.

1. Check tick rate is achievable:
   ```bash
   # Lower tick rate if server is under-powered
   ./plix-server --tickrate 30
   ```

2. Monitor resource usage:
   ```bash
   docker stats plix-server
   ```

3. Check log for warnings:
   ```bash
   RUST_LOG=warn ./plix-server
   ```

---

## Reproducible Builds

Plix uses version pinning for reproducible builds across machines and time.

### Version Pinning Strategy

| Component | Pin Location | Update Procedure |
|-----------|--------------|------------------|
| Rust toolchain | `rust-toolchain.toml` | Change channel version |
| Dependencies | `Cargo.lock` | Run `cargo update` |
| Base image | `Dockerfile` | Update image tag |

### rust-toolchain.toml

Located at repository root. Pins Rust version for all builds.

```toml
[toolchain]
channel = "1.75.0"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

**Update procedure**:
1. Edit `rust-toolchain.toml` with new version
2. Run `cargo build` to verify compatibility
3. Run tests: `cargo test`
4. Commit both `rust-toolchain.toml` and any `Cargo.lock` changes

### Cargo.lock

**IMPORTANT**: `Cargo.lock` is committed to the repository. Do NOT delete it.

The lock file ensures:
- Exact same dependency versions on every build
- No surprise updates from upstream crates
- Reproducible binaries

**Update procedure**:
1. Run `cargo update` to update all dependencies
2. Or `cargo update -p <crate>` for specific crate
3. Run tests to verify: `cargo test`
4. Commit the updated `Cargo.lock`

### Docker Base Image

The Dockerfile pins the Debian base image:

```dockerfile
FROM debian:bookworm-slim
```

For maximum reproducibility, use a specific digest:

```dockerfile
FROM debian:bookworm-slim@sha256:<digest>
```

### SOURCE_DATE_EPOCH

For timestamp reproducibility, set in Dockerfile:

```dockerfile
ENV SOURCE_DATE_EPOCH=0
```

This ensures embedded timestamps don't change between builds.

---

## Data Persistence

### Volume Structure

The `/data` directory contains all persistent data:

```
/data/
├── config/    # Custom configuration files
├── worlds/    # World persistence data
└── logs/      # Optional file-based logs
```

### Docker Volume Mount

```bash
# Named volume (recommended)
docker run -v plix-data:/data plix-server

# Bind mount
docker run -v /path/on/host:/data plix-server

# Config only (read-only)
docker run -v ./config:/data/config:ro plix-server
```

### Ephemeral Mode

Run without persistence (data lost on container stop):

```bash
docker run plix-server  # No -v flag
```

---

## Security Considerations

1. **Non-root container**: Server runs as user `plix` (UID 1000)
2. **Minimal image**: Only essential packages installed
3. **No shell in production**: Consider `--read-only` flag
4. **Network isolation**: Use Docker networks to isolate services

```bash
# Read-only filesystem (more secure)
docker run --read-only -v plix-data:/data plix-server
```
