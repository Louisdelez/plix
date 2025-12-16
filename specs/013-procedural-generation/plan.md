# Implementation Plan: Procedural Generation v1

**Branch**: `013-procedural-generation` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/013-procedural-generation/spec.md`

## Summary

Implement deterministic procedural world generation for the plix voxel game platform. The system generates terrain from a seed using noise-based heightmaps and a basic 3-biome system (plains, mountains, desert). Generation is per-chunk independent (no neighbor dependencies), enabling efficient multiplayer chunk streaming. Key technical approach: use `noise-rs` crate for deterministic Perlin/Simplex noise, pure functional generation (seed + coord → chunk), and integration with existing ChunkedWorld/ChunkManager systems from Features 011/012.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: `noise-rs` (noise generation), `glam` (math), existing plix-common types
**Storage**: N/A (in-memory chunk generation, no persistence in this feature)
**Testing**: `cargo test` with determinism, continuity, and layer validation tests
**Target Platform**: Linux server + client (cross-platform Rust)
**Project Type**: Rust workspace - new module in `plix-common` crate
**Performance Goals**: <10ms per chunk generation, 512 chunks in <5s for initial spawn
**Constraints**: CPU-only generation, no GPU dependencies, deterministic cross-platform
**Scale/Scope**: Infinite world potential, practical testing at 1000 chunks

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Requirement | Compliance |
|-----------|-------------|------------|
| I. Security | Server authoritative | PASS - Generation callable from server (authoritative) and client (prediction only) |
| II. Performance | Stable tick rate, no blocking | PASS - Generation bounded per chunk, integrates with existing budget system |
| II. Performance | Deterministic multithreading | PASS - Pure function (seed + coord → chunk), no shared mutable state |
| III. Architecture | Engine-first modularity | PASS - WorldGenerator is a primitive, gameplay builds on it |
| V. Code Quality | Mandatory testing | PASS - Determinism, continuity, and layer tests specified |
| V. Code Quality | No panics in production | PASS - Handle edge cases (extreme coords, seed extremes) |
| VI. Technical Standards | Stable Rust only | PASS - noise-rs works on stable Rust |
| VI. Technical Standards | Deterministic APIs | PASS - Core requirement of this feature |
| VI. Technical Standards | Tooling compliance | PASS - Will run clippy/fmt |
| VII. Player Experience | Multiplayer priority | PASS - Per-chunk independence enables multiplayer streaming |
| IX. Scoping | Minimal MVP | PASS - 3 biomes, simple layers, no caves/structures |

**Gate Status**: PASS - No violations. Proceeding to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/013-procedural-generation/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── lib.rs           # Re-export worldgen module
│       ├── types.rs         # Extend BlockType with new variants
│       ├── chunk.rs         # Existing chunk types (Feature 011)
│       ├── world.rs         # Existing ChunkedWorld (Feature 011)
│       └── worldgen/        # NEW: World generation module
│           ├── mod.rs       # Module exports
│           ├── config.rs    # WorldGenConfig, BiomeConfig
│           ├── noise.rs     # NoiseSource wrapper for noise-rs
│           ├── biome.rs     # Biome enum, BiomeModel
│           ├── height.rs    # HeightModel (2D heightmap sampling)
│           └── generator.rs # ChunkGenerator (main generation logic)
│
├── plix-server/
│   └── src/
│       └── world.rs         # Integration: generate chunks on demand
│
└── plix-client/
    └── src/
        └── chunk_manager.rs # Integration: generate chunks for prediction
```

**Structure Decision**: World generation logic goes in `plix-common` since both server and client need it. This follows the existing pattern where shared types and logic live in plix-common.

## Complexity Tracking

> No violations requiring justification. Design is minimal:
> - Single new module (worldgen) in existing crate
> - 5 source files for clean separation
> - No new crates, no new dependencies beyond noise-rs
