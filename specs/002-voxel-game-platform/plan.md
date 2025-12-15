# Implementation Plan: Voxel Game Platform - Visual Multiplayer

**Branch**: `002-voxel-game-platform` | **Date**: 2025-12-14 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-voxel-game-platform/spec.md`

## Summary

Transform the existing prototype (working authoritative networking + placeholder scene) into a visually playable voxel platform where the client renders the arena geometry and displays players from network snapshots. The goal is to validate the multiplayer visual loop (arena + players + interpolation + HUD debug) without adding product features (no CEF, no server browser, no accounts).

**Primary Requirements**:
1. Render the static voxel arena in the client window
2. Visualize local and remote players from snapshot data
3. Implement smooth interpolation for remote player movement
4. Display debug HUD (FPS, ping/RTT, player_id, round state)
5. Maintain headless mode and existing tests

**Technical Approach**:
- Extend existing wgpu rendering pipeline in `plix-client/src/render/`
- Use existing arena data from `plix-arena` (TOML format with block types)
- Consume existing `WorldSnapshot` and `PlayerSnapshot` from protocol
- Implement snapshot buffering and interpolation in client
- Add simple text/overlay HUD using wgpu_text or window title fallback

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**:
- wgpu 23.0 (rendering)
- winit 0.30 (windowing)
- tokio (async runtime)
- glam (math/vectors)
- bincode (serialization)

**Storage**: N/A (arenas loaded from TOML files in `assets/arenas/`)
**Testing**: cargo test (workspace-wide)
**Target Platform**: Linux (primary), cross-platform via wgpu
**Project Type**: Workspace with 6 crates

**Performance Goals**:
- Minimum 30 FPS with arena up to 10,000 blocks
- Smooth interpolation (no visible teleportation)
- HUD updates without impacting render performance

**Constraints**:
- Server tick rate: 60 Hz
- Max players: 16
- Headless mode must remain functional (no window dependency)
- No modifications to core networking (Net → State → Render pipeline)

**Scale/Scope**:
- Arena size up to 32x16x32 blocks (test arena)
- 2-16 concurrent players visualized
- Single arena per session (no streaming)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Status | Notes |
|-----------|-------------|--------|-------|
| **I. Security** | Client never trusted for game state | PASS | Visualization only consumes server snapshots |
| **I. Security** | Server authoritative architecture | PASS | No state invention, display only |
| **II. Performance** | Network priority over features | PASS | No feature creep, visualization only |
| **II. Performance** | Separation of concerns | PASS | Render separate from Net/State |
| **III. Architecture** | Strict layer separation | PASS | Net → State → Render pipeline preserved |
| **V. Code Quality** | Mandatory testing | CHECK | Must add render tests if applicable |
| **VI. Technical** | Stable Rust only | PASS | No nightly features |
| **VI. Technical** | cargo clippy/fmt compliance | CHECK | Must verify before merge |
| **VII. Player Experience** | Responsive UI | PASS | HUD must not block rendering |
| **IX. Scoping** | Minimal MVP | PASS | Strictly visualization, no feature creep |
| **IX. Scoping** | No feature creep | PASS | Explicit out-of-scope boundaries |

**Gate Result**: PASS - No violations. Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/002-voxel-game-platform/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/           # Shared types and protocol
│   └── src/
│       ├── types.rs       # PlayerId, BlockType, etc.
│       ├── math.rs        # Vec3, Rotation, AABB
│       └── protocol/      # Messages, snapshots, events
│           └── messages.rs
│
├── plix-net/              # UDP networking layer
│   └── src/
│       └── metrics.rs     # RTT measurement (for HUD)
│
├── plix-arena/            # Arena loading
│   └── src/
│       ├── format.rs      # Arena definition
│       └── loader.rs      # TOML parsing
│
├── plix-server/           # Authoritative server (unchanged)
│   └── src/
│       └── ...
│
├── plix-client/           # Game client (MAIN CHANGES)
│   └── src/
│       ├── main.rs        # Entry point (windowed/headless)
│       ├── render/
│       │   ├── mod.rs
│       │   ├── engine.rs  # wgpu pipeline
│       │   ├── camera.rs  # FPS camera
│       │   ├── voxels.rs  # Arena mesh (TO IMPLEMENT)
│       │   ├── players.rs # Player rendering (TO IMPLEMENT)
│       │   └── hud.rs     # Debug HUD (TO CREATE)
│       ├── interpolation.rs # Remote player smoothing (TO EXTEND)
│       └── net.rs         # Network state
│
└── plix-tools/            # Load test bots (unchanged)
    └── src/
        └── ...

assets/
└── arenas/
    └── test_arena.toml    # Test arena definition
```

**Structure Decision**: Existing workspace structure is maintained. Changes concentrated in `plix-client/src/render/` with new modules for voxel meshing, player rendering, and HUD. No new crates needed.

## Complexity Tracking

> No violations requiring justification. Feature is strictly scoped to visualization.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |

## Implementation Phases

### Phase 1: Arena Voxel Rendering (Static)

**Objective**: Display the arena geometry in the client window.

**Tasks**:
1. Load arena data in client (from local TOML or server payload)
2. Generate mesh from block data (simple cube-per-block initially)
3. Render arena mesh with distinct colors for block types
4. Verify camera navigation works with arena visible

**Deliverable**: Arena visible in windowed mode, camera controls work.

### Phase 2: Player Visualization (Local + Remote)

**Objective**: See players moving in the arena.

**Tasks**:
1. Define simple player visual (capsule/cube mesh)
2. Map PlayerId to rendered entity
3. Update transforms from snapshots
4. Distinguish local vs remote players visually

**Deliverable**: Two connected clients see each other in arena.

### Phase 3: Interpolation (Fluidity)

**Objective**: Smooth remote player movement.

**Tasks**:
1. Buffer incoming snapshots (timeline)
2. Interpolate position/rotation at render time (slightly delayed)
3. Handle missing snapshots gracefully (hold last position)
4. Apply interpolation to display only (not authoritative state)

**Deliverable**: Remote players move without visible teleportation.

### Phase 4: Debug HUD

**Objective**: Display diagnostic information.

**Tasks**:
1. Display FPS counter
2. Display ping/RTT from network metrics
3. Display player_id
4. Display round state and timer

**Deliverable**: HUD visible with live-updating values.

### Phase 5: Validation & Non-Regression

**Objective**: Ensure no breakage.

**Tasks**:
1. `cargo test --workspace` passes
2. Headless mode still works
3. Load test scripts still function
4. Manual test: 2 windowed clients connected

**Deliverable**: All tests green, load test functional.
