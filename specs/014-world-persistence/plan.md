# Implementation Plan: World Persistence

**Branch**: `014-world-persistence` | **Date**: 2025-12-16 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/014-world-persistence/spec.md`

## Summary

Implement save/load functionality for chunked voxel worlds, supporting both solo and server modes. The system uses per-chunk persistence with delta/diff storage for procedurally generated worlds (saving only modified chunks), format versioning for future compatibility, and crash-safe atomic writes. Server auto-save at configurable intervals (default 5 minutes) minimizes data loss.

## Technical Context

**Language/Version**: Rust 1.75+ (stable channel only per constitution)
**Primary Dependencies**: serde + bincode (existing), tokio (existing async runtime), tracing (existing logging)
**Storage**: File system with per-world directories (`~/.local/share/plix/worlds/`)
**Testing**: cargo test (unit + integration)
**Target Platform**: Linux (primary), cross-platform compatible
**Project Type**: Rust workspace (existing 6 crates)
**Performance Goals**: Save <500ms for typical gameplay, metadata load <50ms
**Constraints**: Non-blocking saves, atomic writes, deterministic chunk generation for delta approach
**Scale/Scope**: Thousands of chunks per world, incremental saves only for modified chunks

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Security (Server Authority) | PASS | Server controls save triggers; client cannot falsify world state |
| II. Performance (Low Latency) | PASS | Async/background saves, incremental dirty-chunk approach, bounded I/O |
| III. Architecture (Engine-First) | PASS | Persistence is a reusable engine primitive, not game-mode specific |
| IV. Modding (Extensibility) | PASS | Versioned format allows future extension; mod data fields reserved |
| V. Code Quality (Tested) | PASS | All persistence operations will have unit + integration tests |
| VI. Technical Standards (Rust) | PASS | Stable Rust, explicit serde, deterministic APIs |
| VII. Player Experience | PASS | Solo same as server (local server pattern); seamless world selection |
| VIII. Open Source | PASS | No proprietary dependencies |
| IX. Scoping (Minimal) | PASS | Core save/load only; no backups, no entity persistence, no compression |
| X. Long-Term Vision | PASS | Format versioning ensures 5+ year compatibility evolution |

**Gate Result**: ALL PASS - Proceed to Phase 0

## Project Structure

### Documentation (this feature)

```text
specs/014-world-persistence/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (internal APIs)
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
crates/
├── plix-common/
│   └── src/
│       ├── chunk.rs           # Existing - add persistence_modified flag
│       ├── world.rs           # Existing - add persistence tracking methods
│       └── persist/           # NEW - Core persistence types and traits
│           ├── mod.rs
│           ├── version.rs     # Format versioning and migration
│           ├── world_meta.rs  # WorldMetadata struct
│           ├── chunk_codec.rs # Chunk encode/decode
│           └── error.rs       # Persistence errors
│
├── plix-server/
│   └── src/
│       ├── persist/           # NEW - Server persistence integration
│       │   ├── mod.rs
│       │   ├── world_store.rs # File I/O operations
│       │   ├── scheduler.rs   # Auto-save scheduler
│       │   └── atomic.rs      # Atomic write utilities
│       └── lib.rs             # Add world load/save triggers
│
└── plix-client/
    └── src/
        └── persist/           # NEW - Client persistence (solo mode)
            ├── mod.rs
            └── local_store.rs # Solo save/load (reuse server logic)

tests/
├── persist_unit/              # Unit tests for codec, version, meta
├── persist_integration/       # Integration tests for save/load cycles
└── persist_crash/             # Crash simulation tests
```

**Structure Decision**: Add `persist` module to existing crates rather than new crate. This follows the existing pattern and keeps persistence close to the types it serializes.

## Architecture Overview

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Application Layer                         │
├─────────────────────────────────────────────────────────────────┤
│  Server (plix-server)          │  Client Solo (plix-client)     │
│  ┌─────────────────────┐       │  ┌─────────────────────┐       │
│  │   SaveScheduler     │       │  │   LocalStore        │       │
│  │   - auto-save timer │       │  │   - save on quit    │       │
│  │   - shutdown flush  │       │  │   - load on start   │       │
│  └─────────┬───────────┘       │  └─────────┬───────────┘       │
│            │                   │            │                   │
│  ┌─────────▼───────────┐       │            │                   │
│  │   WorldStore        │◄──────┼────────────┘                   │
│  │   - create/open     │       │  (reuses WorldStore)           │
│  │   - load_chunk      │       │                                │
│  │   - save_chunk      │       │                                │
│  │   - atomic writes   │       │                                │
│  └─────────┬───────────┘       │                                │
├────────────┼───────────────────┼────────────────────────────────┤
│            │           Core Layer (plix-common)                  │
│  ┌─────────▼───────────────────▼───────────────────────┐        │
│  │                   Persistence Types                  │        │
│  │  ┌──────────────┐  ┌──────────────┐  ┌────────────┐ │        │
│  │  │ WorldMeta    │  │ ChunkCodec   │  │ SaveVersion│ │        │
│  │  │ - name/id    │  │ - encode()   │  │ - current  │ │        │
│  │  │ - seed       │  │ - decode()   │  │ - migrate()│ │        │
│  │  │ - version    │  │ - validate() │  │ - compare()│ │        │
│  │  │ - created_at │  │              │  │            │ │        │
│  │  │ - world_kind │  │              │  │            │ │        │
│  │  └──────────────┘  └──────────────┘  └────────────┘ │        │
│  └─────────────────────────────────────────────────────┘        │
├─────────────────────────────────────────────────────────────────┤
│                        File System                               │
│  ~/.local/share/plix/worlds/                                    │
│  └── <world_id>/                                                │
│      ├── meta.bin          (WorldMetadata)                      │
│      ├── chunks/                                                │
│      │   ├── 0_0_0.bin     (ChunkData for modified chunks)     │
│      │   ├── 1_0_0.bin                                         │
│      │   └── ...                                                │
│      └── temp/             (staging for atomic writes)          │
└─────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### 1. WorldMeta (plix-common/src/persist/world_meta.rs)

Stores world-level metadata loaded before any chunk data.

**Fields**:
- `world_id: String` - Unique identifier (directory name)
- `name: String` - Display name
- `format_version: u32` - Save format version (starts at 1)
- `created_at: u64` - Unix timestamp
- `world_kind: WorldKind` - enum { Generated { seed, config }, Arena, Custom }
- `last_saved: u64` - Unix timestamp of last save

### 2. SaveVersion (plix-common/src/persist/version.rs)

Handles format versioning and migration.

**Constants**:
- `CURRENT_VERSION: u32 = 1`
- `MIN_SUPPORTED_VERSION: u32 = 1`

**Operations**:
- `check_compatibility(version) -> VersionCheck` (Ok, NeedsMigration, TooOld, TooNew)
- `migrate(from, to, data) -> Result<data>` (when migrations defined)

### 3. ChunkCodec (plix-common/src/persist/chunk_codec.rs)

Encodes/decodes chunk data for persistence.

**Format** (v1):
```rust
struct ChunkData {
    coord: ChunkCoord,          // 12 bytes (3 × i32)
    blocks: [BlockType; 4096],  // 4096 bytes (4096 × u8)
}
// Total: ~4108 bytes per chunk (before bincode overhead)
```

**Operations**:
- `encode(chunk) -> Vec<u8>` - Serialize chunk to bytes
- `decode(bytes) -> Result<Chunk>` - Deserialize with validation
- `validate(bytes) -> bool` - Check header/size without full decode

### 4. WorldStore (plix-server/src/persist/world_store.rs)

File I/O operations with atomic writes.

**API**:
```rust
impl WorldStore {
    fn create_world(meta: &WorldMeta) -> Result<Self>
    fn open_world(world_id: &str) -> Result<Self>
    fn load_meta(&self) -> Result<WorldMeta>
    fn save_meta(&self, meta: &WorldMeta) -> Result<()>
    fn load_chunk(&self, coord: ChunkCoord) -> Result<Option<Chunk>>
    fn save_chunk(&self, coord: ChunkCoord, chunk: &Chunk) -> Result<()>
    fn delete_chunk(&self, coord: ChunkCoord) -> Result<()>
    fn list_saved_chunks(&self) -> Result<Vec<ChunkCoord>>
    fn world_path(&self) -> &Path
}
```

**Atomic Write Strategy**:
1. Write to `temp/<random>.bin.tmp`
2. fsync() the temp file
3. rename() to final destination (atomic on POSIX)
4. fsync() parent directory

### 5. SaveScheduler (plix-server/src/persist/scheduler.rs)

Manages periodic auto-saves and shutdown flush.

**Configuration**:
- `auto_save_interval_secs: u64` (default: 300 = 5 minutes)
- `max_chunks_per_cycle: usize` (default: 100, for bounded I/O)

**Operations**:
- `tick()` - Check if auto-save due, perform incremental save
- `flush()` - Save all dirty chunks immediately (shutdown)
- `mark_dirty(coord)` - Add chunk to dirty set
- `clear_dirty(coord)` - Remove after successful save

### 6. Modified Chunk Tracking

**Changes to existing types**:

`Chunk` (plix-common/src/chunk.rs):
- Add `modified_for_persistence: bool` field
- Set to `true` when `set_block()` changes a block
- Separate from `dirty` (mesh rebuild) flag

`ChunkedWorld` (plix-common/src/world.rs):
- Add `persistence_dirty: HashSet<ChunkCoord>`
- Add `mark_persistence_dirty(coord)`
- Add `persistence_dirty_chunks() -> impl Iterator`
- Add `clear_persistence_dirty(coord)`

## Delta/Diff Strategy for Procedural Worlds

**Save Logic**:
1. For each chunk in `persistence_dirty_chunks()`:
   - Save chunk to `chunks/<x>_<y>_<z>.bin`
2. Save updated metadata (last_saved timestamp)

**Load Logic**:
1. Load `WorldMeta` from `meta.bin`
2. If `WorldKind::Generated { seed, config }`:
   - Initialize `ChunkGenerator` with seed
   - For chunk access: check file first, fallback to generate
3. Else (Arena/Custom):
   - Load all saved chunks
   - Error if expected chunk missing

**Result**: Only modified chunks stored; unmodified regenerate from seed.

## Observability

**Metrics** (via tracing):
- `chunks_dirty_for_save` - Gauge of pending saves
- `chunks_saved_total` - Counter of successful saves
- `save_cycles_total` - Counter of auto-save cycles
- `save_failures_total` - Counter of I/O errors
- `save_cycle_duration_ms` - Histogram of save times

**Logs**:
- INFO: World open/create, auto-save start/complete, migration applied
- WARN: Chunk corruption detected, version mismatch
- ERROR: I/O failures, migration failure

## Complexity Tracking

No constitution violations to justify.
