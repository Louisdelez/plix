# Data Model: Plix MVP v0.1

**Date**: 2025-12-14
**Status**: Draft

## Overview

This document defines the core data structures for the MVP. All types are designed for:
- Network serialization efficiency (compact binary)
- Deterministic simulation (fixed-point where needed)
- Clear ownership (server vs client vs shared)

## Core Types (plix-common)

### Identifiers

```rust
/// Unique player identifier (assigned by server)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u16);

/// Unique entity identifier (server-assigned)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u32);

/// Tick number (wraps at u32::MAX)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Tick(pub u32);

/// Input sequence number (per-player, wraps at u16::MAX)
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct InputSeq(pub u16);
```

### Math Types

```rust
/// 3D position (fixed-point for network, f32 for local)
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Block position (integer coordinates)
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Rotation (yaw/pitch in radians)
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Rotation {
    pub yaw: f32,   // -PI to PI
    pub pitch: f32, // -PI/2 to PI/2
}

/// Axis-aligned bounding box
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}
```

## Player State

### Server-Side (Authoritative)

```rust
/// Complete player state (server only)
pub struct ServerPlayer {
    pub id: PlayerId,
    pub name: String,           // Max 32 chars
    pub team: TeamId,

    // Transform
    pub position: Vec3,
    pub rotation: Rotation,
    pub velocity: Vec3,

    // Combat
    pub health: u8,             // 0-100
    pub is_dead: bool,
    pub respawn_tick: Option<Tick>,
    pub last_attack_tick: Tick,

    // Input processing
    pub last_input_seq: InputSeq,
    pub pending_inputs: VecDeque<PlayerInput>,

    // Network
    pub connection: ConnectionHandle,
    pub last_snapshot_ack: Tick,

    // Stats (current round)
    pub kills: u16,
    pub deaths: u16,
}
```

### Replicated State (Network)

```rust
/// Player state sent in snapshots
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub id: PlayerId,
    pub position: Vec3,
    pub rotation: Rotation,
    pub health: u8,
    pub is_dead: bool,
    pub animation: AnimationState,
}

#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
    Attacking,
    Dead,
}
```

### Client-Side (Predicted)

```rust
/// Local player state for prediction
pub struct LocalPlayer {
    // Confirmed state from server
    pub confirmed_position: Vec3,
    pub confirmed_rotation: Rotation,
    pub confirmed_input_seq: InputSeq,

    // Predicted state
    pub predicted_position: Vec3,
    pub predicted_velocity: Vec3,

    // Pending inputs awaiting server confirmation
    pub pending_inputs: VecDeque<(InputSeq, PlayerInput)>,
}

/// Remote player state for interpolation
pub struct RemotePlayer {
    pub id: PlayerId,

    // Interpolation buffer (ring buffer of snapshots)
    pub snapshots: VecDeque<(Tick, PlayerSnapshot)>,

    // Current interpolated state
    pub display_position: Vec3,
    pub display_rotation: Rotation,
    pub display_animation: AnimationState,
}
```

## Input Types

```rust
/// Player input for a single tick
#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerInput {
    pub seq: InputSeq,
    pub tick: Tick,             // Client's estimated server tick

    // Movement
    pub move_forward: f32,      // -1.0 to 1.0
    pub move_right: f32,        // -1.0 to 1.0
    pub jump: bool,
    pub crouch: bool,

    // Look
    pub yaw: f32,
    pub pitch: f32,

    // Actions
    pub attack: bool,
}
```

## World & Arena

### Block Types (MVP)

```rust
/// Block type identifier
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BlockType(pub u8);

impl BlockType {
    pub const AIR: Self = Self(0);
    pub const STONE: Self = Self(1);
    pub const BRICK: Self = Self(2);
    pub const METAL: Self = Self(3);
    // MVP: 4 block types sufficient
}

/// Block properties
pub struct BlockProperties {
    pub is_solid: bool,
    pub texture_id: u8,
}
```

### Arena Definition

```rust
/// Arena loaded from file
pub struct Arena {
    pub metadata: ArenaMetadata,
    pub blocks: ChunkStorage,
    pub spawn_points: Vec<SpawnPoint>,
}

pub struct ArenaMetadata {
    pub name: String,
    pub version: String,
    pub size: [u32; 3],         // x, y, z dimensions
}

pub struct SpawnPoint {
    pub team: TeamId,
    pub position: Vec3,
    pub rotation: f32,          // yaw
}

/// Team identifier
#[derive(Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct TeamId(pub u8);
```

### Chunk Storage

```rust
/// Chunk position in world
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Single chunk (16x16x16 blocks)
pub struct Chunk {
    pub pos: ChunkPos,
    pub blocks: [BlockType; 16 * 16 * 16],  // 4096 bytes
}

/// Arena block storage
pub struct ChunkStorage {
    pub chunks: HashMap<ChunkPos, Chunk>,
}
```

## Match State

```rust
/// Current match/round state
#[derive(Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: MatchPhase,
    pub round_number: u16,
    pub round_start_tick: Tick,
    pub round_time_limit: u32,  // seconds
    pub scores: Vec<TeamScore>,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum MatchPhase {
    WaitingForPlayers,
    Countdown,
    Playing,
    RoundEnd,
    MatchEnd,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TeamScore {
    pub team: TeamId,
    pub score: u32,
}
```

## Network Protocol Messages

### Client → Server

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Connection handshake
    Connect {
        protocol_version: u8,
        name: String,
    },

    /// Disconnect gracefully
    Disconnect,

    /// Player input
    Input(PlayerInput),

    /// Acknowledge snapshot receipt
    SnapshotAck { tick: Tick },
}
```

### Server → Client

```rust
#[derive(Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Connection accepted
    Connected {
        player_id: PlayerId,
        tick: Tick,
        tick_rate: u8,
    },

    /// Connection rejected
    Rejected { reason: String },

    /// Kicked from server
    Kicked { reason: String },

    /// World snapshot
    Snapshot(WorldSnapshot),

    /// Game events
    Event(GameEvent),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub tick: Tick,
    pub last_input_seq: InputSeq,   // Last processed input for this client
    pub players: Vec<PlayerSnapshot>,
    pub match_state: MatchState,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum GameEvent {
    PlayerJoined { id: PlayerId, name: String, team: TeamId },
    PlayerLeft { id: PlayerId },
    PlayerDied { victim: PlayerId, killer: Option<PlayerId> },
    PlayerRespawned { id: PlayerId },
    RoundStart { round: u16 },
    RoundEnd { winner: Option<TeamId> },
    MatchEnd { winner: Option<TeamId> },
}
```

## State Ownership Summary

| Data | Owner | Replicated To |
|------|-------|---------------|
| Player position/rotation | Server | All clients (snapshot) |
| Player health | Server | All clients (snapshot) |
| Player input | Client (sent) | Server only |
| Match state | Server | All clients (snapshot) |
| Arena blocks | Server | All clients (on connect) |
| Pending inputs | Client (local) | Not replicated |

## Validation Rules

| Field | Constraint | Enforced By |
|-------|------------|-------------|
| PlayerId | Unique per connection | Server |
| Player name | 1-32 chars, printable | Server |
| Position | Within arena bounds | Server |
| Velocity | Max 10 blocks/sec | Server |
| Health | 0-100 | Server |
| Attack cooldown | Min 500ms | Server |
| Input sequence | Monotonically increasing | Server |
| Protocol version | Must match server | Server |

## Serialization Notes

- All network messages use bincode (compact binary)
- Floats are f32 (sufficient precision, network-friendly)
- Strings are length-prefixed (2 bytes length + UTF-8)
- Enums are u8 discriminant + payload
- Arrays are length-prefixed (2 bytes)
