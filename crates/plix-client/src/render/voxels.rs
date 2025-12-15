//! Voxel/arena rendering (placeholder)

use plix_common::types::BlockType;

/// Voxel renderer (placeholder for wgpu implementation)
#[derive(Debug)]
pub struct VoxelRenderer {
    /// Arena size
    size: [u32; 3],
    /// Whether mesh needs rebuild
    dirty: bool,
}

impl VoxelRenderer {
    /// Create a new voxel renderer
    pub fn new() -> Self {
        Self {
            size: [0, 0, 0],
            dirty: false,
        }
    }

    /// Load arena data
    pub fn load_arena(&mut self, size: [u32; 3], _blocks: &[BlockType]) {
        self.size = size;
        self.dirty = true;
        // TODO: Generate mesh from blocks
    }

    /// Rebuild mesh if dirty
    pub fn update(&mut self) {
        if self.dirty {
            // TODO: Implement greedy meshing
            self.dirty = false;
        }
    }

    /// Render the arena
    pub fn render(&self) {
        // TODO: Implement wgpu rendering
    }

    /// Get arena size
    pub fn size(&self) -> [u32; 3] {
        self.size
    }
}

impl Default for VoxelRenderer {
    fn default() -> Self {
        Self::new()
    }
}
