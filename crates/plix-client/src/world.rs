//! Client-side world state
//! T033-T034 [US1]

use plix_arena::format::LoadedArena;
use plix_common::protocol::BlockEditApplied;
use plix_common::types::{BlockPos, BlockType};

/// Client-side world wrapper with dirty tracking
pub struct ClientWorld {
    /// The loaded arena with current block state
    arena: LoadedArena,
    /// Whether the world has been modified since last mesh rebuild
    dirty: bool,
}

impl ClientWorld {
    /// Create a new client world from a loaded arena
    pub fn new(arena: LoadedArena) -> Self {
        Self {
            arena,
            dirty: true, // Initial load requires mesh build
        }
    }

    /// Get immutable reference to the arena
    pub fn arena(&self) -> &LoadedArena {
        &self.arena
    }

    /// Check if the world has been modified
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear the dirty flag (call after mesh rebuild)
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    /// Mark the world as dirty (requires mesh rebuild)
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Apply a block edit from the server
    pub fn apply_edit(&mut self, edit: &BlockEditApplied) {
        self.arena.set_block(edit.pos, edit.new_block);
        self.dirty = true;
    }

    /// Get block at position
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        self.arena.get_block_at(pos)
    }

    /// Get arena size
    pub fn size(&self) -> [u32; 3] {
        self.arena.size()
    }

    /// Check if position is in bounds
    pub fn is_in_bounds(&self, pos: BlockPos) -> bool {
        self.arena.is_in_bounds(pos)
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
}
