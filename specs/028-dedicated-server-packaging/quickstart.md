# Quickstart: Dedicated Server Packaging

**Feature**: 028-dedicated-server-packaging
**Date**: 2025-12-18

## Prerequisites

- Docker Engine 20.10+ (or Podman 4.0+)
- Linux x86_64 host (primary target)
- 2GB RAM minimum
- UDP port 7777 available (game server)
- TCP port 8080 available (master server, optional)

## Quick Deploy (2 Commands)

```bash
# Build the image
docker build -t plix-server -f deploy/docker/Dockerfile .

# Run the server
docker run -d -p 7777:7777/udp --name plix plix-server
```

Players can now connect to `your-server-ip:7777`.

## Verify Server Running

```bash
# Check container status
docker ps | grep plix

# View logs
docker logs plix

# Check health
docker inspect --format='{{.State.Health.Status}}' plix
```

## Configuration Examples

### FFA Server (Default)

```bash
docker run -d -p 7777:7777/udp \
  -e PLIX_SERVER_NAME="My FFA Server" \
  -e PLIX_GAME_MODE=ffa \
  -e PLIX_MAX_PLAYERS=16 \
  --name plix-ffa plix-server
```

### TDM Server

```bash
docker run -d -p 7777:7777/udp \
  -e PLIX_SERVER_NAME="Team Deathmatch" \
  -e PLIX_GAME_MODE=tdm \
  -e PLIX_MAX_PLAYERS=24 \
  -e PLIX_ARENA=test_arena \
  --name plix-tdm plix-server
```

### CTF Server

```bash
docker run -d -p 7777:7777/udp \
  -e PLIX_SERVER_NAME="Capture The Flag" \
  -e PLIX_GAME_MODE=ctf \
  -e PLIX_MAX_PLAYERS=32 \
  -e PLIX_ARENA=ctf_arena \
  --name plix-ctf plix-server
```

### BR Lite Server

```bash
docker run -d -p 7777:7777/udp \
  -e PLIX_SERVER_NAME="Battle Royale" \
  -e PLIX_GAME_MODE=br_lite \
  -e PLIX_MAX_PLAYERS=50 \
  --name plix-br plix-server
```

## With Data Persistence

```bash
# Create data directory
mkdir -p ./plix-data

# Run with volume
docker run -d -p 7777:7777/udp \
  -v $(pwd)/plix-data:/data \
  -e PLIX_PERSISTENCE=true \
  --name plix plix-server
```

## Full Stack with Master Server

```bash
# Using docker compose
cd deploy/docker
docker compose --profile full up -d

# Or manually
docker network create plix-net

docker run -d --network plix-net \
  -p 8080:8080 \
  --name plix-master plix-master

docker run -d --network plix-net \
  -p 7777:7777/udp \
  -e PLIX_MASTER_URL=http://plix-master:8080 \
  -e PLIX_MASTER_ENABLED=true \
  --name plix-server plix-server
```

## Using Scripts

```bash
# Build images
./deploy/scripts/build.sh --all

# Run server
./deploy/scripts/run.sh --name "My Server" --mode ffa

# Full stack
./deploy/scripts/compose.sh up --with-master --detach

# View logs
./deploy/scripts/compose.sh logs

# Stop
./deploy/scripts/compose.sh down
```

## Non-Docker Deployment

```bash
# Create release archive
./deploy/scripts/release-local.sh --version 1.0.0

# Extract and run
tar -xzf release/plix-server-1.0.0-linux-x86_64.tar.gz
cd plix-server-1.0.0-linux-x86_64
./bin/plix-server --port 7777 --game-modes ffa
```

## Environment Variables Reference

| Variable | Default | Description |
|----------|---------|-------------|
| `PLIX_SERVER_NAME` | Plix Server | Display name |
| `PLIX_PORT` | 7777 | UDP game port |
| `PLIX_MAX_PLAYERS` | 16 | Player limit |
| `PLIX_GAME_MODE` | ffa | Game mode |
| `PLIX_ARENA` | test_arena | Arena name |
| `PLIX_REGION` | unknown | Server region |
| `PLIX_MASTER_URL` | - | Master server URL |
| `PLIX_MASTER_ENABLED` | false | Enable registration |
| `RUST_LOG` | info | Log level |

## Troubleshooting

### Port Already in Use

```bash
# Check what's using port 7777
sudo lsof -i :7777

# Use different port
docker run -d -p 7778:7777/udp -e PLIX_PORT=7777 --name plix plix-server
```

### Container Exits Immediately

```bash
# Check logs
docker logs plix

# Common issues:
# - Invalid configuration
# - Missing assets
# - Port binding failed
```

### Players Can't Connect

1. Check firewall allows UDP 7777
2. Check NAT/port forwarding if behind router
3. Verify server is running: `docker ps`
4. Check server logs for connection attempts

### Log Management

```bash
# Docker handles log rotation by default
# Configure via daemon.json or per-container:
docker run -d \
  --log-driver json-file \
  --log-opt max-size=10m \
  --log-opt max-file=3 \
  --name plix plix-server
```

## Next Steps

1. Read full documentation in `docs/DEDICATED_SERVER.md`
2. Configure firewall and port forwarding
3. Set up monitoring (optional)
4. Register with master server for server browser
