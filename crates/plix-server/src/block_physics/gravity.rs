//! Gravity resolution for block physics
//!
//! Handles falling blocks (sand, etc.)

use plix_common::block_physics::{BlockPhysicsEvent, BlockPhysicsQueue, BlockWorld};
use plix_common::types::{BlockPos, BlockType};

/// Resolve a gravity (fall) event.
///
/// If the block at `event.pos` is gravity-affected and has air below,
/// move it down one cell and re-queue for continued falling.
///
/// Returns true if a block actually fell (for metrics).
pub fn resolve_fall<W: BlockWorld>(
    event: &BlockPhysicsEvent,
    world: &mut W,
    queue: &mut BlockPhysicsQueue,
) -> bool {
    let pos = event.pos;

    // Get the block at this position
    let block = world.get_block(pos);

    // Only process gravity-affected blocks
    if !block.is_gravity_affected() {
        return false;
    }

    // Check if air below (block can fall)
    let below = BlockPos::new(pos.x, pos.y - 1, pos.z);

    // Don't fall below y=0 (world floor)
    if below.y < 0 {
        return false;
    }

    let block_below = world.get_block(below);

    // Can only fall into air or liquid
    if block_below != BlockType::AIR && !block_below.is_liquid() {
        // Landed on solid - stop falling
        return false;
    }

    // Move block down: remove from current position, place at new position
    world.set_block(pos, BlockType::AIR);
    world.set_block(below, block);

    // Re-queue for continued falling next tick
    queue.push(BlockPhysicsEvent::fall(below));

    true
}

/// Check if a block at the given position should start falling.
/// Returns a fall event if the block should fall, None otherwise.
pub fn detect_fall_event<W: BlockWorld>(pos: BlockPos, world: &W) -> Option<BlockPhysicsEvent> {
    let block = world.get_block(pos);

    // Only gravity-affected blocks can fall
    if !block.is_gravity_affected() {
        return None;
    }

    // Check if air below
    let below = BlockPos::new(pos.x, pos.y - 1, pos.z);

    // Don't check below world floor
    if below.y < 0 {
        return None;
    }

    let block_below = world.get_block(below);

    // Can fall into air or liquid
    if block_below == BlockType::AIR || block_below.is_liquid() {
        Some(BlockPhysicsEvent::fall(pos))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::world::ChunkedWorld;

    fn setup_test_world() -> ChunkedWorld {
        ChunkedWorld::new()
    }

    #[test]
    fn test_sand_falls_into_air() {
        let mut world = setup_test_world();
        let mut queue = BlockPhysicsQueue::new();

        // Place sand at y=5
        let sand_pos = BlockPos::new(0, 5, 0);
        world.set_block(sand_pos, BlockType::SAND);

        // Create fall event
        let event = BlockPhysicsEvent::fall(sand_pos);

        // Resolve - should fall
        let fell = resolve_fall(&event, &mut world, &mut queue);
        assert!(fell);

        // Check sand moved down
        assert_eq!(world.get_block(sand_pos), BlockType::AIR);
        assert_eq!(world.get_block(BlockPos::new(0, 4, 0)), BlockType::SAND);

        // Check re-queued for continued falling
        assert_eq!(queue.len(), 1);
    }

    #[test]
    fn test_sand_lands_on_solid() {
        let mut world = setup_test_world();
        let mut queue = BlockPhysicsQueue::new();

        // Place stone as floor, sand directly above
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(0, 1, 0), BlockType::SAND);

        let event = BlockPhysicsEvent::fall(BlockPos::new(0, 1, 0));
        let fell = resolve_fall(&event, &mut world, &mut queue);

        // Should not fall (stone below)
        assert!(!fell);
        assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::SAND);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_non_gravity_block_does_not_fall() {
        let mut world = setup_test_world();
        let mut queue = BlockPhysicsQueue::new();

        // Place stone in air
        let stone_pos = BlockPos::new(0, 5, 0);
        world.set_block(stone_pos, BlockType::STONE);

        let event = BlockPhysicsEvent::fall(stone_pos);
        let fell = resolve_fall(&event, &mut world, &mut queue);

        // Stone doesn't fall
        assert!(!fell);
        assert_eq!(world.get_block(stone_pos), BlockType::STONE);
    }

    #[test]
    fn test_detect_fall_event() {
        let mut world = setup_test_world();

        // Sand with air below should trigger fall
        world.set_block(BlockPos::new(0, 5, 0), BlockType::SAND);
        let event = detect_fall_event(BlockPos::new(0, 5, 0), &world);
        assert!(event.is_some());

        // Sand with solid below should not trigger
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(0, 1, 0), BlockType::SAND);
        let event = detect_fall_event(BlockPos::new(0, 1, 0), &world);
        assert!(event.is_none());

        // Stone should never trigger (not gravity-affected)
        world.set_block(BlockPos::new(1, 5, 0), BlockType::STONE);
        let event = detect_fall_event(BlockPos::new(1, 5, 0), &world);
        assert!(event.is_none());
    }

    #[test]
    fn test_sand_falls_through_water() {
        let mut world = setup_test_world();
        let mut queue = BlockPhysicsQueue::new();

        // Place water, then sand above
        world.set_block(BlockPos::new(0, 4, 0), BlockType::WATER);
        world.set_block(BlockPos::new(0, 5, 0), BlockType::SAND);

        let event = BlockPhysicsEvent::fall(BlockPos::new(0, 5, 0));
        let fell = resolve_fall(&event, &mut world, &mut queue);

        // Sand should displace water
        assert!(fell);
        assert_eq!(world.get_block(BlockPos::new(0, 5, 0)), BlockType::AIR);
        assert_eq!(world.get_block(BlockPos::new(0, 4, 0)), BlockType::SAND);
    }

    #[test]
    fn test_no_fall_below_world_floor() {
        let mut world = setup_test_world();
        let mut queue = BlockPhysicsQueue::new();

        // Place sand at y=0
        world.set_block(BlockPos::new(0, 0, 0), BlockType::SAND);

        let event = BlockPhysicsEvent::fall(BlockPos::new(0, 0, 0));
        let fell = resolve_fall(&event, &mut world, &mut queue);

        // Should not fall below y=0
        assert!(!fell);
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::SAND);
    }
}
