# Quickstart: World Persistence

**Feature**: 014-world-persistence
**Date**: 2025-12-16

## Overview

This guide covers implementing world persistence for plix. After completing this feature, worlds can be saved to disk and reloaded across game sessions, supporting both solo play and persistent servers.

---

## Prerequisites

Before starting:
- [ ] Understand chunked world system (Feature 011)
- [ ] Understand procedural generation (Feature 013)
- [ ] Review existing serde/bincode usage in protocol

---

## Implementation Order

### Phase 1: Core Types (plix-common)

**Estimated files**: 5 new files in `crates/plix-common/src/persist/`

1. **error.rs** - Define `PersistError` enum
2. **version.rs** - Define `CURRENT_VERSION`, `VersionCheck`, comparison logic
3. **world_meta.rs** - Define `WorldMetadata`, `WorldKind` structs
4. **chunk_codec.rs** - Define `ChunkData`, encode/decode functions
5. **mod.rs** - Re-export public types

**Validation**: Unit tests pass for all types

### Phase 2: Chunk Tracking (plix-common)

**Estimated changes**: 2 existing files

1. **chunk.rs** - Add `modified_for_persistence` field to `Chunk`
2. **world.rs** - Add `persistence_dirty` HashSet and methods to `ChunkedWorld`

**Validation**: Existing tests still pass, new tracking tests pass

### Phase 3: Storage (plix-server)

**Estimated files**: 3 new files in `crates/plix-server/src/persist/`

1. **atomic.rs** - Atomic write utilities (temp file + rename)
2. **world_store.rs** - WorldStore struct with create/open/load/save
3. **mod.rs** - Re-export public types

**Validation**: Integration test: create world → save chunk → reload → verify

### Phase 4: Auto-Save (plix-server)

**Estimated files**: 1 new file

1. **scheduler.rs** - SaveScheduler with tick/flush logic

**Integration**:
- Add scheduler to server main loop
- Hook save trigger to block modifications
- Add shutdown flush

**Validation**: Integration test: modify blocks → tick → verify saved

### Phase 5: Server Integration (plix-server)

**Changes**: Modify `lib.rs` and related

1. Add world loading on server start
2. Add world saving on shutdown
3. Hook block edits to mark persistence dirty
4. Add auto-save tick to game loop

**Validation**: Manual test: start server → modify world → restart → verify state

### Phase 6: Solo Mode (plix-client)

**Estimated files**: 2 new files in `crates/plix-client/src/persist/`

1. **local_store.rs** - Wrapper around WorldStore for solo
2. **mod.rs** - Re-export

**Integration**:
- Save on quit/menu
- Load on world selection
- Optional auto-save

**Validation**: Manual test: solo play → quit → reload → verify state

---

## Key Code Snippets

### Atomic Write Utility

```rust
// crates/plix-server/src/persist/atomic.rs

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

pub fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");

    // Write to temp file
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;

    // Atomic rename
    fs::rename(&temp_path, path)?;

    // Sync parent directory (Linux/POSIX durability)
    if let Some(parent) = path.parent() {
        if let Ok(dir) = File::open(parent) {
            let _ = dir.sync_all();
        }
    }

    Ok(())
}
```

### ChunkCodec Encode/Decode

```rust
// crates/plix-common/src/persist/chunk_codec.rs

use bincode::{deserialize, serialize};
use serde::{Deserialize, Serialize};

use crate::chunk::{Chunk, ChunkCoord, CHUNK_BLOCK_COUNT};
use crate::persist::PersistError;
use crate::types::BlockType;

pub const CHUNK_FORMAT_VERSION: u8 = 1;

#[derive(Serialize, Deserialize)]
pub struct ChunkData {
    pub version: u8,
    pub coord: ChunkCoord,
    pub blocks: [BlockType; CHUNK_BLOCK_COUNT],
}

impl ChunkData {
    pub fn from_chunk(chunk: &Chunk) -> Self {
        Self {
            version: CHUNK_FORMAT_VERSION,
            coord: chunk.coord(),
            blocks: chunk.blocks().clone(),
        }
    }

    pub fn into_chunk(self) -> Chunk {
        Chunk::from_blocks(self.coord, self.blocks)
    }
}

pub fn encode(chunk: &Chunk) -> Result<Vec<u8>, PersistError> {
    let data = ChunkData::from_chunk(chunk);
    serialize(&data).map_err(|e| PersistError::Codec(e.to_string()))
}

pub fn decode(bytes: &[u8]) -> Result<Chunk, PersistError> {
    let data: ChunkData = deserialize(bytes)
        .map_err(|e| PersistError::Codec(e.to_string()))?;

    if data.version > CHUNK_FORMAT_VERSION {
        return Err(PersistError::VersionMismatch {
            world_version: data.version as u32,
            reason: VersionCheck::TooNew(data.version as u32),
        });
    }

    Ok(data.into_chunk())
}
```

### Dirty Tracking in ChunkedWorld

```rust
// crates/plix-common/src/world.rs (additions)

use std::collections::HashSet;

pub struct ChunkedWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
    persistence_dirty: HashSet<ChunkCoord>,  // NEW
}

impl ChunkedWorld {
    pub fn set_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<ChunkCoord> {
        let (chunk_coord, local) = world_to_chunk(pos.x, pos.y, pos.z);

        // Mark for persistence BEFORE modifying
        if let Some(chunk) = self.chunks.get_mut(&chunk_coord) {
            let old_block = chunk.get_block(local.0, local.1, local.2);
            if old_block != block {
                self.persistence_dirty.insert(chunk_coord);  // NEW
            }
        }

        // ... existing set_block logic ...
    }

    // NEW methods
    pub fn persistence_dirty_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.persistence_dirty.iter().copied()
    }

    pub fn clear_persistence_dirty(&mut self, coord: ChunkCoord) {
        self.persistence_dirty.remove(&coord);
    }

    pub fn persistence_dirty_count(&self) -> usize {
        self.persistence_dirty.len()
    }
}
```

### SaveScheduler Tick

```rust
// crates/plix-server/src/persist/scheduler.rs

use std::time::{Duration, Instant};

pub struct SaveScheduler {
    config: SaveSchedulerConfig,
    last_save: Instant,
    dirty_chunks: HashSet<ChunkCoord>,
    metrics: SaveMetrics,
}

impl SaveScheduler {
    pub fn tick(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
    ) -> Result<usize, PersistError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let now = Instant::now();
        if now.duration_since(self.last_save) < self.config.interval {
            return Ok(0);
        }

        let start = Instant::now();
        let mut saved = 0;

        // Take up to max_chunks_per_cycle
        let to_save: Vec<_> = world
            .persistence_dirty_chunks()
            .take(self.config.max_chunks_per_cycle)
            .collect();

        for coord in to_save {
            if let Some(chunk) = world.get_chunk(coord) {
                match store.save_chunk(coord, chunk) {
                    Ok(()) => {
                        // Note: caller must clear dirty flag
                        saved += 1;
                        self.metrics.chunks_saved_total += 1;
                    }
                    Err(e) => {
                        tracing::error!("Failed to save chunk {:?}: {}", coord, e);
                        self.metrics.save_failures_total += 1;
                    }
                }
            }
        }

        self.last_save = now;
        self.metrics.save_cycles_total += 1;
        self.metrics.chunks_saved_last_cycle = saved as u32;
        self.metrics.last_cycle_duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            "Auto-save complete: {} chunks in {}ms",
            saved,
            self.metrics.last_cycle_duration_ms
        );

        Ok(saved)
    }
}
```

---

## Testing Checklist

### Unit Tests

- [ ] ChunkCodec encode/decode round-trip
- [ ] ChunkCodec version validation
- [ ] WorldMetadata serialization
- [ ] Version check logic (current, compatible, too old, too new)
- [ ] Chunk filename generation/parsing
- [ ] Atomic write utility (mock filesystem)

### Integration Tests

- [ ] Create new world → verify directory structure
- [ ] Save chunk → load chunk → verify identical
- [ ] Procedural world: modify → save → reload → verify regeneration + modifications
- [ ] Version migration (when migrations exist)
- [ ] List worlds with mixed valid/corrupted entries

### Manual Tests

- [ ] Server: start → modify → restart → verify state preserved
- [ ] Server: modify during play → wait for auto-save → kill process → restart → verify
- [ ] Solo: play → quit → reload → verify state preserved
- [ ] Version mismatch: create world with future version → verify clear error message

---

## Common Pitfalls

### 1. Forgetting to Clear Dirty Flags

After saving a chunk, clear the dirty flag:
```rust
if store.save_chunk(coord, chunk).is_ok() {
    world.clear_persistence_dirty(coord);
}
```

### 2. Blocking Main Thread

Don't call `save_all_chunks()` synchronously. Use:
- Incremental saves with chunk budget
- Async task for large flushes

### 3. Partial Writes on Crash

Always use atomic write pattern:
```rust
// WRONG: Direct write
fs::write(path, data)?;

// RIGHT: Atomic via temp + rename
atomic_write(path, data)?;
```

### 4. Missing Parent Directory Sync

For true durability on Linux, sync parent directory after rename:
```rust
fs::rename(&temp_path, &final_path)?;
File::open(final_path.parent().unwrap())?.sync_all()?;
```

### 5. Chunk Filename Collisions

Ensure signed integers are handled correctly:
```rust
// WRONG: -1 becomes "4294967295"
format!("{}_{}_{}bin", coord.x as u32, ...)

// RIGHT: Keep signed
format!("{}_{}_{}bin", coord.x, coord.y, coord.z)
```

---

## Success Criteria Verification

| Criterion | How to Verify |
|-----------|---------------|
| SC-001: 100% fidelity | Integration test: random blocks → save → load → compare |
| SC-002: <500ms save | Benchmark test with 100 dirty chunks |
| SC-003: <50ms metadata | Benchmark test loading 10 world metadata |
| SC-004: Proportional size | Create 1000-chunk procedural world, modify 10, verify ~40KB saved |
| SC-005: Version handling | Unit tests for all VersionCheck variants |
| SC-006: Crash recovery | Kill during save, verify previous state loadable |
| SC-007: Server persistence | Manual test: server restart preserves state |

---

## Next Steps

After implementing:
1. Run `/speckit.tasks` to generate detailed task breakdown
2. Implement in order: Phase 1 → Phase 6
3. Run tests after each phase
4. Update CLAUDE.md with new technology if any added
