# Research: Voxel Game Platform - Visual Multiplayer

**Feature**: 002-voxel-game-platform
**Date**: 2025-12-14
**Status**: Complete

## Research Questions

### RQ-1: Voxel Mesh Generation Strategy

**Context**: Need to render arena blocks efficiently. Arena size up to 32x16x32 = 16,384 blocks.

**Decision**: Start with naive cube-per-visible-face, defer greedy meshing.

**Rationale**:
- Naive approach generates more vertices but is simpler to implement correctly
- Test arena (32x16x32) produces ~50k-100k triangles max (acceptable for 30 FPS target)
- Greedy meshing is an optimization that can be added later if needed
- Existing `voxels.rs` has placeholder structure ready for implementation

**Alternatives Considered**:
1. **Greedy meshing**: Combines adjacent faces into larger quads. Better performance but more complex. Rejected for MVP.
2. **Marching cubes**: Produces smooth surfaces. Not appropriate for blocky voxel aesthetic.
3. **Pre-baked meshes**: Load from file. Adds asset pipeline complexity. Rejected.

**Implementation Notes**:
- Generate mesh once at arena load (static arena)
- Use block type to determine face colors
- Only generate faces between solid and non-solid blocks (face culling)
- Vertex format: position + color (existing in engine.rs)

---

### RQ-2: Player Visual Representation

**Context**: Need simple visual for players. Must distinguish local vs remote.

**Decision**: Use colored capsules (cylinder + hemispheres approximated as elongated cubes).

**Rationale**:
- Capsule shape matches player collision box (radius 0.3, height 1.8)
- Simple geometry (few triangles)
- Color differentiation: local player = blue, remote = red/green by team
- Existing `players.rs` and `PlayerInstance` struct support this

**Alternatives Considered**:
1. **Cubes**: Too primitive, doesn't convey player orientation.
2. **Full 3D models**: Out of scope, requires asset loading.
3. **Billboard sprites**: Doesn't work well with FPS camera.

**Implementation Notes**:
- Generate capsule mesh at startup (shared geometry)
- Instance buffer for per-player transforms
- Local player may be invisible in first-person (camera position = player position)

---

### RQ-3: Snapshot Interpolation Approach

**Context**: Remote players receive position updates at 60 Hz server tick rate. Need smooth movement.

**Decision**: Buffer-based interpolation with 100ms render delay.

**Rationale**:
- Standard technique for authoritative multiplayer games
- 100ms delay = 6 ticks at 60 Hz, provides buffer for network jitter
- Linear interpolation between two snapshot positions
- Existing `interpolation.rs` has placeholder structure

**Alternatives Considered**:
1. **No interpolation**: Would show discrete jumps at tick boundaries.
2. **Extrapolation**: Predicts future position. Can cause rubber-banding on direction changes.
3. **Hermite interpolation**: Smoother curves but more complex and requires velocity data.

**Implementation Notes**:
```
render_time = current_time - interpolation_delay (100ms)
find snapshots: snap_a.tick <= render_time < snap_b.tick
t = (render_time - snap_a.time) / (snap_b.time - snap_a.time)
position = lerp(snap_a.position, snap_b.position, t)
rotation = slerp(snap_a.rotation, snap_b.rotation, t)
```

**Edge Cases**:
- Missing snapshot: Hold last known position
- Large gap (>500ms): Snap to latest position (reconnection scenario)
- Buffer too old: Discard stale snapshots

---

### RQ-4: HUD Rendering Strategy

**Context**: Need to display FPS, ping, player_id, round state. Must not impact game performance.

**Decision**: Window title fallback for MVP, optional wgpu_text overlay later.

**Rationale**:
- Window title update is trivial and works immediately
- No additional dependencies
- Provides functional HUD for debugging without rendering pipeline changes
- Can upgrade to proper text rendering in future iteration

**Alternatives Considered**:
1. **wgpu_text/glyph_brush**: Full text rendering. Adds dependency and complexity.
2. **egui integration**: Full immediate-mode GUI. Overkill for debug HUD.
3. **Pre-rendered bitmap font**: Works but requires texture loading infrastructure.

**Implementation Notes**:
- Update window title every 500ms (avoid excessive updates)
- Format: `PLIX | FPS: 60 | Ping: 25ms | ID: 1 | Round: Playing 2:45`
- Future: Add overlay rendering as separate feature

---

### RQ-5: Arena Data Source

**Context**: Client needs arena geometry. Server sends `arena_data: Vec<u8>` in Connected message.

**Decision**: Use server-provided arena data (existing protocol).

**Rationale**:
- Server already sends serialized arena in `Connected` message
- Ensures client and server have identical arena
- No need for separate asset distribution
- Already implemented in protocol

**Alternatives Considered**:
1. **Local file loading**: Client loads from disk. Requires asset sync mechanism.
2. **Download from URL**: Adds complexity and network dependency.
3. **Embedded arenas**: Bake into binary. Inflexible.

**Implementation Notes**:
- Deserialize `arena_data` from `Connected` message
- `LoadedArena` struct contains block array
- Generate mesh immediately after receiving arena data

---

### RQ-6: Headless Mode Preservation

**Context**: Headless mode must continue working for tests and load tests.

**Decision**: Conditional compilation/runtime check for rendering code.

**Rationale**:
- Existing structure already separates windowed vs headless in `main.rs`
- Render module only instantiated in windowed mode
- No changes needed to headless path

**Alternatives Considered**:
1. **Feature flags**: `#[cfg(feature = "render")]`. Adds build complexity.
2. **Separate binaries**: One for headless, one for windowed. Maintenance burden.

**Implementation Notes**:
- `RenderEngine` only created when window exists
- Headless mode uses same network/state code, just no rendering
- Verify with `cargo run --bin plix-client -- --headless`

---

### RQ-7: FPS Measurement

**Context**: Need accurate FPS display in HUD.

**Decision**: Frame time averaging over 60 frames.

**Rationale**:
- Smooths out frame time spikes
- Standard approach for FPS counters
- 60-frame window = ~1 second at target framerate

**Implementation Notes**:
```rust
struct FpsCounter {
    frame_times: VecDeque<Duration>,
    last_update: Instant,
}

impl FpsCounter {
    fn update(&mut self, delta: Duration) -> f32 {
        self.frame_times.push_back(delta);
        if self.frame_times.len() > 60 {
            self.frame_times.pop_front();
        }
        let avg = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        1.0 / avg.as_secs_f32()
    }
}
```

---

### RQ-8: RTT/Ping Display

**Context**: Need network latency metric for HUD.

**Decision**: Use existing `plix-net` metrics.

**Rationale**:
- `plix-net/src/metrics.rs` already tracks RTT
- Connection struct exposes RTT calculation
- No new implementation needed

**Implementation Notes**:
- Access `connection.metrics().rtt()` from client
- Display in milliseconds
- Update every snapshot received (smoothed value)

---

## Dependencies Summary

| Component | Existing | New | Notes |
|-----------|----------|-----|-------|
| wgpu | Yes | No | Version 23.0 |
| winit | Yes | No | Version 0.30 |
| glam | Yes | No | Vec3, Mat4 |
| Arena loader | Yes | No | plix-arena crate |
| Protocol | Yes | No | WorldSnapshot, PlayerSnapshot |
| RTT metrics | Yes | No | plix-net metrics |
| Text rendering | No | Optional | Window title for MVP |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Voxel mesh too slow | Accept naive approach for MVP, profile if <30 FPS |
| Interpolation jitter | Tune delay parameter, add smoothing |
| HUD blocks rendering | Update title async, throttle updates |
| Headless breaks | Run headless tests in CI before merge |

---

## Open Questions (Resolved)

All questions resolved. No NEEDS CLARIFICATION remaining.
