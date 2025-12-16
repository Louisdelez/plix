//! Chunk system types and utilities for the chunked world pipeline.
//!
//! This module provides:
//! - Chunk coordinate math (world <-> chunk <-> local conversions)
//! - Chunk data structure for 16x16x16 block regions
//! - Boundary detection for neighbor invalidation

use crate::math::{Vec3, AABB};
use crate::types::{BlockPos, BlockType, ChunkPos};

/// Chunk size in blocks per dimension (16x16x16)
pub const CHUNK_SIZE: usize = 16;

/// Total blocks per chunk (16^3 = 4096)
pub const CHUNK_BLOCK_COUNT: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;

/// Type alias for chunk coordinates (uses existing ChunkPos)
pub type ChunkCoord = ChunkPos;

/// Convert local coordinates (0-15, 0-15, 0-15) to array index.
///
/// Index formula: z * 256 + y * 16 + x
#[inline]
pub fn local_to_index(lx: usize, ly: usize, lz: usize) -> usize {
    debug_assert!(lx < CHUNK_SIZE && ly < CHUNK_SIZE && lz < CHUNK_SIZE);
    lz * CHUNK_SIZE * CHUNK_SIZE + ly * CHUNK_SIZE + lx
}

/// Convert array index to local coordinates.
///
/// Returns (lx, ly, lz) where each is in range 0-15.
#[inline]
pub fn index_to_local(index: usize) -> (usize, usize, usize) {
    debug_assert!(index < CHUNK_BLOCK_COUNT);
    let lz = index / (CHUNK_SIZE * CHUNK_SIZE);
    let remainder = index % (CHUNK_SIZE * CHUNK_SIZE);
    let ly = remainder / CHUNK_SIZE;
    let lx = remainder % CHUNK_SIZE;
    (lx, ly, lz)
}

/// Convert world coordinates to chunk coordinate and local position.
///
/// Uses floor division to handle negative coordinates correctly.
#[inline]
pub fn world_to_chunk(x: i32, y: i32, z: i32) -> (ChunkCoord, (usize, usize, usize)) {
    let pos = BlockPos::new(x, y, z);
    (pos.chunk_pos(), pos.local_pos())
}

/// Convert chunk coordinate and local position to world coordinates.
#[inline]
pub fn chunk_to_world(coord: ChunkCoord, local: (usize, usize, usize)) -> (i32, i32, i32) {
    let (lx, ly, lz) = local;
    (
        coord.x * CHUNK_SIZE as i32 + lx as i32,
        coord.y * CHUNK_SIZE as i32 + ly as i32,
        coord.z * CHUNK_SIZE as i32 + lz as i32,
    )
}

/// Check if local coordinates are on a chunk boundary.
///
/// Returns true if any axis is at 0 or CHUNK_SIZE-1.
#[inline]
pub fn is_boundary_local(local: (usize, usize, usize)) -> bool {
    let (lx, ly, lz) = local;
    lx == 0
        || lx == CHUNK_SIZE - 1
        || ly == 0
        || ly == CHUNK_SIZE - 1
        || lz == 0
        || lz == CHUNK_SIZE - 1
}

/// Get neighbor chunk coordinates affected by a boundary block edit.
///
/// Returns a list of neighbor chunks that should be marked dirty when
/// a block at the given local position is modified.
pub fn boundary_neighbors(coord: ChunkCoord, local: (usize, usize, usize)) -> Vec<ChunkCoord> {
    let (lx, ly, lz) = local;
    let mut neighbors = Vec::new();

    if lx == 0 {
        neighbors.push(ChunkCoord::new(coord.x - 1, coord.y, coord.z));
    }
    if lx == CHUNK_SIZE - 1 {
        neighbors.push(ChunkCoord::new(coord.x + 1, coord.y, coord.z));
    }
    if ly == 0 {
        neighbors.push(ChunkCoord::new(coord.x, coord.y - 1, coord.z));
    }
    if ly == CHUNK_SIZE - 1 {
        neighbors.push(ChunkCoord::new(coord.x, coord.y + 1, coord.z));
    }
    if lz == 0 {
        neighbors.push(ChunkCoord::new(coord.x, coord.y, coord.z - 1));
    }
    if lz == CHUNK_SIZE - 1 {
        neighbors.push(ChunkCoord::new(coord.x, coord.y, coord.z + 1));
    }

    neighbors
}

/// A 16x16x16 chunk of blocks.
#[derive(Clone)]
pub struct Chunk {
    /// Dense block storage (4096 blocks)
    blocks: [BlockType; CHUNK_BLOCK_COUNT],
    /// Whether this chunk needs mesh rebuild
    dirty: bool,
    /// World-space axis-aligned bounding box
    aabb: AABB,
    /// Chunk coordinate
    coord: ChunkCoord,
}

impl Chunk {
    /// Create a new chunk filled with AIR at the given coordinate.
    pub fn new(coord: ChunkCoord) -> Self {
        Self {
            blocks: [BlockType::AIR; CHUNK_BLOCK_COUNT],
            dirty: true, // New chunks need initial mesh
            aabb: Self::compute_aabb(coord),
            coord,
        }
    }

    /// Create a chunk from existing block data.
    pub fn from_blocks(coord: ChunkCoord, blocks: [BlockType; CHUNK_BLOCK_COUNT]) -> Self {
        Self {
            blocks,
            dirty: true,
            aabb: Self::compute_aabb(coord),
            coord,
        }
    }

    /// Compute the world-space AABB for a chunk coordinate.
    fn compute_aabb(coord: ChunkCoord) -> AABB {
        let min = Vec3::new(
            (coord.x * CHUNK_SIZE as i32) as f32,
            (coord.y * CHUNK_SIZE as i32) as f32,
            (coord.z * CHUNK_SIZE as i32) as f32,
        );
        let max = min + Vec3::splat(CHUNK_SIZE as f32);
        AABB::new(min, max)
    }

    /// Get the chunk coordinate.
    #[inline]
    pub fn coord(&self) -> ChunkCoord {
        self.coord
    }

    /// Get the world-space AABB.
    #[inline]
    pub fn aabb(&self) -> &AABB {
        &self.aabb
    }

    /// Get a block at local coordinates.
    #[inline]
    pub fn get_block(&self, lx: usize, ly: usize, lz: usize) -> BlockType {
        self.blocks[local_to_index(lx, ly, lz)]
    }

    /// Set a block at local coordinates.
    #[inline]
    pub fn set_block(&mut self, lx: usize, ly: usize, lz: usize, block: BlockType) {
        self.blocks[local_to_index(lx, ly, lz)] = block;
    }

    /// Mark this chunk as needing mesh rebuild.
    #[inline]
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clear the dirty flag (after mesh rebuild).
    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Check if this chunk needs mesh rebuild.
    #[inline]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if this chunk contains only AIR blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.iter().all(|b| !b.is_solid())
    }

    /// Get direct access to block array (for meshing).
    #[inline]
    pub fn blocks(&self) -> &[BlockType; CHUNK_BLOCK_COUNT] {
        &self.blocks
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            .field("coord", &self.coord)
            .field("dirty", &self.dirty)
            .field("aabb", &self.aabb)
            .field("blocks", &format!("[{} blocks]", CHUNK_BLOCK_COUNT))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_to_index_and_back() {
        // Test all corners
        assert_eq!(local_to_index(0, 0, 0), 0);
        assert_eq!(index_to_local(0), (0, 0, 0));

        assert_eq!(local_to_index(15, 0, 0), 15);
        assert_eq!(index_to_local(15), (15, 0, 0));

        assert_eq!(local_to_index(0, 15, 0), 15 * 16);
        assert_eq!(index_to_local(15 * 16), (0, 15, 0));

        assert_eq!(local_to_index(0, 0, 15), 15 * 256);
        assert_eq!(index_to_local(15 * 256), (0, 0, 15));

        assert_eq!(local_to_index(15, 15, 15), 4095);
        assert_eq!(index_to_local(4095), (15, 15, 15));
    }

    #[test]
    fn test_local_index_roundtrip() {
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let idx = local_to_index(x, y, z);
                    let (rx, ry, rz) = index_to_local(idx);
                    assert_eq!(
                        (x, y, z),
                        (rx, ry, rz),
                        "Roundtrip failed for ({}, {}, {})",
                        x,
                        y,
                        z
                    );
                }
            }
        }
    }

    #[test]
    fn test_world_to_chunk_positive() {
        let (chunk, local) = world_to_chunk(17, 5, 32);
        assert_eq!(chunk, ChunkCoord::new(1, 0, 2));
        assert_eq!(local, (1, 5, 0));
    }

    #[test]
    fn test_world_to_chunk_negative() {
        // Negative coordinates should use floor division
        let (chunk, local) = world_to_chunk(-1, -17, -32);
        assert_eq!(chunk, ChunkCoord::new(-1, -2, -2));
        assert_eq!(local, (15, 15, 0));

        let (chunk2, local2) = world_to_chunk(-16, -16, -16);
        assert_eq!(chunk2, ChunkCoord::new(-1, -1, -1));
        assert_eq!(local2, (0, 0, 0));
    }

    #[test]
    fn test_world_to_chunk_boundary() {
        // Exactly on chunk boundaries
        let (chunk, local) = world_to_chunk(0, 0, 0);
        assert_eq!(chunk, ChunkCoord::new(0, 0, 0));
        assert_eq!(local, (0, 0, 0));

        let (chunk2, local2) = world_to_chunk(16, 16, 16);
        assert_eq!(chunk2, ChunkCoord::new(1, 1, 1));
        assert_eq!(local2, (0, 0, 0));

        let (chunk3, local3) = world_to_chunk(15, 15, 15);
        assert_eq!(chunk3, ChunkCoord::new(0, 0, 0));
        assert_eq!(local3, (15, 15, 15));
    }

    #[test]
    fn test_chunk_to_world() {
        assert_eq!(
            chunk_to_world(ChunkCoord::new(0, 0, 0), (0, 0, 0)),
            (0, 0, 0)
        );
        assert_eq!(
            chunk_to_world(ChunkCoord::new(1, 0, 2), (1, 5, 0)),
            (17, 5, 32)
        );
        assert_eq!(
            chunk_to_world(ChunkCoord::new(-1, -2, -2), (15, 15, 0)),
            (-1, -17, -32)
        );
    }

    #[test]
    fn test_coordinate_roundtrip() {
        // World -> Chunk+Local -> World should be identity
        let test_coords = [
            (0, 0, 0),
            (17, 5, 32),
            (-1, -17, -32),
            (255, -128, 1000),
            (-1000, 500, -500),
        ];

        for (x, y, z) in test_coords {
            let (chunk, local) = world_to_chunk(x, y, z);
            let (rx, ry, rz) = chunk_to_world(chunk, local);
            assert_eq!(
                (x, y, z),
                (rx, ry, rz),
                "Roundtrip failed for ({}, {}, {})",
                x,
                y,
                z
            );
        }
    }

    #[test]
    fn test_is_boundary_local() {
        // Interior - not boundary
        assert!(!is_boundary_local((1, 1, 1)));
        assert!(!is_boundary_local((7, 8, 9)));
        assert!(!is_boundary_local((14, 14, 14)));

        // Boundary cases
        assert!(is_boundary_local((0, 5, 5)));
        assert!(is_boundary_local((15, 5, 5)));
        assert!(is_boundary_local((5, 0, 5)));
        assert!(is_boundary_local((5, 15, 5)));
        assert!(is_boundary_local((5, 5, 0)));
        assert!(is_boundary_local((5, 5, 15)));

        // Corners are definitely boundary
        assert!(is_boundary_local((0, 0, 0)));
        assert!(is_boundary_local((15, 15, 15)));
    }

    #[test]
    fn test_boundary_neighbors() {
        let coord = ChunkCoord::new(5, 5, 5);

        // Interior block - no neighbors
        let neighbors = boundary_neighbors(coord, (7, 7, 7));
        assert!(neighbors.is_empty());

        // X=0 boundary
        let neighbors = boundary_neighbors(coord, (0, 7, 7));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5)));

        // X=15 boundary
        let neighbors = boundary_neighbors(coord, (15, 7, 7));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(6, 5, 5)));

        // Corner - multiple neighbors
        let neighbors = boundary_neighbors(coord, (0, 0, 0));
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5)));
        assert!(neighbors.contains(&ChunkCoord::new(5, 4, 5)));
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 4)));
    }

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new(ChunkCoord::new(1, 2, 3));
        assert_eq!(chunk.coord(), ChunkCoord::new(1, 2, 3));
        assert!(chunk.is_dirty());
        assert!(chunk.is_empty());

        // Check AABB
        let aabb = chunk.aabb();
        assert_eq!(aabb.min, Vec3::new(16.0, 32.0, 48.0));
        assert_eq!(aabb.max, Vec3::new(32.0, 48.0, 64.0));
    }

    #[test]
    fn test_chunk_get_set_block() {
        let mut chunk = Chunk::new(ChunkCoord::new(0, 0, 0));

        // All blocks start as AIR
        assert_eq!(chunk.get_block(5, 5, 5), BlockType::AIR);

        // Set and get
        chunk.set_block(5, 5, 5, BlockType::STONE);
        assert_eq!(chunk.get_block(5, 5, 5), BlockType::STONE);
        assert!(!chunk.is_empty());

        // Other blocks unaffected
        assert_eq!(chunk.get_block(0, 0, 0), BlockType::AIR);
        assert_eq!(chunk.get_block(15, 15, 15), BlockType::AIR);
    }

    #[test]
    fn test_chunk_dirty_flag() {
        let mut chunk = Chunk::new(ChunkCoord::new(0, 0, 0));

        // New chunks are dirty
        assert!(chunk.is_dirty());

        // Clear dirty
        chunk.clear_dirty();
        assert!(!chunk.is_dirty());

        // Mark dirty
        chunk.mark_dirty();
        assert!(chunk.is_dirty());
    }

    #[test]
    fn test_chunk_is_empty() {
        let mut chunk = Chunk::new(ChunkCoord::new(0, 0, 0));
        assert!(chunk.is_empty());

        chunk.set_block(0, 0, 0, BlockType::STONE);
        assert!(!chunk.is_empty());

        chunk.set_block(0, 0, 0, BlockType::AIR);
        assert!(chunk.is_empty());
    }

    // ==================== Feature 012 Tests ====================

    #[test]
    fn test_boundary_neighbors_face() {
        // [F012 T017 US2] Single-axis boundary returns exactly 1 neighbor
        let coord = ChunkCoord::new(5, 5, 5);

        // X=0 face (not on Y or Z boundary)
        let neighbors = boundary_neighbors(coord, (0, 8, 8));
        assert_eq!(
            neighbors.len(),
            1,
            "Face boundary should have exactly 1 neighbor"
        );
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5)));

        // X=15 face
        let neighbors = boundary_neighbors(coord, (15, 8, 8));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(6, 5, 5)));

        // Y=0 face
        let neighbors = boundary_neighbors(coord, (8, 0, 8));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(5, 4, 5)));

        // Y=15 face
        let neighbors = boundary_neighbors(coord, (8, 15, 8));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(5, 6, 5)));

        // Z=0 face
        let neighbors = boundary_neighbors(coord, (8, 8, 0));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 4)));

        // Z=15 face
        let neighbors = boundary_neighbors(coord, (8, 8, 15));
        assert_eq!(neighbors.len(), 1);
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 6)));
    }

    #[test]
    fn test_boundary_neighbors_edge() {
        // [F012 T018 US2] Two-axis boundary (edge) returns exactly 2 neighbors
        let coord = ChunkCoord::new(5, 5, 5);

        // X=0, Y=0 edge (Z in interior)
        let neighbors = boundary_neighbors(coord, (0, 0, 8));
        assert_eq!(
            neighbors.len(),
            2,
            "Edge boundary should have exactly 2 neighbors"
        );
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5))); // -X neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 4, 5))); // -Y neighbor

        // X=15, Z=15 edge (Y in interior)
        let neighbors = boundary_neighbors(coord, (15, 8, 15));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&ChunkCoord::new(6, 5, 5))); // +X neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 6))); // +Z neighbor

        // Y=0, Z=15 edge (X in interior)
        let neighbors = boundary_neighbors(coord, (8, 0, 15));
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&ChunkCoord::new(5, 4, 5))); // -Y neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 6))); // +Z neighbor
    }

    #[test]
    fn test_boundary_neighbors_corner() {
        // [F012 T019 US2] Three-axis boundary (corner) returns exactly 3 neighbors
        let coord = ChunkCoord::new(5, 5, 5);

        // (0, 0, 0) corner - all minimum
        let neighbors = boundary_neighbors(coord, (0, 0, 0));
        assert_eq!(
            neighbors.len(),
            3,
            "Corner boundary should have exactly 3 neighbors"
        );
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5))); // -X neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 4, 5))); // -Y neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 4))); // -Z neighbor

        // (15, 15, 15) corner - all maximum
        let neighbors = boundary_neighbors(coord, (15, 15, 15));
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&ChunkCoord::new(6, 5, 5))); // +X neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 6, 5))); // +Y neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 6))); // +Z neighbor

        // (0, 15, 0) corner - mixed
        let neighbors = boundary_neighbors(coord, (0, 15, 0));
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&ChunkCoord::new(4, 5, 5))); // -X neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 6, 5))); // +Y neighbor
        assert!(neighbors.contains(&ChunkCoord::new(5, 5, 4))); // -Z neighbor
    }
}
