# Contract: Docker Compose

**Feature**: 028-dedicated-server-packaging
**Type**: Service Orchestration Specification

## docker-compose.yml

```yaml
version: "3.8"

services:
  # ==========================================================================
  # Game Server
  # ==========================================================================
  plix-server:
    build:
      context: ../..
      dockerfile: deploy/docker/Dockerfile
    image: plix-server:latest
    container_name: plix-server
    ports:
      - "7777:7777/udp"
    volumes:
      - plix-data:/data
      - ./config:/data/config:ro
    environment:
      # Server identity
      - PLIX_SERVER_NAME=${PLIX_SERVER_NAME:-Plix Server}
      - PLIX_REGION=${PLIX_REGION:-unknown}
      - PLIX_TAGS=${PLIX_TAGS:-}
      # Gameplay
      - PLIX_GAME_MODE=${PLIX_GAME_MODE:-ffa}
      - PLIX_ARENA=${PLIX_ARENA:-test_arena}
      - PLIX_MAX_PLAYERS=${PLIX_MAX_PLAYERS:-16}
      - PLIX_TICKRATE=${PLIX_TICKRATE:-60}
      # Master server registration
      - PLIX_MASTER_URL=${PLIX_MASTER_URL:-http://plix-master:8080}
      - PLIX_MASTER_ENABLED=${PLIX_MASTER_ENABLED:-false}
      # Logging
      - RUST_LOG=${RUST_LOG:-info}
    restart: unless-stopped
    networks:
      - plix-network
    depends_on:
      plix-master:
        condition: service_healthy
        required: false

  # ==========================================================================
  # Master Server (Server Browser)
  # ==========================================================================
  plix-master:
    build:
      context: ../..
      dockerfile: deploy/docker/Dockerfile.master
    image: plix-master:latest
    container_name: plix-master
    ports:
      - "8080:8080/tcp"
    environment:
      - PLIX_MASTER_TTL=${PLIX_MASTER_TTL:-60}
      - PLIX_MASTER_RATE_LIMIT=${PLIX_MASTER_RATE_LIMIT:-10}
      - RUST_LOG=${RUST_LOG:-info}
    restart: unless-stopped
    networks:
      - plix-network
    profiles:
      - full
      - master

volumes:
  plix-data:
    driver: local

networks:
  plix-network:
    driver: bridge
```

## Usage Profiles

### Server Only (Default)

```bash
docker compose up plix-server
```

### Server + Master (Full Stack)

```bash
docker compose --profile full up
```

### Master Only

```bash
docker compose --profile master up plix-master
```

## Environment File (.env)

```env
# Server identity
PLIX_SERVER_NAME=My Plix Server
PLIX_REGION=eu-west
PLIX_TAGS=competitive,vanilla

# Gameplay
PLIX_GAME_MODE=ffa
PLIX_ARENA=test_arena
PLIX_MAX_PLAYERS=16
PLIX_TICKRATE=60

# Master registration
PLIX_MASTER_ENABLED=true
PLIX_MASTER_URL=http://plix-master:8080

# Master server settings
PLIX_MASTER_TTL=60
PLIX_MASTER_RATE_LIMIT=10

# Logging
RUST_LOG=info
```

## Volume Mounts

| Mount | Container Path | Purpose |
|-------|---------------|---------|
| `plix-data` | `/data` | All persistent data |
| `./config` | `/data/config:ro` | Custom configuration (read-only) |

## Port Mappings

| Service | Host Port | Container Port | Protocol |
|---------|-----------|----------------|----------|
| plix-server | 7777 | 7777 | UDP |
| plix-master | 8080 | 8080 | TCP |

## Network

- Internal network: `plix-network` (bridge)
- Server discovers master via hostname `plix-master`
- Master accessible from host on port 8080
