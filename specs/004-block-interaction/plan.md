# Implementation Plan: Server-Authoritative Block Interaction

**Branch**: `004-block-interaction` | **Date**: 2025-12-15 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-block-interaction/spec.md`

## Summary

Add server-authoritative block interactions (place/remove) to the voxel world, enabling multiplayer interactivity while maintaining the existing competitive/low-latency architecture. The server validates and applies all block edits, broadcasting reliable events to all clients. Late joiners receive current world state on connect. Client renders mesh updates on confirmed edits only.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only)
**Primary Dependencies**: glam (math), bincode (serialization), wgpu (rendering), tokio (async), winit (input)
**Storage**: N/A (in-memory state only, no persistence)
**Testing**: cargo test (unit/integration tests in crates/plix-server/tests/, crates/plix-tools/tests/)
**Target Platform**: Linux (desktop client + headless server)
**Project Type**: Multi-crate workspace (plix-common, plix-net, plix-server, plix-client, plix-arena, plix-tools)
**Performance Goals**: 60 tick server, <200ms edit visibility, no frame drops >50ms on mesh update
**Constraints**: Server-authoritative, no client trust, maintain headless mode + load tests
**Scale/Scope**: Arena-bounded world (small fixed size), 2-8 players typical

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | ✅ PASS | All block edits validated server-side; client sends requests only |
| II. Performance (Low Latency) | ✅ PASS | Edit processing within single tick; mesh updates bounded |
| III. Architecture (Engine-First) | ✅ PASS | Extends existing primitives (protocol, events, arena) |
| IV. Modding | N/A | No mod API changes in this feature |
| V. Code Quality | ✅ PASS | Unit tests required for validation logic |
| VI. Technical Standards | ✅ PASS | Stable Rust, bincode serialization, versioned protocol |
| VII. Player Experience | ✅ PASS | Real-time feedback, responsive UI |
| VIII. Open Source | ✅ PASS | No proprietary dependencies |
| IX. Scoping & Realism | ✅ PASS | MVP scope: single block type, no inventory |
| X. Long-Term Vision | ✅ PASS | Extends existing patterns, no tech debt |

**Gate Status**: PASS - No violations requiring justification.

## Project Structure

### Documentation (this feature)

```text
specs/004-block-interaction/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── block-protocol.md
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/src/
│   ├── protocol/
│   │   └── messages.rs      # Add BlockEditRequest, BlockEditApplied, BlockEditRejected
│   └── types.rs             # BlockPos, BlockType already exist
├── plix-server/src/
│   ├── sim/
│   │   ├── combat.rs        # Existing combat (reference pattern)
│   │   └── block_edit.rs    # NEW: Block edit validation + application
│   ├── replication/
│   │   └── events.rs        # Extend with block edit events
│   └── lib.rs               # Process block edits in tick loop
├── plix-client/src/
│   ├── input.rs             # Add block action inputs (place/remove)
│   ├── world.rs             # NEW: Mutable client world state
│   ├── raycast.rs           # NEW: Camera raycast for targeting
│   └── render/
│       └── voxels.rs        # Update mesh on block edits
└── plix-arena/src/
    └── loaded.rs            # Ensure arena supports mutation

tests/
└── crates/plix-server/tests/
    └── block_edit_test.rs   # NEW: Validation + replication tests
```

**Structure Decision**: Extends existing multi-crate workspace. Block editing follows combat system pattern: validation in `sim/`, events in `replication/`, client input in `input.rs`.

## Architecture Overview

### Data Flow

```
Client                          Server                          Other Clients
  │                               │                                  │
  │ [Player aims at block]        │                                  │
  │ [DDA raycast → target pos]    │                                  │
  │                               │                                  │
  ├──BlockEditRequest────────────►│                                  │
  │  {kind, pos, block_type}      │                                  │
  │                               │ [Validate]                       │
  │                               │  - bounds                        │
  │                               │  - range                         │
  │                               │  - cell state                    │
  │                               │  - rate limit                    │
  │                               │  - player collision              │
  │                               │                                  │
  │                               │ [If valid: apply to world]       │
  │                               │                                  │
  │◄──BlockEditApplied───────────┼──BlockEditApplied────────────────►│
  │   {pos, new_block, tick}      │   {pos, new_block, tick}         │
  │                               │                                  │
  │ [Update client world]         │                          [Update client world]
  │ [Rebuild chunk mesh]          │                          [Rebuild chunk mesh]
  │ [Show "Block placed"]         │                                  │
  │                               │                                  │
  │◄──BlockEditRejected──────────┤                                  │
  │   {reason} (on failure)       │                                  │
```

### Late Join Flow

```
New Client                      Server
    │                             │
    ├──Connect──────────────────►│
    │                             │
    │◄──Connected────────────────┤
    │   {arena_data: CURRENT}     │  ← Arena data includes all edits
    │                             │
    │ [Load arena with edits]     │
    │ [Build mesh]                │
```

**Decision**: Arena data sent on connect already includes current state. No separate edit log needed for MVP - server sends full current world state.

## Implementation Phases

### Phase 1: Protocol & Data Model

1. **Extend protocol messages** (`plix-common/src/protocol/messages.rs`)
   - Add `BlockEditRequest` to `ClientMessage` enum
   - Add `BlockEditApplied`, `BlockEditRejected` to `ServerMessage`/`GameEvent`
   - Define `BlockEditKind::Place | Remove`
   - Define `BlockEditRejectReason` enum

2. **Ensure arena mutability** (`plix-arena`)
   - Verify `LoadedArena` can mutate block data
   - Add `set_block(pos, block_type)` method if missing

### Phase 2: Server Block Edit System

3. **Create block edit validation** (`plix-server/src/sim/block_edit.rs`)
   - `BlockEditSystem` struct with validation methods
   - Constants: `MAX_EDIT_RANGE = 5.0`, `EDIT_COOLDOWN_TICKS = 15` (4/sec at 60Hz)
   - Validation checks:
     - `is_in_bounds(pos, arena)` → bool
     - `is_in_range(pos, player_pos)` → bool
     - `is_valid_remove(pos, arena)` → bool (cell not air)
     - `is_valid_place(pos, arena, players)` → bool (cell air, no player collision)
     - `is_rate_limited(player, tick)` → bool

4. **Track edit cooldown per player** (`plix-server/src/session.rs`)
   - Add `last_edit_tick: Option<Tick>` to `ServerPlayer`

5. **Integrate into tick loop** (`plix-server/src/lib.rs`)
   - Collect block edit requests from input queue
   - Validate using `BlockEditSystem`
   - Apply valid edits to `LoadedArena`
   - Broadcast `BlockEditApplied` to all clients
   - Send `BlockEditRejected` to requester only

6. **Ensure late join correctness**
   - Verify `arena_data` in `Connected` message uses current arena state
   - Test: connect client after edits, verify world matches

### Phase 3: Client Targeting

7. **Implement DDA raycast** (`plix-client/src/raycast.rs`)
   - `raycast_blocks(origin, direction, max_dist, arena)` → `Option<RaycastHit>`
   - `RaycastHit { block_pos, face_normal, distance }`
   - Use camera position + forward direction

8. **Add block action input** (`plix-client/src/input.rs`)
   - Track `remove_block: bool`, `place_block: bool`
   - Key bindings: LMB = remove, RMB = place (or configurable)
   - Extend `PlayerInput` or send separate `BlockEditRequest`

9. **Send edit requests** (`plix-client/src/lib.rs`)
   - On block action input:
     - Raycast to get target
     - For remove: use hit block position
     - For place: use adjacent cell (hit pos + face normal)
     - Send `BlockEditRequest` to server

### Phase 4: Client World & Rendering

10. **Create mutable client world** (`plix-client/src/world.rs`)
    - `ClientWorld` wrapping arena data
    - `apply_edit(pos, block_type)` method
    - Initialize from `Connected` arena data

11. **Handle edit events** (`plix-client/src/lib.rs`)
    - On `BlockEditApplied`: update `ClientWorld`, mark mesh dirty
    - On `BlockEditRejected`: show debug message

12. **Update voxel renderer** (`plix-client/src/render/voxels.rs`)
    - On dirty flag: rebuild affected chunk mesh
    - MVP: rebuild entire mesh (arena is small)
    - Ensure no frame freeze (bound work per frame if needed)

13. **Debug HUD feedback**
    - Display brief messages: "Block placed", "Block removed", "Edit rejected: {reason}"

### Phase 5: Testing & Validation

14. **Unit tests** (`crates/plix-server/tests/block_edit_test.rs`)
    - Test each validation rule independently
    - Test cooldown enforcement
    - Test valid edit application

15. **Integration tests**
    - Two clients see same world after edits
    - Late joiner sees correct state
    - Invalid edits rejected with correct reason

16. **Non-regression**
    - `cargo test --workspace` passes
    - `./scripts/run_load_test.sh` passes (bots ignore edits)
    - Headless server mode unchanged

## Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Edit transport | Separate message vs PlayerInput extension | Use `BlockEditRequest` message for clarity and reliable channel |
| Late join sync | Full arena state vs delta log | Full state on connect - simpler, arena is small |
| Mesh update | Per-chunk vs full rebuild | Full rebuild for MVP - arena is small, correctness first |
| Block type | Single default vs selection | Single type (Stone) - no inventory in scope |
| Rate limit | Per player | 4 edits/sec (15 tick cooldown at 60Hz) |
| Interaction range | 5 blocks | Matches typical Minecraft-style gameplay |

## Milestones

- **M1**: Protocol + server validation + edit events broadcasting
- **M2**: Client raycast targeting + sending requests
- **M3**: Client applies edits + mesh updates correctly
- **M4**: Late joiner correctness validated
- **M5**: Tests green + load test passes

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Mesh rebuild causes frame drops | Bound work per frame; arena is small enough for full rebuild |
| Late join desync | Use existing arena data path; test explicitly |
| Load test regression | Bots don't send block edits; existing behavior unchanged |
| Protocol version mismatch | Increment protocol version; test client/server compatibility |
