# Data Model: Voxel Game Platform - Visual Multiplayer

**Feature**: 002-voxel-game-platform
**Date**: 2025-12-14

## Overview

This document describes the data structures used for visualization in the client. Most structures already exist in `plix-common` and `plix-arena`. This feature adds rendering-specific structures in `plix-client`.

## Existing Structures (No Changes)

### Arena Data (plix-arena)

```
LoadedArena
├── definition: Arena
│   ├── metadata: ArenaMetadata
│   │   ├── name: String
│   │   ├── version: String
│   │   └── size: [u32; 3]  // x, y, z dimensions
│   └── spawn_points: Vec<SpawnPoint>
│       ├── team: u8
│       ├── position: [f32; 3]
│       └── rotation: f32  // yaw in degrees
│
└── blocks: Vec<BlockType>  // Flattened 3D array
    // Index = z * size_y * size_x + y * size_x + x
```

**BlockType Enum**:
- `AIR (0)` - Empty space
- `STONE (1)` - Gray solid block
- `BRICK (2)` - Red/brown solid block
- `METAL (3)` - Blue/silver solid block

### Player State (plix-common/protocol)

```
PlayerSnapshot
├── id: PlayerId (u16)
├── position: Vec3
├── rotation: Rotation
│   ├── yaw: f32    // radians
│   └── pitch: f32  // radians
├── health: u8  // 0-100
├── is_dead: bool
└── animation: AnimationState
```

### World State (plix-common/protocol)

```
WorldSnapshot
├── tick: Tick (u32)
├── last_input_seq: InputSeq (u16)
├── players: Vec<PlayerSnapshot>
└── match_state: MatchState
    ├── phase: MatchPhase
    │   └── WaitingForPlayers | Countdown | Playing | RoundEnd | MatchEnd
    ├── round_number: u16
    ├── round_start_tick: Tick
    ├── round_time_limit: u32  // seconds
    └── scores: Vec<TeamScore>
```

## New Structures (Client-Side)

### Render State

```
RenderState
├── arena_mesh: Option<Mesh>
├── player_mesh: Mesh  // Shared capsule geometry
├── player_instances: Vec<PlayerRenderInstance>
├── local_player_id: Option<PlayerId>
└── interpolation_buffer: InterpolationBuffer
```

### Player Render Instance

```
PlayerRenderInstance
├── player_id: PlayerId
├── position: Vec3  // Interpolated position
├── rotation: Rotation  // Interpolated rotation
├── color: [f32; 3]  // RGB based on team/local
└── visible: bool  // False for dead or local in FPS
```

### Interpolation Buffer

```
InterpolationBuffer
├── snapshots: VecDeque<TimestampedSnapshot>
│   └── TimestampedSnapshot
│       ├── receive_time: Instant
│       ├── tick: Tick
│       └── players: Vec<PlayerSnapshot>
├── interpolation_delay: Duration  // 100ms default
└── max_buffer_size: usize  // 10 snapshots
```

### Mesh Structure

```
Mesh
├── vertex_buffer: wgpu::Buffer
├── index_buffer: wgpu::Buffer
├── num_indices: u32
└── instance_buffer: Option<wgpu::Buffer>
```

### Vertex Format

```
Vertex (existing)
├── position: [f32; 3]
└── color: [f32; 3]
```

### HUD State

```
HudState
├── fps: f32
├── rtt_ms: u32
├── player_id: Option<PlayerId>
├── match_phase: MatchPhase
├── round_timer_secs: Option<u32>
└── last_update: Instant
```

### FPS Counter

```
FpsCounter
├── frame_times: VecDeque<Duration>  // Last 60 frames
└── current_fps: f32
```

## Data Flow

```
                    ┌─────────────┐
                    │   Server    │
                    └──────┬──────┘
                           │ WorldSnapshot
                           ▼
                    ┌─────────────┐
                    │  plix-net   │
                    └──────┬──────┘
                           │
          ┌────────────────┼────────────────┐
          ▼                ▼                ▼
   ┌────────────┐   ┌────────────┐   ┌────────────┐
   │ Interp.    │   │  Client    │   │  Metrics   │
   │ Buffer     │   │  State     │   │  (RTT)     │
   └─────┬──────┘   └─────┬──────┘   └─────┬──────┘
         │                │                │
         ▼                ▼                ▼
   ┌─────────────────────────────────────────────┐
   │              RenderState                     │
   │  ┌──────────┐  ┌───────────┐  ┌──────────┐  │
   │  │  Arena   │  │  Players  │  │   HUD    │  │
   │  │  Mesh    │  │ Instances │  │  State   │  │
   │  └──────────┘  └───────────┘  └──────────┘  │
   └─────────────────────┬───────────────────────┘
                         │
                         ▼
                    ┌─────────────┐
                    │   wgpu      │
                    │  Renderer   │
                    └─────────────┘
```

## State Transitions

### Interpolation Buffer

```
[New Snapshot Received]
    │
    ▼
┌───────────────────────────────┐
│ Add to buffer with timestamp  │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│ Trim buffer if > max_size     │
└───────────────────────────────┘

[Render Frame]
    │
    ▼
┌───────────────────────────────┐
│ Calculate render_time =       │
│ now - interpolation_delay     │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│ Find snap_a, snap_b around    │
│ render_time                   │
└───────────────┬───────────────┘
                │
                ▼
┌───────────────────────────────┐
│ Lerp position/rotation        │
│ between snap_a and snap_b     │
└───────────────────────────────┘
```

### Player Visibility

```
PlayerSnapshot
    │
    ▼
┌─────────────────────────────────┐
│ is_dead == true?                │──Yes──► visible = false
└───────────────┬─────────────────┘
                │ No
                ▼
┌─────────────────────────────────┐
│ id == local_player_id?          │──Yes──► visible = false (FPS view)
└───────────────┬─────────────────┘
                │ No
                ▼
visible = true
color = team_color(player.team)
```

## Validation Rules

| Field | Validation |
|-------|------------|
| `position` | Within arena bounds |
| `health` | 0-100 range |
| `tick` | Monotonically increasing |
| `interpolation_delay` | 50ms - 200ms |
| `max_buffer_size` | 5-20 snapshots |

## Color Scheme

| Entity | Color RGB |
|--------|-----------|
| Local player | (0.2, 0.4, 0.8) Blue |
| Team 0 remote | (0.8, 0.2, 0.2) Red |
| Team 1 remote | (0.2, 0.8, 0.2) Green |
| Stone block | (0.5, 0.5, 0.5) Gray |
| Brick block | (0.7, 0.3, 0.2) Brown |
| Metal block | (0.4, 0.5, 0.7) Steel |
