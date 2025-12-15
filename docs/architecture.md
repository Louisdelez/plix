# Plix Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLIENTS (8-16)                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Input      │  │  Prediction  │  │ Interpolation│          │
│  │   Capture    │  │  (local)     │  │  (remote)    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         └─────────────────┼─────────────────┘                   │
│                           │                                     │
│                    ┌──────▼───────┐                             │
│                    │   Renderer   │                             │
│                    └──────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
                            │ UDP
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    AUTHORITATIVE SERVER                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Session    │  │  Validation  │  │  Match State │          │
│  │   Manager    │  │  (anti-cheat)│  │   Machine    │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                   │
│         └─────────────────┼─────────────────┘                   │
│                           │                                     │
│                    ┌──────▼───────┐                             │
│                    │  Simulation  │                             │
│                    │  (movement,  │                             │
│                    │   combat)    │                             │
│                    └──────┬───────┘                             │
│                           │                                     │
│                    ┌──────▼───────┐                             │
│                    │  Replication │                             │
│                    │  (snapshots) │                             │
│                    └──────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Dependencies

```
plix-common ◄──────────────────────────────────┐
     │                                          │
     ▼                                          │
plix-net ◄────────┬──────────────┬─────────────┤
                  │              │              │
                  ▼              ▼              │
           plix-server     plix-client         │
                  │              │              │
                  └──────┬───────┘              │
                         ▼                      │
                   plix-arena ◄─────────────────┤
                         │                      │
                         ▼                      │
                   plix-tools ◄─────────────────┘
```

## Module Structure

### plix-common

| Module | Purpose |
|--------|---------|
| `math` | Vec3, Rotation, AABB types |
| `types` | PlayerId, EntityId, Tick, InputSeq |
| `time` | Tick math, rate configuration |
| `protocol` | Message definitions, binary codec |

### plix-net

| Module | Purpose |
|--------|---------|
| `transport` | UDP socket wrapper |
| `packet` | Header format (version, channel, seq, ack) |
| `channel/unreliable` | Fire-and-forget delivery |
| `channel/reliable` | ACK/resend delivery |
| `channel/ordered` | Ordered reliable delivery |
| `connection` | State machine (handshake, keepalive, timeout) |
| `metrics` | RTT, jitter, packet loss measurement |

### plix-server

| Module | Purpose |
|--------|---------|
| `tick` | Fixed 60 Hz game loop |
| `session` | Player connection management |
| `validation` | Input validation, anti-speedhack |
| `match_state` | Round state machine |
| `sim/movement` | Player movement physics |
| `sim/collision` | AABB collision detection |
| `sim/combat` | Melee attack validation |
| `replication/state` | Authoritative game state |
| `replication/snapshot` | Delta compression |
| `replication/events` | Game event buffer |

### plix-client

| Module | Purpose |
|--------|---------|
| `input` | FPS input capture |
| `commands` | Command buffer with sequence |
| `prediction` | Local player prediction |
| `reconciliation` | Server correction handling |
| `interpolation` | Remote player smoothing |
| `net` | Network message handling |
| `render/*` | Voxel and player rendering |
| `ui/*` | HUD and debug overlays |

### plix-arena

| Module | Purpose |
|--------|---------|
| `format` | Arena file structures |
| `loader` | TOML arena loading |
| `validate` | Arena validation |
| `spawn` | Spawn point management |

### plix-tools

| Module | Purpose |
|--------|---------|
| `bot` | Headless bot client |
| `net_sim` | Network condition simulation |

## Data Flow

### Client Input to Server

1. Client captures input (WASD, mouse)
2. Input buffered with sequence number
3. Client predicts local movement
4. Input sent to server via UDP

### Server Processing

1. Server receives input
2. Validates input (rate limits, bounds)
3. Applies to authoritative state
4. Runs simulation tick
5. Generates world snapshot

### Server to Client

1. Server sends snapshot to all clients
2. Client receives snapshot
3. Reconciles local prediction with server state
4. Interpolates remote players
5. Renders frame

## Tick Rate

- Server: 60 Hz (16.67ms per tick)
- Client send: 60 Hz
- Client render: Variable (vsync or unlimited)
- Snapshot broadcast: 20-60 Hz (configurable)
