//! Block World Trait
//!
//! Trait for abstracting block world access in the physics system.
//! Allows physics to work with both LoadedArena and ChunkedWorld.

use crate::types::{BlockPos, BlockType};

/// Trait for block world access.
///
/// This abstraction allows the physics system to work with different
/// world implementations (LoadedArena, ChunkedWorld, etc.)
pub trait BlockWorld {
    /// Get block at the given world position.
    /// Returns AIR for out-of-bounds or unloaded positions.
    fn get_block(&self, pos: BlockPos) -> BlockType;

    /// Set block at the given world position.
    /// May be a no-op for out-of-bounds positions.
    fn set_block(&mut self, pos: BlockPos, block: BlockType);
}

// Implement for ChunkedWorld
impl BlockWorld for crate::world::ChunkedWorld {
    fn get_block(&self, pos: BlockPos) -> BlockType {
        self.get_block(pos)
    }

    fn set_block(&mut self, pos: BlockPos, block: BlockType) {
        self.set_block(pos, block);
    }
}
