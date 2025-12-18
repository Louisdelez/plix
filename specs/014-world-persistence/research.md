# Research: World Persistence

**Feature**: 014-world-persistence
**Date**: 2025-12-16

## Research Questions Addressed

This document consolidates research findings for technical decisions in the World Persistence feature.

---

## 1. Atomic File Write Strategy

**Decision**: Write-to-temp + rename pattern

**Rationale**:
- `rename()` is atomic on POSIX systems (Linux, macOS)
- Guarantees file is either fully old or fully new after crash
- No partial writes visible to readers
- Standard pattern used by SQLite, Minecraft, and most game save systems

**Alternatives Considered**:

| Alternative | Rejected Because |
|-------------|------------------|
| Write-in-place | Crash leaves corrupted file |
| Copy-on-write filesystem features | Not portable, requires specific FS |
| WAL (Write-Ahead Log) | Over-engineering for per-chunk files |
| Database (SQLite) | Added dependency, slower for streaming access |

**Implementation**:
```rust
fn atomic_write(path: &Path, data: &[u8]) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    file.write_all(data)?;
    file.sync_all()?;  // fsync the file
    std::fs::rename(&temp_path, path)?;
    // fsync parent directory for durability
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
```

---

## 2. Chunk File Format

**Decision**: bincode serialization of `ChunkData` struct

**Rationale**:
- bincode already used throughout codebase for network protocol
- Compact binary format (~4KB per chunk)
- Deterministic encoding (same data = same bytes)
- Fast encode/decode (no parsing overhead)
- Serde integration already in place for all types

**Alternatives Considered**:

| Alternative | Rejected Because |
|-------------|------------------|
| JSON/TOML | Text formats too large (~50KB+ per chunk) |
| MessagePack | Additional dependency, marginal benefit |
| Cap'n Proto | Additional dependency, complexity |
| Custom binary | Reinventing bincode, maintenance burden |
| Compressed (zstd) | Out of scope per spec; can add later |

**Format Structure (v1)**:
```rust
#[derive(Serialize, Deserialize)]
struct ChunkData {
    version: u8,                    // Always 1 for v1
    coord: ChunkCoord,              // (i32, i32, i32)
    blocks: [BlockType; 4096],      // 4096 × u8
}
// Approximate size: 4109 bytes
```

---

## 3. World Directory Structure

**Decision**: One directory per world with metadata + chunks subdirectory

**Rationale**:
- Simple filesystem navigation
- Easy backup (copy directory)
- Chunk files named by coordinate (fast lookup)
- Metadata separate for quick world listing
- Temp directory for atomic writes

**Structure**:
```
~/.local/share/plix/worlds/
└── my_world/
    ├── meta.bin           # WorldMetadata (small, loaded first)
    ├── chunks/
    │   ├── 0_0_0.bin      # Chunk at (0, 0, 0)
    │   ├── -1_0_0.bin     # Chunk at (-1, 0, 0)
    │   └── ...
    └── temp/              # Staging for atomic writes
```

**Alternatives Considered**:

| Alternative | Rejected Because |
|-------------|------------------|
| Single file per world | Can't do incremental saves |
| SQLite database | Added dependency, overkill |
| Region files (Minecraft Anvil) | Complexity not justified for MVP |
| Nested directories by coordinate | Unnecessary for expected scale |

---

## 4. Version Migration Strategy

**Decision**: Version number in metadata, explicit migration functions

**Rationale**:
- Simple integer version (u32) easy to compare
- Migration functions registered per version pair
- Fail-fast for unsupported versions
- Clear error messages for users

**Version Handling**:
```rust
pub enum VersionCheck {
    Current,                    // version == CURRENT
    Compatible(u32),            // MIN_SUPPORTED <= version < CURRENT
    TooOld(u32),               // version < MIN_SUPPORTED
    TooNew(u32),               // version > CURRENT
}

impl SaveVersion {
    pub fn check(version: u32) -> VersionCheck {
        match version {
            v if v == CURRENT_VERSION => VersionCheck::Current,
            v if v >= MIN_SUPPORTED_VERSION => VersionCheck::Compatible(v),
            v if v > CURRENT_VERSION => VersionCheck::TooNew(v),
            v => VersionCheck::TooOld(v),
        }
    }
}
```

**Migration Registry** (for future use):
```rust
type MigrationFn = fn(&[u8]) -> Result<Vec<u8>, MigrationError>;

// Example: v1 -> v2 migration (when needed)
fn migrate_v1_to_v2(data: &[u8]) -> Result<Vec<u8>, MigrationError> {
    // Transform old format to new format
}
```

---

## 5. Dirty Chunk Tracking

**Decision**: Separate `persistence_dirty` HashSet in ChunkedWorld

**Rationale**:
- Existing `dirty` flag on Chunk is for mesh rebuilding
- Persistence needs separate tracking (different lifecycle)
- HashSet allows O(1) add/remove/contains
- Iterator for save operations

**Implementation Approach**:
```rust
pub struct ChunkedWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
    persistence_dirty: HashSet<ChunkCoord>,  // NEW
}

impl ChunkedWorld {
    pub fn set_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<ChunkCoord> {
        // ... existing logic ...
        self.persistence_dirty.insert(chunk_coord);  // NEW
        // ... return affected chunks for mesh rebuild ...
    }

    pub fn persistence_dirty_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.persistence_dirty.iter().copied()
    }

    pub fn clear_persistence_dirty(&mut self, coord: ChunkCoord) {
        self.persistence_dirty.remove(&coord);
    }
}
```

---

## 6. Auto-Save Scheduling

**Decision**: Timer-based with chunk budget per cycle

**Rationale**:
- Fixed interval (default 5 min) predictable for users
- Chunk budget prevents save from blocking game loop
- Leftover chunks saved in next cycle
- Flush on shutdown saves everything

**Configuration**:
```rust
pub struct SaveSchedulerConfig {
    pub auto_save_interval: Duration,    // Default: 5 minutes
    pub max_chunks_per_cycle: usize,     // Default: 100
    pub enabled: bool,                   // Default: true for server
}
```

**Tick Logic**:
```rust
impl SaveScheduler {
    pub fn tick(&mut self, world: &ChunkedWorld, store: &WorldStore) {
        if !self.config.enabled { return; }

        let now = Instant::now();
        if now.duration_since(self.last_save) < self.config.auto_save_interval {
            return;
        }

        let dirty: Vec<_> = world.persistence_dirty_chunks()
            .take(self.config.max_chunks_per_cycle)
            .collect();

        for coord in dirty {
            if let Some(chunk) = world.get_chunk(coord) {
                if let Err(e) = store.save_chunk(coord, chunk) {
                    tracing::error!("Failed to save chunk {:?}: {}", coord, e);
                    self.metrics.save_failures += 1;
                } else {
                    world.clear_persistence_dirty(coord);
                    self.metrics.chunks_saved += 1;
                }
            }
        }

        self.last_save = now;
        self.metrics.save_cycles += 1;
    }
}
```

---

## 7. Solo vs Server Mode

**Decision**: Unified WorldStore, different triggers

**Rationale**:
- Same persistence code for both modes
- Server: auto-save timer + shutdown flush
- Solo: save on quit/menu, optional auto-save
- Reduces code duplication and testing surface

**Mode Differences**:

| Aspect | Server | Solo |
|--------|--------|------|
| Auto-save | Enabled by default | Optional/configurable |
| Save trigger | Timer + shutdown | Quit/menu + optional timer |
| WorldStore | Shared via Arc | Owned by client |
| Path | Server data directory | User data directory |

---

## 8. Error Handling Strategy

**Decision**: Explicit Result types, no panics, graceful degradation

**Rationale**:
- Constitution requires no panics in production
- I/O errors are expected (disk full, permissions)
- Corruption should be detected and reported, not ignored
- Partial success acceptable (some chunks saved)

**Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum PersistError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Encode(#[from] bincode::Error),

    #[error("World not found: {0}")]
    WorldNotFound(String),

    #[error("Version too new: world v{world}, max supported v{supported}")]
    VersionTooNew { world: u32, supported: u32 },

    #[error("Version too old: world v{world}, min supported v{supported}")]
    VersionTooOld { world: u32, supported: u32 },

    #[error("Chunk corrupted at {coord:?}: {reason}")]
    ChunkCorrupted { coord: ChunkCoord, reason: String },

    #[error("Metadata corrupted: {0}")]
    MetadataCorrupted(String),
}
```

---

## 9. Platform-Specific Paths

**Decision**: Use `dirs` crate for standard data directories

**Rationale**:
- Cross-platform standard (XDG on Linux, AppData on Windows)
- No hardcoded paths in code
- User-overridable via config

**Default Paths**:
- Linux: `~/.local/share/plix/worlds/`
- macOS: `~/Library/Application Support/plix/worlds/`
- Windows: `%APPDATA%\plix\worlds\`

**Implementation**:
```rust
pub fn default_worlds_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("worlds")
}
```

**Note**: `dirs` crate is lightweight and commonly used. If not already in deps, add as `dirs = "5"`.

---

## 10. Testing Strategy

**Decision**: Multi-layer testing approach

**Unit Tests** (persist module):
- ChunkCodec encode/decode round-trip
- Version check logic
- Atomic write utility
- Path generation

**Integration Tests**:
- Create world → save chunks → reload → verify
- Procedural world with modifications (delta save)
- Version migration (when migrations exist)

**Crash Simulation**:
- Write interrupted mid-file (temp file remains)
- Recovery loads previous valid state

**Example Test**:
```rust
#[test]
fn test_chunk_roundtrip() {
    let chunk = Chunk::new(ChunkCoord::new(1, 2, 3));
    chunk.set_block(0, 0, 0, BlockType::STONE);

    let encoded = ChunkCodec::encode(&chunk).unwrap();
    let decoded = ChunkCodec::decode(&encoded).unwrap();

    assert_eq!(chunk.coord(), decoded.coord());
    assert_eq!(chunk.get_block(0, 0, 0), decoded.get_block(0, 0, 0));
}

#[test]
fn test_version_too_new() {
    let check = SaveVersion::check(CURRENT_VERSION + 1);
    assert!(matches!(check, VersionCheck::TooNew(_)));
}
```

---

## Summary of Decisions

| Topic | Decision |
|-------|----------|
| Atomic writes | Write-to-temp + rename |
| Chunk format | bincode serialization |
| Directory structure | Per-world dir with chunks/ subdir |
| Versioning | Integer version + migration registry |
| Dirty tracking | Separate HashSet in ChunkedWorld |
| Auto-save | Timer-based with chunk budget |
| Solo/Server | Unified store, different triggers |
| Errors | Explicit Result types, no panics |
| Paths | `dirs` crate for platform defaults |
| Testing | Unit + integration + crash simulation |

All research questions resolved. Proceed to Phase 1 (data model and contracts).
