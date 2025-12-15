# Render API Contract

**Feature**: 002-voxel-game-platform
**Date**: 2025-12-14

## Overview

This document defines the internal API contracts for the rendering system. Since this is a native Rust application (not a web service), these are module-level interfaces rather than HTTP endpoints.

## Module Interfaces

### VoxelRenderer (voxels.rs)

```rust
/// Generate mesh from arena block data
pub fn generate_arena_mesh(
    arena: &LoadedArena,
    device: &wgpu::Device,
) -> Mesh;

/// Block type to color mapping
pub fn block_color(block_type: BlockType) -> [f32; 3];
```

**Inputs**:
- `arena`: Loaded arena with block array and dimensions
- `device`: wgpu device for buffer creation

**Outputs**:
- `Mesh`: Vertex and index buffers ready for rendering

**Behavior**:
- Generates faces only between solid and non-solid blocks
- Applies color based on block type
- Returns empty mesh if arena is empty

---

### PlayerRenderer (players.rs)

```rust
/// Generate shared player mesh (capsule)
pub fn generate_player_mesh(device: &wgpu::Device) -> Mesh;

/// Update player instances from interpolated state
pub fn update_player_instances(
    instances: &mut Vec<PlayerRenderInstance>,
    players: &[InterpolatedPlayer],
    local_player_id: Option<PlayerId>,
);

/// Upload instance data to GPU buffer
pub fn upload_instances(
    instances: &[PlayerRenderInstance],
    buffer: &wgpu::Buffer,
    queue: &wgpu::Queue,
);
```

**Inputs**:
- `players`: Interpolated player positions from buffer
- `local_player_id`: ID of local player (for visibility)

**Outputs**:
- `instances`: Updated render instances

**Behavior**:
- Local player not rendered (first-person view)
- Dead players not rendered
- Color based on team membership

---

### InterpolationBuffer (interpolation.rs)

```rust
/// Add new snapshot to buffer
pub fn push_snapshot(&mut self, snapshot: WorldSnapshot, receive_time: Instant);

/// Get interpolated player states for current render time
pub fn interpolate(&self, render_time: Instant) -> Vec<InterpolatedPlayer>;

/// Clear all buffered snapshots
pub fn clear(&mut self);
```

**Inputs**:
- `snapshot`: WorldSnapshot from server
- `receive_time`: When snapshot was received

**Outputs**:
- `Vec<InterpolatedPlayer>`: Interpolated positions for all players

**Configuration**:
```rust
InterpolationConfig {
    delay_ms: 100,      // Render delay behind real-time
    max_buffer: 10,     // Max snapshots to keep
    snap_threshold_ms: 500,  // Snap instead of lerp if gap too large
}
```

---

### HUD (hud.rs)

```rust
/// Update HUD state from current metrics
pub fn update(&mut self,
    fps: f32,
    rtt_ms: u32,
    player_id: Option<PlayerId>,
    match_state: &MatchState,
);

/// Format HUD for window title
pub fn format_title(&self) -> String;
```

**Output Format**:
```
"PLIX | FPS: {fps:.0} | Ping: {rtt}ms | ID: {id} | {phase} {timer}"
```

**Examples**:
- `"PLIX | FPS: 60 | Ping: 25ms | ID: 1 | Playing 2:45"`
- `"PLIX | FPS: 58 | Ping: -- | ID: -- | Connecting..."`
- `"PLIX | FPS: 60 | Ping: 30ms | ID: 3 | Countdown 3"`

---

### FpsCounter (hud.rs)

```rust
/// Record frame time
pub fn record_frame(&mut self, delta: Duration);

/// Get current FPS (smoothed)
pub fn fps(&self) -> f32;
```

**Behavior**:
- Maintains rolling window of 60 frame times
- Returns 1.0 / average_frame_time
- Clamps to 0.0 if no frames recorded

---

## Render Pipeline Flow

```
Frame Start
    │
    ├── FpsCounter.record_frame(delta)
    │
    ├── InterpolationBuffer.interpolate(now - 100ms)
    │       │
    │       └── Vec<InterpolatedPlayer>
    │
    ├── PlayerRenderer.update_player_instances(...)
    │
    ├── HUD.update(fps, rtt, player_id, match_state)
    │
    ├── Window.set_title(hud.format_title())
    │
    ├── RenderPass
    │   ├── Draw arena_mesh
    │   └── Draw player instances
    │
    └── Present
```

## Error Handling

| Scenario | Handling |
|----------|----------|
| Empty arena | Render nothing (no mesh) |
| No snapshots | Keep last interpolated state |
| Missing player in snapshot | Remove from instances |
| New player in snapshot | Add to instances |
| RTT unavailable | Display "--" in HUD |
| Disconnected | Display "Disconnected" in title |

## Thread Safety

All rendering operations run on main thread (winit event loop). No synchronization needed within render code.

Network operations (snapshot reception) may occur on async task. Snapshots passed via channel to main thread before processing.
