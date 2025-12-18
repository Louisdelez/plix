# Implementation Plan: Weapons & Items v1

**Branch**: `022-weapons-items-v1` | **Date**: 2025-12-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/022-weapons-items-v1/spec.md`

## Summary

Implement a unified weapon system integrating melee (sword) and ranged (bow) combat with the existing hotbar from Feature 021. The sword uses cone-based hit detection (60°, 2.5 blocks, 25 damage, 0.6s cooldown). The bow fires server-side projectile entities (15 damage, 0.8s cooldown, max 128 projectiles) with spread/recoil mechanics. All combat is server-authoritative with cooldown enforcement and event-based replication.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: glam (math), bincode (serialization), tokio (async), existing plix-common/plix-server crates
**Storage**: N/A (in-memory state only - projectiles, cooldowns, recoil state)
**Testing**: cargo test (unit + integration tests)
**Target Platform**: Linux server (headless), clients on desktop
**Project Type**: Rust workspace (multi-crate)
**Performance Goals**: 60 TPS tick rate, max 128 concurrent projectiles, O(n) collision checks
**Constraints**: Server-authoritative combat, event-driven replication (no per-tick projectile updates)
**Scale/Scope**: Up to 16 players per server, 128 max projectiles

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | All combat validation server-side, cooldowns server-enforced |
| II. Performance (Low Latency) | ✅ PASS | Event-driven replication, bounded projectiles (128 max) |
| III. Architecture (Engine-First) | ✅ PASS | Extends existing item system, uses engine primitives |
| IV. Modding (First-Class) | ✅ PASS | WeaponDef as static data, future-extensible via data mods |
| V. Code Quality | ✅ PASS | Mandatory tests for combat/collision logic |
| VI. Technical Standards | ✅ PASS | Stable Rust, clippy/fmt compliance |
| VII. Player Experience | ✅ PASS | Multiplayer-first design, responsive combat |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping & Realism | ✅ PASS | Minimal MVP (2 weapons, no ammo system) |
| X. Long-Term Vision | ✅ PASS | Weapon system designed for extensibility |

## Project Structure

### Documentation (this feature)

```text
specs/022-weapons-items-v1/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (protocol events)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── types.rs              # Add ProjectileId, WeaponType
│       ├── inventory/
│       │   └── item.rs           # Extend ItemKind with weapon subtype
│       └── protocol/
│           └── messages.rs       # Add projectile events
│
├── plix-server/
│   └── src/
│       ├── weapons/              # NEW MODULE
│       │   ├── mod.rs            # Module exports
│       │   ├── defs.rs           # WeaponDef constants
│       │   ├── cooldown.rs       # CooldownTracker per player/weapon
│       │   ├── melee.rs          # MeleeSystem (cone hit detection)
│       │   ├── ranged.rs         # RangedSystem (projectile spawn, spread)
│       │   ├── projectiles.rs    # ProjectileManager (tick, collisions)
│       │   └── recoil.rs         # RecoilState per player
│       ├── inventory/
│       │   └── use_system.rs     # Route UseActiveItem → weapon systems
│       └── session.rs            # Add weapon state to player session
│
└── plix-server/
    └── tests/
        ├── melee_combat_test.rs       # Sword hit detection tests
        ├── ranged_combat_test.rs      # Bow/projectile tests
        ├── cooldown_test.rs           # Cooldown enforcement tests
        ├── projectile_limit_test.rs   # 128 limit tests
        └── spread_recoil_test.rs      # Accuracy/recoil tests
```

**Structure Decision**: Extends existing Rust workspace structure. New `weapons/` module in plix-server containing all combat logic. Types shared via plix-common.

## Complexity Tracking

No violations - design aligns with constitution principles.

## Phase 0: Research Summary

### Decision 1: Projectile Storage
- **Decision**: Use `Vec<Projectile>` with slot reuse via generation IDs
- **Rationale**: Simple, cache-friendly for 128 max projectiles, O(n) iteration acceptable
- **Alternatives**: HashMap (slower iteration), Slab crate (external dependency)

### Decision 2: Collision Detection
- **Decision**: Sphere-vs-capsule for players, discrete stepping for blocks
- **Rationale**: Simple, accurate enough for v1, matches existing hitbox model
- **Alternatives**: Continuous raycast (complex), spatial grid (overkill for 128 projectiles)

### Decision 3: Spread Calculation
- **Decision**: Server-side random offset within cone, seeded by tick for reproducibility
- **Rationale**: Server-authoritative, prevents client manipulation
- **Alternatives**: Client-side spread (cheat-vulnerable)

### Decision 4: Recoil Model
- **Decision**: Additive spread penalty with decay over ticks
- **Rationale**: Simple, tunable, no client state required
- **Alternatives**: Camera recoil (visual only, out of scope for v1)

## Phase 1: Design Artifacts

See generated files:
- `data-model.md` - Entity definitions
- `contracts/` - Protocol messages
- `quickstart.md` - Integration guide
