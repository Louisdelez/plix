# Data Model: Dedicated Server Packaging

**Feature**: 028-dedicated-server-packaging
**Date**: 2025-12-18

## Entities

### ServerConfig

Server configuration entity loaded from TOML file with environment variable and CLI overrides.

**Source**: `deploy/config/server.toml`

```toml
# Server identity
server_name = "My Plix Server"      # PLIX_SERVER_NAME, --server-name
region = "eu-west"                   # PLIX_REGION, --region
tags = ["competitive", "vanilla"]    # PLIX_TAGS, --tags (comma-separated)

# Network
port = 7777                          # PLIX_PORT, --port
max_players = 16                     # PLIX_MAX_PLAYERS, --max-players

# Gameplay
game_mode = "ffa"                    # PLIX_GAME_MODE, --game-modes
arena = "test_arena"                 # PLIX_ARENA, --arena
tickrate = 60                        # PLIX_TICKRATE, --tickrate

# Master server registration
[master]
enabled = false                      # PLIX_MASTER_ENABLED
url = "http://localhost:8080"        # PLIX_MASTER_URL, --master-url

# Persistence (Feature 014)
[persistence]
enabled = false                      # PLIX_PERSISTENCE, --persistence
world_id = ""                        # PLIX_WORLD_ID, --world-id
autosave_interval = 300              # PLIX_AUTOSAVE_INTERVAL, --autosave-interval

# Paths
[paths]
assets_dir = "/data/assets"          # PLIX_ASSETS_DIR, --assets-dir
data_dir = "/data"                   # PLIX_DATA_DIR (container default)

# Logging
[logging]
level = "info"                       # PLIX_LOG_LEVEL, --log-level, RUST_LOG
```

**Validation Rules**:
- `port`: 1024-65535, must not be in use
- `max_players`: 1-64
- `tickrate`: 20-60
- `game_mode`: one of: ffa, tdm, ctf, br_lite
- `region`: non-empty string, max 32 chars
- `server_name`: non-empty string, max 64 chars
- `tags`: array of strings, max 10 tags, each max 32 chars

### ConfigPriority

Configuration value resolution order (highest to lowest):

| Priority | Source | Example |
|----------|--------|---------|
| 1 (highest) | CLI flags | `--port 7778` |
| 2 | Environment variables | `PLIX_PORT=7778` |
| 3 | Config file | `port = 7778` |
| 4 (lowest) | Defaults | `7777` |

### DataDirectory

Container data directory structure.

```
/data/
├── config/
│   └── server.toml          # Main configuration (optional, uses defaults)
├── worlds/
│   └── <world_id>/          # Per-world persistence data
│       ├── world.bin        # Serialized world state
│       └── meta.toml        # World metadata
├── logs/
│   └── (optional file logs) # Only if explicitly configured
└── assets/
    └── arenas/              # Custom arena definitions
        └── custom.toml
```

**Volume Mount Points**:
- `/data` - All persistent data (recommended single mount)
- `/data/config` - Config only (for read-only config injection)
- `/data/worlds` - World persistence only
- `/data/assets` - Custom assets only

### ReleaseArchive

Non-Docker release package structure.

```
plix-server-<version>-linux-x86_64/
├── bin/
│   └── plix-server          # Static binary
├── assets/
│   └── arenas/
│       ├── test_arena.toml
│       ├── ffa_arena.toml
│       ├── ctf_arena.toml
│       └── training_arena.toml
├── config/
│   └── server.toml.example
├── README.md                # Quick start guide
├── LICENSE
└── CHECKSUMS.sha256         # SHA256 checksums for all files
```

### DockerImage

Docker image metadata.

| Attribute | Value |
|-----------|-------|
| Base Image | `debian:bookworm-slim@sha256:<digest>` |
| User | `plix` (UID 1000, GID 1000) |
| Workdir | `/app` |
| Data Dir | `/data` |
| Exposed Ports | `7777/udp` (game) |
| Entrypoint | `["/app/plix-server"]` |
| CMD | `["--assets-dir", "/app/assets"]` |

### DockerComposeService

Service definitions for docker-compose.yml.

**plix-server service**:
```yaml
services:
  plix-server:
    image: plix-server:latest
    ports:
      - "7777:7777/udp"
    volumes:
      - ./data:/data
    environment:
      - PLIX_SERVER_NAME=My Server
      - PLIX_GAME_MODE=ffa
      - PLIX_MAX_PLAYERS=16
    restart: unless-stopped
```

**plix-master service** (optional):
```yaml
services:
  plix-master:
    image: plix-master:latest
    ports:
      - "8080:8080/tcp"
    environment:
      - PLIX_MASTER_TTL=60
      - PLIX_MASTER_RATE_LIMIT=10
    restart: unless-stopped
```

## State Transitions

### ServerStartup

```
[Initial]
    │
    ▼
[LoadConfig] ─── Config Error ──► [Exit(1)]
    │
    ▼
[ValidateConfig] ─── Invalid ──► [Exit(1)]
    │
    ▼
[BindPort] ─── Port in use ──► [Exit(1)]
    │
    ▼
[LoadArena] ─── Arena not found ──► [Exit(1)]
    │
    ▼
[RegisterMaster] ─── Failed ──► [Continue without registration]
    │
    ▼
[Running] ◄──── Heartbeat loop ────┐
    │                               │
    └───────────────────────────────┘
```

### ConfigReload (future consideration)

Not in scope for v1. Configuration is read once at startup.

## Environment Variable Mapping

Complete mapping of PLIX_* environment variables to CLI flags:

| Environment Variable | CLI Flag | Type | Default |
|---------------------|----------|------|---------|
| `PLIX_PORT` | `--port` | u16 | 7777 |
| `PLIX_SERVER_NAME` | `--server-name` | String | "Plix Server" |
| `PLIX_REGION` | `--region` | String | "unknown" |
| `PLIX_TAGS` | `--tags` | String (comma-sep) | "" |
| `PLIX_MAX_PLAYERS` | `--max-players` | u8 | 16 |
| `PLIX_GAME_MODE` | `--game-modes` | String | "ffa" |
| `PLIX_ARENA` | `--arena` | String | "test_arena" |
| `PLIX_TICKRATE` | `--tickrate` | u8 | 60 |
| `PLIX_ASSETS_DIR` | `--assets-dir` | Path | "assets" |
| `PLIX_LOG_LEVEL` | `--log-level` | String | "info" |
| `PLIX_PERSISTENCE` | `--persistence` | bool | false |
| `PLIX_WORLD_ID` | `--world-id` | String | "" |
| `PLIX_AUTOSAVE_INTERVAL` | `--autosave-interval` | u64 | 300 |
| `PLIX_MASTER_URL` | `--master-url` | URL | "" |
| `PLIX_MASTER_ENABLED` | (new) | bool | false |

## No Database

This feature uses file-based configuration only. No database is required.
World persistence (Feature 014) uses binary files, not a database.
