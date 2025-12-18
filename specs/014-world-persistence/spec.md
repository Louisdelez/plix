# Feature Specification: World Persistence

**Feature Branch**: `014-world-persistence`
**Created**: 2025-12-16
**Status**: Draft
**Input**: User description: "Feature 014 – World Persistence: Save and load chunked voxel worlds in solo and server modes with format versioning for future compatibility"

## Clarifications

### Session 2025-12-16

- Q: Should servers auto-save periodically during operation? → A: Yes, periodic auto-save at fixed intervals (e.g., every 5 minutes)

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Solo World Save and Reload (Priority: P1)

As a solo player, I want to save my voxel world before quitting and reload it later in the exact same state, so that I can continue playing where I left off without losing any progress.

**Why this priority**: This is the core value proposition of the feature. Without basic save/load functionality, all player progress is lost on exit, making the game unsuitable for any meaningful play session.

**Independent Test**: Can be fully tested by creating a solo world, placing/removing blocks, saving, restarting the game, and loading the world. Delivers immediate value by preserving player work.

**Acceptance Scenarios**:

1. **Given** a solo world with modified blocks, **When** the player saves and quits, **Then** the world data is written to disk and can be found in the saves directory.
2. **Given** a previously saved world, **When** the player loads it, **Then** the world state (all blocks and chunks) is identical to when it was saved.
3. **Given** an in-progress game, **When** a save operation completes, **Then** gameplay is not noticeably interrupted (save happens without freezing).

---

### User Story 2 - Server World Persistence (Priority: P1)

As a server administrator, I want the server world to persist between server restarts, so that players can continue their progress on a persistent multiplayer world.

**Why this priority**: Equal to solo save/load as it addresses the same core need for multiplayer environments. Server persistence is essential for any meaningful multiplayer experience.

**Independent Test**: Can be tested by running a server, having clients modify blocks, stopping the server, restarting it, and verifying the world state is preserved.

**Acceptance Scenarios**:

1. **Given** a running server with player modifications, **When** the server is stopped gracefully, **Then** the world is automatically saved before shutdown.
2. **Given** a server that was previously stopped, **When** it restarts, **Then** the world loads from the last saved state automatically.
3. **Given** multiple chunks modified by different players, **When** the server saves, **Then** all modified chunks are persisted correctly.
4. **Given** a running server with the default auto-save interval, **When** 5 minutes elapse with modified chunks, **Then** the server automatically saves all modified chunks without admin intervention.

---

### User Story 3 - Procedural World with Modifications (Priority: P2)

As a player in a procedurally generated world, I want my block modifications to be saved without storing the entire world, so that save files remain small while preserving my changes.

**Why this priority**: Builds on P1 functionality to support the existing procedural generation system (Feature 013). Critical for practical use since most worlds will be procedurally generated.

**Independent Test**: Can be tested by generating a world with a seed, modifying some blocks, saving, reloading, and verifying that both the original generated terrain and modifications are correct.

**Acceptance Scenarios**:

1. **Given** a procedurally generated world with local modifications, **When** saving, **Then** only modified chunks are stored (not the entire generated world).
2. **Given** a saved procedural world with modifications, **When** loading, **Then** unmodified areas regenerate from the seed and modified areas load from saved data.
3. **Given** a world with seed 12345 and modifications at chunk (0,0,0), **When** reloading, **Then** the world is identical to before saving (generated + modified parts).

---

### User Story 4 - World Metadata Access (Priority: P2)

As a player, I want to see information about my saved worlds (name, creation date, seed) before loading them, so that I can choose which world to open.

**Why this priority**: Improves usability by providing world selection context. Not blocking for core functionality but essential for multi-world management.

**Independent Test**: Can be tested by creating multiple worlds and verifying their metadata is displayed correctly in a world list without loading the full world.

**Acceptance Scenarios**:

1. **Given** multiple saved worlds, **When** listing available worlds, **Then** each world's name, seed (if applicable), and creation date are shown.
2. **Given** a world save directory, **When** querying metadata, **Then** the metadata loads quickly without loading chunk data.
3. **Given** a corrupted world with valid metadata, **When** listing worlds, **Then** the world appears in the list with an indicator that it may have issues.

---

### User Story 5 - Version Compatibility Handling (Priority: P2)

As a player or server admin, I want clear feedback when a world cannot be loaded due to version incompatibility, so that I understand why and what options I have.

**Why this priority**: Essential for long-term maintainability and user trust. Players need confidence that their worlds won't silently corrupt.

**Independent Test**: Can be tested by creating worlds with different version numbers and attempting to load them with a client that supports/doesn't support those versions.

**Acceptance Scenarios**:

1. **Given** a world saved with the current format version, **When** loading, **Then** the world loads successfully.
2. **Given** a world saved with an older supported version, **When** loading, **Then** the world is migrated automatically and loads correctly.
3. **Given** a world saved with a newer unsupported version, **When** attempting to load, **Then** loading fails with a clear error message explaining the version mismatch.
4. **Given** a world saved with an older unsupported version, **When** attempting to load, **Then** loading fails with a message explaining migration is not available.

---

### User Story 6 - Crash-Safe Saving (Priority: P3)

As a player or server admin, I want my world to remain valid even if the game crashes during a save, so that I don't lose my entire world due to a single crash.

**Why this priority**: Important for data integrity but less critical than basic functionality. Addresses reliability concerns for long-running servers.

**Independent Test**: Can be tested by simulating a crash during save (e.g., killing the process) and verifying the world can still be loaded (from previous valid state).

**Acceptance Scenarios**:

1. **Given** a save in progress, **When** the process is killed, **Then** the previous valid world state remains loadable.
2. **Given** a partial save file, **When** loading, **Then** the system detects corruption and falls back to the last valid state or reports the error clearly.
3. **Given** concurrent chunk saves, **When** one chunk save fails, **Then** other chunks remain valid and the failed chunk is retried or reported.

---

### Edge Cases

- What happens when disk space runs out during save? System must detect this and report an error without corrupting existing data.
- How does the system handle very large worlds (thousands of chunks)? Saves must be incremental and not block gameplay.
- What happens if the saves directory is read-only? System must detect and report permission errors clearly.
- What happens if a chunk file is corrupted? System must isolate the corruption and allow other chunks to load, reporting the specific issue.
- How are simultaneous save requests handled? System must queue or merge requests to prevent conflicts.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST save world data to persistent storage in a structured format per chunk.
- **FR-002**: System MUST load world data from persistent storage and restore exact world state (blocks, chunk presence).
- **FR-003**: System MUST support saving/loading in both solo (local) and server modes.
- **FR-004**: System MUST save only modified chunks for procedurally generated worlds (delta/diff approach).
- **FR-005**: System MUST store and load world metadata (name, seed, creation timestamp, generation parameters) separately from chunk data.
- **FR-006**: System MUST include a format version number in all saved world data.
- **FR-007**: System MUST successfully load worlds with supported format versions.
- **FR-008**: System MUST automatically migrate worlds from older supported versions to current version when possible.
- **FR-009**: System MUST reject loading worlds with unsupported versions (too old or too new) with clear error messages.
- **FR-010**: System MUST use atomic save operations to prevent corruption from crashes or interruptions.
- **FR-011**: System MUST validate world data integrity on load and reject or report corrupted data.
- **FR-012**: System MUST perform save operations without blocking gameplay (async or background saving).
- **FR-013**: System MUST track which chunks have been modified since generation or last save.
- **FR-014**: System MUST provide a mechanism to list available worlds with their metadata without loading full world data.
- **FR-015**: Server MUST perform periodic auto-saves at configurable intervals (default: every 5 minutes) to minimize data loss from unexpected shutdowns.

### Key Entities

- **World**: The top-level container representing a complete game world. Has a unique identifier/name, optional seed for procedural generation, creation timestamp, generation parameters, and format version.
- **WorldMetadata**: Lightweight information about a world (name, seed, creation date, version) that can be loaded quickly without loading chunk data. Stored separately from chunk data.
- **Chunk**: A fixed-size 3D section of the world containing block data. Identified by its coordinates. Can be in states: not-loaded, generated (unmodified), modified, saved.
- **ChunkDiff**: Represents modifications to a chunk relative to its procedurally generated state. Contains only the blocks that differ from generation.
- **SaveVersion**: Format version identifier included in all persistent data. Used for compatibility checking and migration decisions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Players can save and reload a world with 100% fidelity (no block data loss or corruption).
- **SC-002**: Save operations for typical gameplay (dozens of modified chunks) complete in under 500ms without blocking the main game loop.
- **SC-003**: World metadata loads in under 50ms, enabling quick world selection screens.
- **SC-004**: Save file size for a procedural world with local modifications is proportional to modified chunks only (not total world size).
- **SC-005**: System correctly handles version mismatches 100% of the time (never silently corrupts or loses data).
- **SC-006**: System recovers gracefully from interrupted saves 100% of the time (no permanent data loss from crashes).
- **SC-007**: Server can persist and restore worlds across restarts with zero data loss in normal operation.

## Assumptions

- Worlds are chunked (as established in Feature 011 - Chunked World).
- Procedural generation is deterministic from a seed (as established in Feature 013 - Procedural Generation).
- The file system is the persistence mechanism (no database required for this feature).
- Chunk coordinates and block types are stable across versions (structural changes to these would require explicit migration).
- Default save location: `~/.local/share/plix/worlds/` on Linux, platform-appropriate locations on other systems.
- Maximum of 3 [NEEDS CLARIFICATION] items have been avoided by using reasonable defaults based on industry standards (Minecraft-style chunk saving, write-ahead patterns for crash safety).

## Out of Scope

- Entity persistence (players, mobs, items) - future feature.
- Database or distributed storage backends.
- Client-side session/network state persistence.
- Advanced compression optimization beyond basic binary serialization.
- World backup/restore UI or automatic backup rotation.
- Cross-platform world transfer tools.
