//! ChunkedWorld container for managing chunks in the world.
//!
//! This module provides the main container for storing and accessing
//! blocks across chunk boundaries.

use std::collections::{HashMap, HashSet};

use crate::chunk::{boundary_neighbors, Chunk, ChunkCoord};
use crate::types::{BlockPos, BlockType};

/// Container for all chunks in the world.
///
/// Provides block access across chunk boundaries with automatic
/// chunk creation when setting blocks.
///
/// ## Dirty Tracking
///
/// Two types of dirty tracking are maintained:
/// - **Mesh dirty**: Chunks needing visual mesh rebuild (cleared after render)
/// - **Persistence dirty**: Chunks with block changes needing save to disk
#[derive(Clone)]
pub struct ChunkedWorld {
    chunks: HashMap<ChunkCoord, Chunk>,
    /// Chunks that have been modified and need to be saved to disk.
    persistence_dirty: HashSet<ChunkCoord>,
}

impl ChunkedWorld {
    /// Create a new empty world.
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            persistence_dirty: HashSet::new(),
        }
    }

    /// Get a block at the given world position.
    ///
    /// Returns AIR if the chunk is not loaded.
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        let chunk_coord = pos.chunk_pos();
        match self.chunks.get(&chunk_coord) {
            Some(chunk) => {
                let (lx, ly, lz) = pos.local_pos();
                chunk.get_block(lx, ly, lz)
            }
            None => BlockType::AIR,
        }
    }

    /// Set a block at the given world position.
    ///
    /// Creates the chunk if it doesn't exist.
    /// Returns the list of chunk coordinates that were affected (for mesh dirty marking).
    ///
    /// Also marks the chunk as persistence dirty if the block actually changed.
    pub fn set_block(&mut self, pos: BlockPos, block: BlockType) -> Vec<ChunkCoord> {
        let chunk_coord = pos.chunk_pos();
        let (lx, ly, lz) = pos.local_pos();

        // Check if block is actually changing (before setting)
        let old_block = self
            .chunks
            .get(&chunk_coord)
            .map(|c| c.get_block(lx, ly, lz))
            .unwrap_or(BlockType::AIR);

        // Ensure chunk exists and set the block
        let chunk = self.ensure_chunk(chunk_coord);
        chunk.set_block(lx, ly, lz, block);
        chunk.mark_dirty();

        // Mark for persistence if block actually changed
        if old_block != block {
            self.persistence_dirty.insert(chunk_coord);
        }

        // Collect affected chunks (self + boundary neighbors)
        let mut affected = vec![chunk_coord];
        affected.extend(boundary_neighbors(chunk_coord, (lx, ly, lz)));
        affected
    }

    /// Get an immutable reference to a chunk.
    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    /// Get a mutable reference to a chunk.
    pub fn get_chunk_mut(&mut self, coord: ChunkCoord) -> Option<&mut Chunk> {
        self.chunks.get_mut(&coord)
    }

    /// Ensure a chunk exists at the given coordinate, creating it if needed.
    pub fn ensure_chunk(&mut self, coord: ChunkCoord) -> &mut Chunk {
        self.chunks
            .entry(coord)
            .or_insert_with(|| Chunk::new(coord))
    }

    /// Remove a chunk from the world.
    pub fn remove_chunk(&mut self, coord: ChunkCoord) -> Option<Chunk> {
        self.chunks.remove(&coord)
    }

    /// Check if a chunk is loaded.
    pub fn has_chunk(&self, coord: ChunkCoord) -> bool {
        self.chunks.contains_key(&coord)
    }

    /// Insert a chunk into the world.
    ///
    /// Used by procedural generation to add newly generated chunks.
    /// The chunk is marked dirty for mesh generation.
    pub fn insert_chunk(&mut self, coord: ChunkCoord, mut chunk: Chunk) {
        chunk.mark_dirty();
        self.chunks.insert(coord, chunk);
    }

    /// Get or generate a chunk using the provided generator.
    ///
    /// If the chunk already exists, returns a reference to it.
    /// Otherwise, generates it using the provided generator function.
    pub fn get_or_generate_chunk<F>(&mut self, coord: ChunkCoord, generator: F) -> &mut Chunk
    where
        F: FnOnce(ChunkCoord) -> Chunk,
    {
        self.chunks.entry(coord).or_insert_with(|| {
            let mut chunk = generator(coord);
            chunk.mark_dirty();
            chunk
        })
    }

    /// Iterate over all loaded chunks.
    pub fn iter_chunks(&self) -> impl Iterator<Item = (&ChunkCoord, &Chunk)> {
        self.chunks.iter()
    }

    /// Iterate over all loaded chunks mutably.
    pub fn iter_chunks_mut(&mut self) -> impl Iterator<Item = (&ChunkCoord, &mut Chunk)> {
        self.chunks.iter_mut()
    }

    /// Get the number of loaded chunks.
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Iterate over dirty chunks that need mesh rebuild.
    pub fn dirty_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.chunks
            .iter()
            .filter(|(_, chunk)| chunk.is_dirty())
            .map(|(coord, _)| *coord)
    }

    /// Clear dirty flags on all chunks.
    pub fn clear_all_dirty(&mut self) {
        for chunk in self.chunks.values_mut() {
            chunk.clear_dirty();
        }
    }

    // =========================================================================
    // PERSISTENCE DIRTY TRACKING
    // =========================================================================

    /// Mark a chunk as needing to be saved to disk.
    ///
    /// Call this when a chunk has been modified and needs persistence.
    pub fn mark_persistence_dirty(&mut self, coord: ChunkCoord) {
        self.persistence_dirty.insert(coord);
    }

    /// Clear the persistence dirty flag for a chunk (after saving).
    pub fn clear_persistence_dirty(&mut self, coord: ChunkCoord) {
        self.persistence_dirty.remove(&coord);
    }

    /// Check if a chunk is marked for persistence.
    pub fn is_persistence_dirty(&self, coord: ChunkCoord) -> bool {
        self.persistence_dirty.contains(&coord)
    }

    /// Get the number of chunks needing persistence.
    pub fn persistence_dirty_count(&self) -> usize {
        self.persistence_dirty.len()
    }

    /// Iterate over chunk coordinates that need to be saved.
    pub fn persistence_dirty_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.persistence_dirty.iter().copied()
    }

    /// Clear all persistence dirty flags (after full save).
    pub fn clear_all_persistence_dirty(&mut self) {
        self.persistence_dirty.clear();
    }

    /// Get block at world coordinates (convenience method).
    pub fn get_block_xyz(&self, x: i32, y: i32, z: i32) -> BlockType {
        self.get_block(BlockPos::new(x, y, z))
    }

    /// Set block at world coordinates (convenience method).
    pub fn set_block_xyz(&mut self, x: i32, y: i32, z: i32, block: BlockType) -> Vec<ChunkCoord> {
        self.set_block(BlockPos::new(x, y, z), block)
    }

    /// Get the bounds of loaded chunks (min and max chunk coordinates).
    ///
    /// Returns None if no chunks are loaded.
    pub fn bounds(&self) -> Option<(ChunkCoord, ChunkCoord)> {
        if self.chunks.is_empty() {
            return None;
        }

        let mut min = ChunkCoord::new(i32::MAX, i32::MAX, i32::MAX);
        let mut max = ChunkCoord::new(i32::MIN, i32::MIN, i32::MIN);

        for coord in self.chunks.keys() {
            min = ChunkCoord::new(min.x.min(coord.x), min.y.min(coord.y), min.z.min(coord.z));
            max = ChunkCoord::new(max.x.max(coord.x), max.y.max(coord.y), max.z.max(coord.z));
        }

        Some((min, max))
    }

    /// Populate this world from a flat block array (used for arena loading).
    ///
    /// The array is indexed as: index = z * size_y * size_x + y * size_x + x
    pub fn from_flat_blocks(blocks: &[BlockType], size_x: u32, size_y: u32, size_z: u32) -> Self {
        let mut world = Self::new();

        for z in 0..size_z {
            for y in 0..size_y {
                for x in 0..size_x {
                    let index = (z * size_y * size_x + y * size_x + x) as usize;
                    if index < blocks.len() {
                        let block = blocks[index];
                        if block.is_solid() {
                            let pos = BlockPos::new(x as i32, y as i32, z as i32);
                            let chunk_coord = pos.chunk_pos();
                            let (lx, ly, lz) = pos.local_pos();
                            let chunk = world.ensure_chunk(chunk_coord);
                            chunk.set_block(lx, ly, lz, block);
                        }
                    }
                }
            }
        }

        // Mark all chunks dirty for initial mesh build
        for chunk in world.chunks.values_mut() {
            chunk.mark_dirty();
        }

        world
    }
}

impl Default for ChunkedWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ChunkedWorld {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkedWorld")
            .field("chunk_count", &self.chunks.len())
            .field("bounds", &self.bounds())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_world_is_empty() {
        let world = ChunkedWorld::new();
        assert_eq!(world.chunk_count(), 0);
        assert!(world.bounds().is_none());
    }

    #[test]
    fn test_get_block_unloaded_returns_air() {
        let world = ChunkedWorld::new();
        assert_eq!(
            world.get_block(BlockPos::new(100, 100, 100)),
            BlockType::AIR
        );
    }

    #[test]
    fn test_set_and_get_block() {
        let mut world = ChunkedWorld::new();

        let pos = BlockPos::new(17, 5, 32);
        world.set_block(pos, BlockType::STONE);

        assert_eq!(world.get_block(pos), BlockType::STONE);
        assert_eq!(world.chunk_count(), 1);

        // Other blocks are still AIR
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::AIR);
    }

    #[test]
    fn test_set_block_creates_chunk() {
        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(5, 5, 5);

        assert!(!world.has_chunk(coord));

        let pos = BlockPos::new(80, 80, 80); // chunk (5, 5, 5)
        world.set_block(pos, BlockType::BRICK);

        assert!(world.has_chunk(coord));
    }

    #[test]
    fn test_set_block_returns_affected_chunks() {
        let mut world = ChunkedWorld::new();

        // Interior block - only affects own chunk
        let affected = world.set_block(BlockPos::new(24, 24, 24), BlockType::STONE);
        assert_eq!(affected.len(), 1);
        assert!(affected.contains(&ChunkCoord::new(1, 1, 1)));

        // Boundary block at x=0 of chunk (1,1,1) -> also affects chunk (0,1,1)
        let affected = world.set_block(BlockPos::new(16, 24, 24), BlockType::STONE);
        assert_eq!(affected.len(), 2);
        assert!(affected.contains(&ChunkCoord::new(1, 1, 1)));
        assert!(affected.contains(&ChunkCoord::new(0, 1, 1)));
    }

    #[test]
    fn test_cross_chunk_blocks() {
        let mut world = ChunkedWorld::new();

        // Set blocks in different chunks
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(16, 0, 0), BlockType::BRICK);
        world.set_block(BlockPos::new(-1, 0, 0), BlockType::METAL);

        assert_eq!(world.chunk_count(), 3);
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::STONE);
        assert_eq!(world.get_block(BlockPos::new(16, 0, 0)), BlockType::BRICK);
        assert_eq!(world.get_block(BlockPos::new(-1, 0, 0)), BlockType::METAL);
    }

    #[test]
    fn test_negative_coordinates() {
        let mut world = ChunkedWorld::new();

        let pos = BlockPos::new(-50, -100, -200);
        world.set_block(pos, BlockType::STONE);

        assert_eq!(world.get_block(pos), BlockType::STONE);
        assert!(world.has_chunk(pos.chunk_pos()));
    }

    #[test]
    fn test_remove_chunk() {
        let mut world = ChunkedWorld::new();

        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        assert!(world.has_chunk(ChunkCoord::new(0, 0, 0)));

        let removed = world.remove_chunk(ChunkCoord::new(0, 0, 0));
        assert!(removed.is_some());
        assert!(!world.has_chunk(ChunkCoord::new(0, 0, 0)));
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::AIR);
    }

    #[test]
    fn test_dirty_chunks() {
        let mut world = ChunkedWorld::new();

        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(16, 0, 0), BlockType::STONE);

        let dirty: Vec<_> = world.dirty_chunks().collect();
        assert_eq!(dirty.len(), 2);

        // Clear dirty on one chunk
        if let Some(chunk) = world.get_chunk_mut(ChunkCoord::new(0, 0, 0)) {
            chunk.clear_dirty();
        }

        let dirty: Vec<_> = world.dirty_chunks().collect();
        assert_eq!(dirty.len(), 1);
    }

    #[test]
    fn test_from_flat_blocks() {
        // Create a simple 32x16x32 block world with stone floor at y=0
        let size_x = 32u32;
        let size_y = 16u32;
        let size_z = 32u32;
        let mut blocks = vec![BlockType::AIR; (size_x * size_y * size_z) as usize];

        // Fill floor with stone at y=0
        let y = 0u32;
        for z in 0..size_z {
            for x in 0..size_x {
                let index = (z * size_y * size_x + y * size_x + x) as usize;
                blocks[index] = BlockType::STONE;
            }
        }

        let world = ChunkedWorld::from_flat_blocks(&blocks, size_x, size_y, size_z);

        // Should have created chunks for the floor
        assert!(world.chunk_count() > 0);

        // Check floor blocks
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::STONE);
        assert_eq!(world.get_block(BlockPos::new(15, 0, 15)), BlockType::STONE);
        assert_eq!(world.get_block(BlockPos::new(31, 0, 31)), BlockType::STONE);

        // Check air above
        assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::AIR);
    }

    #[test]
    fn test_bounds() {
        let mut world = ChunkedWorld::new();

        world.set_block(BlockPos::new(-50, 0, 100), BlockType::STONE);
        world.set_block(BlockPos::new(100, 50, -50), BlockType::STONE);

        let bounds = world.bounds().unwrap();
        assert_eq!(bounds.0, ChunkCoord::new(-4, 0, -4)); // min
        assert_eq!(bounds.1, ChunkCoord::new(6, 3, 6)); // max
    }

    #[test]
    fn test_iter_chunks() {
        let mut world = ChunkedWorld::new();

        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(16, 0, 0), BlockType::STONE);

        let coords: Vec<_> = world.iter_chunks().map(|(c, _)| *c).collect();
        assert_eq!(coords.len(), 2);
        assert!(coords.contains(&ChunkCoord::new(0, 0, 0)));
        assert!(coords.contains(&ChunkCoord::new(1, 0, 0)));
    }

    #[test]
    fn test_insert_chunk() {
        use crate::chunk::Chunk;

        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(5, 0, 5);

        assert!(!world.has_chunk(coord));

        let mut chunk = Chunk::new(coord);
        chunk.set_block(4, 4, 4, BlockType::STONE);
        world.insert_chunk(coord, chunk);

        assert!(world.has_chunk(coord));
        assert_eq!(
            world.get_block(BlockPos::new(5 * 16 + 4, 4, 5 * 16 + 4)),
            BlockType::STONE
        );

        // Verify chunk is marked dirty
        let chunk_ref = world.get_chunk(coord).unwrap();
        assert!(chunk_ref.is_dirty());
    }

    #[test]
    fn test_get_or_generate_chunk() {
        use crate::chunk::Chunk;

        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(10, 0, 10);

        // Generator called for missing chunk
        let mut generator_called = false;
        world.get_or_generate_chunk(coord, |c| {
            generator_called = true;
            let mut chunk = Chunk::new(c);
            chunk.set_block(0, 0, 0, BlockType::BEDROCK);
            chunk
        });

        assert!(
            generator_called,
            "Generator should be called for missing chunk"
        );
        assert!(world.has_chunk(coord));
        assert_eq!(
            world.get_block(BlockPos::new(10 * 16, 0, 10 * 16)),
            BlockType::BEDROCK
        );

        // Generator NOT called for existing chunk
        let mut generator_called_again = false;
        world.get_or_generate_chunk(coord, |_| {
            generator_called_again = true;
            panic!("Generator should not be called for existing chunk");
        });

        assert!(
            !generator_called_again,
            "Generator should not be called for existing chunk"
        );
    }

    // ==================== Persistence Dirty Tracking Tests ====================

    #[test]
    fn test_persistence_dirty_on_block_change() {
        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(0, 0, 0);

        // Initially no dirty chunks
        assert_eq!(world.persistence_dirty_count(), 0);

        // Setting a block marks persistence dirty
        world.set_block(BlockPos::new(5, 5, 5), BlockType::STONE);
        assert!(world.is_persistence_dirty(coord));
        assert_eq!(world.persistence_dirty_count(), 1);
    }

    #[test]
    fn test_persistence_dirty_no_change() {
        let mut world = ChunkedWorld::new();

        // Set a block
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.clear_all_persistence_dirty();

        // Setting same block again should NOT mark dirty
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        assert_eq!(world.persistence_dirty_count(), 0);
    }

    #[test]
    fn test_persistence_dirty_clear_single() {
        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(0, 0, 0);

        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        assert!(world.is_persistence_dirty(coord));

        world.clear_persistence_dirty(coord);
        assert!(!world.is_persistence_dirty(coord));
        assert_eq!(world.persistence_dirty_count(), 0);
    }

    #[test]
    fn test_persistence_dirty_clear_all() {
        let mut world = ChunkedWorld::new();

        // Modify multiple chunks
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(16, 0, 0), BlockType::BRICK);
        world.set_block(BlockPos::new(32, 0, 0), BlockType::METAL);

        assert_eq!(world.persistence_dirty_count(), 3);

        world.clear_all_persistence_dirty();
        assert_eq!(world.persistence_dirty_count(), 0);
    }

    #[test]
    fn test_persistence_dirty_chunks_iterator() {
        let mut world = ChunkedWorld::new();

        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(16, 0, 0), BlockType::BRICK);

        let dirty: Vec<_> = world.persistence_dirty_chunks().collect();
        assert_eq!(dirty.len(), 2);
        assert!(dirty.contains(&ChunkCoord::new(0, 0, 0)));
        assert!(dirty.contains(&ChunkCoord::new(1, 0, 0)));
    }

    #[test]
    fn test_persistence_dirty_manual_mark() {
        let mut world = ChunkedWorld::new();
        let coord = ChunkCoord::new(5, 5, 5);

        // Manually mark a chunk dirty (even if not loaded)
        world.mark_persistence_dirty(coord);
        assert!(world.is_persistence_dirty(coord));

        world.clear_persistence_dirty(coord);
        assert!(!world.is_persistence_dirty(coord));
    }
}
