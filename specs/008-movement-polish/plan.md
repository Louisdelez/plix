# Implementation Plan: Movement Polish

**Branch**: `008-movement-polish` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/008-movement-polish/spec.md`

## Summary

Replace the prototype movement system with a robust, deterministic, server-authoritative physics implementation. The goal is to deliver reliable collision detection, consistent jumping, automatic step-up, proper friction/air control, and smooth network reconciliation for competitive FPS gameplay.

**Key Technical Changes:**
- Refactor `collision.rs` to use capsule collider (not AABB) with proper axis-separated resolution
- Update `movement.rs` constants and physics to match clarified values
- Add step-up logic for voxel terrain navigation
- Ensure client prediction uses identical code path as server
- Implement smooth correction interpolation (within 100ms)

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: glam (math), bincode (serialization), tokio (async), wgpu (client rendering)
**Storage**: N/A (in-memory state only)
**Testing**: cargo test (unit + integration tests in crates/*/tests/)
**Target Platform**: Linux server (headless), Linux/Windows client (wgpu)
**Project Type**: Workspace with multiple crates (plix-common, plix-server, plix-client, plix-arena)
**Performance Goals**: 60 TPS server tick rate, <0.2 block prediction error in 95% samples
**Constraints**: Must not break existing combat/block systems, headless server mode required
**Scale/Scope**: 8 players per match, voxel arenas up to 64x64x64

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ Pass | Server-authoritative architecture, client inputs are validated |
| II. Performance (Low Latency) | ✅ Pass | 60Hz tick rate, deterministic physics, no GC pauses |
| III. Architecture (Engine-First) | ✅ Pass | Core engine primitives in plix-common/plix-server |
| IV. Modding | N/A | No mod API changes in this feature |
| V. Code Quality | ✅ Pass | Full test coverage required, explicit error handling |
| VI. Technical Standards | ✅ Pass | Stable Rust, cargo clippy/fmt enforced |
| VII. Player Experience | ✅ Pass | Smooth movement, no rubber-banding |
| VIII. Open Source | ✅ Pass | All code public, no proprietary deps |
| IX. Scoping & Realism | ✅ Pass | Minimal scope - polish existing system, no new features |
| X. Long-Term Vision | ✅ Pass | Foundation for all future movement-dependent features |

**Gate Result**: ✅ PASS - No violations, proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/008-movement-polish/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal message contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── math.rs          # Vec3, AABB, Rotation (add Capsule)
│       ├── types.rs         # BlockType, PlayerId, etc.
│       └── protocol/
│           └── messages.rs  # ClientMessage, ServerMessage, Snapshot
├── plix-server/
│   └── src/
│       ├── sim/
│       │   ├── movement.rs  # MovementSystem, apply_input (MODIFY)
│       │   ├── collision.rs # CollisionWorld, move_and_slide (REWRITE)
│       │   └── mod.rs
│       ├── session.rs       # PlayerSession state
│       └── lib.rs           # GameServer tick loop
├── plix-client/
│   └── src/
│       ├── prediction.rs    # Client-side prediction (MODIFY)
│       ├── reconciliation.rs # Server correction handling (MODIFY)
│       └── interpolation.rs # Entity interpolation
└── plix-arena/
    └── src/
        └── format.rs        # LoadedArena, block queries
```

**Structure Decision**: Existing workspace structure maintained. Changes primarily in `plix-server/src/sim/` and `plix-client/src/`.

## Complexity Tracking

> No violations to justify - feature uses minimal scope within existing architecture.

## Phases Overview (from user input)

### Phase 1 — Collision Model Rewrite (Foundation)
- Introduce CapsuleCollider (0.4m radius, 1.8m height)
- Implement capsule ↔ voxel collision (AABB per voxel)
- Axis-separated resolution (Y → X → Z)
- Prevent tunneling at max speed

### Phase 2 — Ground Detection & Step-Up
- Reliable `is_grounded` detection
- Step-up (max 0.5 blocks) when grounded
- Fail step-up if head collision

### Phase 3 — Jump System
- Jump impulse calculated from gravity (20 m/s²) and target height (1.25 blocks)
- Jump only when grounded, no double jump

### Phase 4 — Friction & Acceleration Model
- Ground friction for responsive stopping
- Air control capped at 30%
- Speed clamped to 6 m/s

### Phase 5 — Hitbox & Combat Stability
- Single capsule for collision + hit detection
- Snapshot positions post-collision

### Phase 6 — Network Desync Fixes
- Clamp correction deltas
- Smooth interpolation within 100ms
- Identical code on client/server

### Phase 7 — Regression & Validation
- Unit tests (≥20)
- Integration tests (combat + block interaction)
- Load test with bots

## Physics Constants (Clarified)

| Parameter | Value | Source |
|-----------|-------|--------|
| Capsule Height | 1.8m | Clarification session |
| Capsule Radius | 0.4m | Clarification session |
| Movement Speed | 6.0 m/s | Clarification session |
| Gravity | 20.0 m/s² | Clarification session |
| Jump Height | 1.25 blocks | Clarification session |
| Step Height | 0.5 blocks | Spec FR-020 |
| Air Control | 30% | Clarification session |
| Tick Rate | 60 Hz | Existing system |
| Correction Smoothing | ≤100ms | Spec FR-062 |

**Jump Impulse Calculation**:
```
v = sqrt(2 * g * h)
v = sqrt(2 * 20 * 1.25) = sqrt(50) ≈ 7.07 m/s
```

## Definition of Done

- [ ] No player clips through blocks (SC-001)
- [ ] No visible desync in combat (SC-002: <0.2 block error 95%)
- [ ] Jump height consistent within 1% (SC-006)
- [ ] Step-up works on voxel terrain (SC-005)
- [ ] Speed strictly bounded to 6 m/s
- [ ] Server authoritative (FR-003)
- [ ] All tests pass (cargo test)
- [ ] Clippy/fmt clean
