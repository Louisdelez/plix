//! Voxel/arena rendering (placeholder)
//!
//! T036 [US1]: This module is DEPRECATED. Chunk-based rendering is now handled
//! by `chunk_mesher.rs` and integrated into `RenderEngine` via `load_chunked_world()`.
//!
//! This placeholder remains for backwards compatibility but is not used in the
//! current chunked rendering pipeline.

use plix_common::types::BlockType;

/// Voxel renderer (placeholder - DEPRECATED, see chunk_mesher.rs)
#[derive(Debug)]
#[deprecated(
    since = "0.1.0",
    note = "Use chunk_mesher.rs and RenderEngine::load_chunked_world() instead"
)]
pub struct VoxelRenderer {
    /// Arena size
    size: [u32; 3],
    /// Whether mesh needs rebuild
    dirty: bool,
}

#[allow(deprecated)]
impl VoxelRenderer {
    /// Create a new voxel renderer (DEPRECATED)
    pub fn new() -> Self {
        Self {
            size: [0, 0, 0],
            dirty: false,
        }
    }

    /// Load arena data (DEPRECATED)
    pub fn load_arena(&mut self, size: [u32; 3], _blocks: &[BlockType]) {
        self.size = size;
        self.dirty = true;
    }

    /// Rebuild mesh if dirty (DEPRECATED)
    pub fn update(&mut self) {
        if self.dirty {
            self.dirty = false;
        }
    }

    /// Render the arena (DEPRECATED)
    pub fn render(&self) {
        // No-op
    }

    /// Get arena size
    pub fn size(&self) -> [u32; 3] {
        self.size
    }
}

#[allow(deprecated)]
impl Default for VoxelRenderer {
    fn default() -> Self {
        Self::new()
    }
}
