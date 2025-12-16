//! Client-side world state
//! T033-T034 [US1]

use std::collections::HashSet;

use plix_arena::format::LoadedArena;
use plix_common::chunk::ChunkCoord;
use plix_common::protocol::BlockEditApplied;
use plix_common::types::{BlockPos, BlockType};
use plix_common::ChunkedWorld;

/// Client-side world wrapper with chunk-based storage and dirty tracking.
///
/// T033 [US1]: This struct now uses ChunkedWorld for block storage while
/// maintaining the LoadedArena reference for features like raycasting that
/// need arena bounds information.
pub struct ClientWorld {
    /// The original loaded arena (for bounds, spawn info, raycasting)
    arena: LoadedArena,
    /// T033 [US1]: Chunked world storage for per-chunk meshing
    chunks: ChunkedWorld,
    /// Set of chunk coordinates that need mesh rebuild
    dirty_chunks: HashSet<ChunkCoord>,
}

impl ClientWorld {
    /// Create a new client world from a loaded arena.
    ///
    /// T033 [US1]: Converts the arena to ChunkedWorld on creation.
    pub fn new(arena: LoadedArena) -> Self {
        // T034 [US1]: Convert arena to chunked world
        let chunks = arena.to_chunked_world();

        // Mark all initial chunks as dirty for first mesh build
        let dirty_chunks: HashSet<ChunkCoord> = chunks.iter_chunks().map(|(c, _)| *c).collect();

        Self {
            arena,
            chunks,
            dirty_chunks,
        }
    }

    /// Get immutable reference to the arena (for bounds, raycasting, etc.)
    pub fn arena(&self) -> &LoadedArena {
        &self.arena
    }

    /// T033 [US1]: Get immutable reference to the chunked world.
    pub fn chunked_world(&self) -> &ChunkedWorld {
        &self.chunks
    }

    /// Check if any chunks need mesh rebuild.
    pub fn is_dirty(&self) -> bool {
        !self.dirty_chunks.is_empty()
    }

    /// Get the set of dirty chunk coordinates.
    pub fn dirty_chunks(&self) -> &HashSet<ChunkCoord> {
        &self.dirty_chunks
    }

    /// Clear all dirty flags (call after all meshes rebuilt).
    pub fn clear_dirty(&mut self) {
        self.dirty_chunks.clear();
    }

    /// Clear dirty flag for a specific chunk.
    pub fn clear_chunk_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.remove(&coord);
    }

    /// Mark a chunk as dirty (requires mesh rebuild).
    pub fn mark_chunk_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.insert(coord);
    }

    /// Apply a block edit from the server.
    ///
    /// T033 [US1]: Updates both arena (for raycasting) and chunked world,
    /// marking affected chunks as dirty.
    pub fn apply_edit(&mut self, edit: &BlockEditApplied) {
        // Update arena for backwards compatibility (raycasting, etc.)
        self.arena.set_block(edit.pos, edit.new_block);

        // Update chunked world and get affected chunks
        let affected = self.chunks.set_block(edit.pos, edit.new_block);

        // Mark affected chunks as dirty (includes boundary neighbors)
        for coord in affected {
            self.dirty_chunks.insert(coord);
        }
    }

    /// Get block at position.
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        self.chunks.get_block(pos)
    }

    /// Get arena size.
    pub fn size(&self) -> [u32; 3] {
        self.arena.size()
    }

    /// Check if position is in bounds.
    pub fn is_in_bounds(&self, pos: BlockPos) -> bool {
        self.arena.is_in_bounds(pos)
    }

    /// Get total chunk count.
    pub fn chunk_count(&self) -> usize {
        self.chunks.chunk_count()
    }

    /// Iterate over all loaded chunks.
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &plix_common::Chunk)> {
        self.chunks.iter_chunks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_arena::format::{Arena, ArenaMetadata, BlockDefinitions};
    use plix_common::time::Tick;

    fn make_test_arena() -> LoadedArena {
        let definition = Arena {
            metadata: ArenaMetadata {
                name: "test".to_string(),
                version: "1.0".to_string(),
                size: [16, 16, 16],
            },
            spawn_points: vec![],
            blocks: BlockDefinitions {
                floor: None,
                walls: None,
                regions: vec![],
            },
        };

        let blocks = vec![BlockType::STONE; 16 * 16 * 16];
        LoadedArena { definition, blocks }
    }

    #[test]
    fn test_new_world_is_dirty() {
        let world = ClientWorld::new(make_test_arena());
        assert!(world.is_dirty());
    }

    #[test]
    fn test_clear_dirty() {
        let mut world = ClientWorld::new(make_test_arena());
        assert!(world.is_dirty());
        world.clear_dirty();
        assert!(!world.is_dirty());
    }

    #[test]
    fn test_apply_edit_sets_dirty() {
        let mut world = ClientWorld::new(make_test_arena());
        world.clear_dirty();
        assert!(!world.is_dirty());

        let edit = BlockEditApplied {
            pos: BlockPos { x: 5, y: 5, z: 5 },
            new_block: BlockType::AIR,
            tick: Tick(100),
        };
        world.apply_edit(&edit);

        assert!(world.is_dirty());
        assert_eq!(
            world.get_block(BlockPos { x: 5, y: 5, z: 5 }),
            BlockType::AIR
        );
    }

    // ==================== Feature 012 Integration Tests ====================

    #[test]
    fn test_block_edit_marks_dirty_via_chunk_manager() {
        // [F012 T047] Integration test: Block edit via ChunkManager marks correct chunks
        use crate::chunk_manager::{ChunkManager, ChunkManagerConfig};
        use plix_common::math::Vec3;

        // Create world
        let world = ClientWorld::new(make_test_arena());

        // Create chunk manager and initialize from world
        let mut chunk_manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 100,
            max_retries: 3,
        });
        chunk_manager.initialize_from_world(world.chunked_world());

        // Drain initial dirty queue via update()
        let player_pos = Vec3::new(8.0, 8.0, 8.0);
        while chunk_manager.dirty_count() > 0 {
            chunk_manager.update(player_pos, world.chunked_world());
        }

        // Edit a block at interior position (should mark 1 chunk)
        let interior_pos = BlockPos::new(8, 8, 8); // Interior of chunk (0, 0, 0)
        chunk_manager.mark_dirty_for_block(interior_pos);

        assert!(
            chunk_manager.dirty_count() >= 1,
            "Interior edit should mark at least 1 chunk dirty"
        );

        // Drain and test boundary edit
        while chunk_manager.dirty_count() > 0 {
            chunk_manager.update(player_pos, world.chunked_world());
        }

        // Edit a block at boundary (should mark multiple chunks if neighbors loaded)
        let boundary_pos = BlockPos::new(0, 8, 8); // X=0 boundary of chunk (0, 0, 0)
        chunk_manager.mark_dirty_for_block(boundary_pos);

        // Should mark chunk (0,0,0) at minimum, plus neighbor (-1,0,0) if loaded
        assert!(
            chunk_manager.dirty_count() >= 1,
            "Boundary edit should mark at least the containing chunk"
        );
    }
}
