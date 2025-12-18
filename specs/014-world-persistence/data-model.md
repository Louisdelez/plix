# Data Model: World Persistence

**Feature**: 014-world-persistence
**Date**: 2025-12-16

## Overview

This document defines the data structures for world persistence. All types are designed for bincode serialization with explicit versioning for forward compatibility.

---

## Core Entities

### WorldMetadata

Top-level metadata for a persistent world. Loaded first to check compatibility before loading chunks.

```rust
/// World metadata stored in `meta.bin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldMetadata {
    /// Format version for compatibility checking
    pub format_version: u32,

    /// Unique identifier (also directory name)
    pub world_id: String,

    /// Human-readable display name
    pub name: String,

    /// World generation type
    pub world_kind: WorldKind,

    /// Creation timestamp (Unix seconds)
    pub created_at: u64,

    /// Last save timestamp (Unix seconds)
    pub last_saved: u64,
}
```

**Validation Rules**:
- `format_version` must be >= `MIN_SUPPORTED_VERSION` and <= `CURRENT_VERSION`
- `world_id` must be non-empty, valid filesystem name (alphanumeric, underscore, hyphen)
- `name` must be non-empty, max 64 characters
- `created_at` <= `last_saved`

---

### WorldKind

Discriminator for world generation type, determining load behavior.

```rust
/// How the world was created (affects load strategy)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorldKind {
    /// Procedurally generated world (regenerate unmodified chunks)
    Generated {
        /// Seed for deterministic generation
        seed: u64,
        /// Generation config (optional, for non-default settings)
        config: Option<WorldGenConfig>,
    },

    /// Imported from arena definition
    Arena {
        /// Original arena file name
        arena_name: String,
    },

    /// Manually created or imported (all chunks must be saved)
    Custom,
}
```

**Load Behavior by Kind**:

| Kind | Missing Chunk | Saved Chunk |
|------|---------------|-------------|
| Generated | Regenerate from seed | Load from file |
| Arena | Error (required) | Load from file |
| Custom | Error (required) | Load from file |

---

### ChunkData

Serializable chunk data for file storage.

```rust
/// Chunk data stored in `chunks/<x>_<y>_<z>.bin`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkData {
    /// Data format version (for per-chunk migration if needed)
    pub version: u8,

    /// Chunk coordinates
    pub coord: ChunkCoord,

    /// Block data (dense array, 16×16×16)
    pub blocks: [BlockType; CHUNK_BLOCK_COUNT],
}
```

**Size**: ~4109 bytes per chunk (bincode overhead minimal)

**Validation Rules**:
- `version` must be supported
- `blocks.len()` must equal `CHUNK_BLOCK_COUNT` (4096)
- Block types must be valid (0-255 range)

---

### SaveVersion

Version management for compatibility.

```rust
/// Current save format version
pub const CURRENT_VERSION: u32 = 1;

/// Minimum supported version for loading
pub const MIN_SUPPORTED_VERSION: u32 = 1;

/// Result of version compatibility check
#[derive(Debug, Clone, PartialEq)]
pub enum VersionCheck {
    /// Version matches current
    Current,

    /// Version is older but supported (migration available)
    NeedsMigration(u32),

    /// Version is too old (no migration path)
    TooOld(u32),

    /// Version is newer than current (from future game version)
    TooNew(u32),
}
```

---

### PersistError

Error types for persistence operations.

```rust
/// Errors that can occur during persistence operations
#[derive(Debug, Clone)]
pub enum PersistError {
    /// File system I/O error
    Io(String),

    /// Serialization/deserialization error
    Codec(String),

    /// World directory not found
    WorldNotFound(String),

    /// Version incompatibility
    VersionMismatch {
        world_version: u32,
        reason: VersionCheck,
    },

    /// Chunk data corrupted or invalid
    ChunkCorrupted {
        coord: ChunkCoord,
        reason: String,
    },

    /// Metadata corrupted or invalid
    MetadataCorrupted(String),

    /// Permission denied
    PermissionDenied(String),

    /// Disk full
    DiskFull,
}
```

---

## Runtime Entities

### WorldStore

Handle to an open world directory.

```rust
/// Handle to a world directory for I/O operations
pub struct WorldStore {
    /// Path to world directory
    path: PathBuf,

    /// Cached metadata (updated on save)
    metadata: WorldMetadata,
}
```

**Invariants**:
- `path` points to valid directory with `meta.bin`
- `metadata.world_id` matches directory name
- `chunks/` subdirectory exists

---

### SaveScheduler

Auto-save state and configuration.

```rust
/// Configuration for auto-save behavior
#[derive(Debug, Clone)]
pub struct SaveSchedulerConfig {
    /// Time between auto-saves
    pub interval: Duration,

    /// Maximum chunks to save per cycle (bounded I/O)
    pub max_chunks_per_cycle: usize,

    /// Whether auto-save is enabled
    pub enabled: bool,
}

/// Runtime state for save scheduling
pub struct SaveScheduler {
    config: SaveSchedulerConfig,
    last_save: Instant,
    metrics: SaveMetrics,
}

/// Metrics tracked during save operations
#[derive(Debug, Default, Clone)]
pub struct SaveMetrics {
    pub chunks_saved_total: u64,
    pub chunks_saved_last_cycle: u32,
    pub save_cycles_total: u64,
    pub save_failures_total: u64,
    pub last_cycle_duration_ms: u64,
}
```

**Default Configuration**:
- `interval`: 5 minutes (300 seconds)
- `max_chunks_per_cycle`: 100
- `enabled`: true for server, configurable for solo

---

## Relationships

```text
WorldMetadata 1 ──────────────── 1 WorldStore
     │
     │ world_kind
     ▼
WorldKind ◄──── Generated { seed, config }
          ◄──── Arena { arena_name }
          ◄──── Custom

WorldStore 1 ──────────────── * ChunkData
                                  │
                                  │ coord
                                  ▼
                            ChunkCoord (i32, i32, i32)
```

---

## State Transitions

### Chunk Persistence States

```text
                    ┌─────────────────────────────────────────┐
                    │                                         │
                    ▼                                         │
    ┌───────────┐   generate   ┌───────────┐   modify   ┌───────────┐
    │ NotLoaded │ ───────────► │ Generated │ ─────────► │  Dirty    │
    └───────────┘              └───────────┘            └───────────┘
         │                           │                       │
         │                           │                       │ save
         │ load from file            │ load from file        │
         │                           │                       ▼
         │                           │                 ┌───────────┐
         └───────────────────────────┴────────────────►│   Saved   │
                                                       └───────────┘
                                                             │
                                                             │ modify
                                                             │
                                                             └──► Dirty
```

**State Definitions**:
- **NotLoaded**: Chunk not in memory
- **Generated**: Chunk generated from seed, not yet modified
- **Dirty**: Chunk has been modified, needs saving
- **Saved**: Chunk modifications persisted to disk

---

## File Format Specifications

### meta.bin

```
┌────────────────────────────────────────┐
│ bincode-encoded WorldMetadata          │
│                                        │
│ ┌──────────────┬─────────────────────┐ │
│ │ format_version │ u32 (4 bytes)      │ │
│ ├──────────────┼─────────────────────┤ │
│ │ world_id      │ String (len + UTF8) │ │
│ ├──────────────┼─────────────────────┤ │
│ │ name          │ String (len + UTF8) │ │
│ ├──────────────┼─────────────────────┤ │
│ │ world_kind    │ enum (tag + data)   │ │
│ ├──────────────┼─────────────────────┤ │
│ │ created_at    │ u64 (8 bytes)       │ │
│ ├──────────────┼─────────────────────┤ │
│ │ last_saved    │ u64 (8 bytes)       │ │
│ └──────────────┴─────────────────────┘ │
└────────────────────────────────────────┘

Typical size: ~100-200 bytes
```

### chunks/<x>_<y>_<z>.bin

```
┌────────────────────────────────────────┐
│ bincode-encoded ChunkData              │
│                                        │
│ ┌──────────────┬─────────────────────┐ │
│ │ version       │ u8 (1 byte)         │ │
│ ├──────────────┼─────────────────────┤ │
│ │ coord.x       │ i32 (4 bytes)       │ │
│ ├──────────────┼─────────────────────┤ │
│ │ coord.y       │ i32 (4 bytes)       │ │
│ ├──────────────┼─────────────────────┤ │
│ │ coord.z       │ i32 (4 bytes)       │ │
│ ├──────────────┼─────────────────────┤ │
│ │ blocks[4096]  │ [u8; 4096] (4096 B) │ │
│ └──────────────┴─────────────────────┘ │
└────────────────────────────────────────┘

Fixed size: ~4109 bytes
```

---

## Indexes and Lookups

### World Discovery

To list available worlds:
1. Enumerate directories in `~/.local/share/plix/worlds/`
2. For each directory, load `meta.bin`
3. Return list of `WorldMetadata` (or error indicator for corrupted)

**Performance**: O(n) where n = number of worlds, each metadata ~100-200 bytes

### Chunk Lookup

To find if a chunk is saved:
1. Check if `chunks/<x>_<y>_<z>.bin` exists
2. Filename format: `{x}_{y}_{z}.bin` (signed integers with negative sign)

**Examples**:
- Chunk (0, 0, 0) → `0_0_0.bin`
- Chunk (-1, 5, 3) → `-1_5_3.bin`
- Chunk (100, -20, 7) → `100_-20_7.bin`

---

## Migration Strategy

### Version 1 (Initial)

No migrations needed - this is the first version.

### Future Migrations

When `CURRENT_VERSION` increases:

1. Add migration function:
   ```rust
   fn migrate_v1_to_v2(old: ChunkDataV1) -> ChunkDataV2 {
       // Transform fields
   }
   ```

2. Update `MIN_SUPPORTED_VERSION` if old versions unsupported

3. Apply migration during load:
   ```rust
   fn load_chunk(&self, coord: ChunkCoord) -> Result<Chunk, PersistError> {
       let data = self.read_chunk_file(coord)?;
       let chunk_data = match data.version {
           1 => migrate_v1_to_current(data)?,
           CURRENT_CHUNK_VERSION => data,
           v => return Err(PersistError::VersionMismatch { ... }),
       };
       Ok(chunk_data.into_chunk())
   }
   ```
