//! Liquid spreading for block physics
//!
//! Handles liquid blocks (water) spreading horizontally and downward.

use plix_common::block_physics::{
    BlockPhysicsConfig, BlockPhysicsEvent, BlockPhysicsQueue, BlockWorld,
};
use plix_common::types::{BlockPos, BlockType};

/// Resolve a liquid spread event.
///
/// Liquid spreading follows these rules:
/// 1. Vertical priority: Try to flow down first
/// 2. If can't go down, spread horizontally
/// 3. Respect maximum spread distance from source
///
/// Returns true if liquid actually spread (for metrics).
pub fn resolve_liquid_spread<W: BlockWorld>(
    event: &BlockPhysicsEvent,
    world: &mut W,
    queue: &mut BlockPhysicsQueue,
    config: &BlockPhysicsConfig,
) -> bool {
    let pos = event.pos;

    // Extract depth from event
    let depth = match event.kind {
        plix_common::block_physics::BlockPhysicsEventKind::LiquidSpread { depth } => depth,
        _ => return false,
    };

    // Get the block at this position
    let block = world.get_block(pos);

    // Only process liquid blocks
    if !block.is_liquid() {
        return false;
    }

    // Check max spread distance
    if depth >= config.max_liquid_spread_distance() {
        return false;
    }

    let mut spread_count = 0;

    // Priority 1: Try to flow down
    let below = BlockPos::new(pos.x, pos.y - 1, pos.z);
    if below.y >= 0 {
        let block_below = world.get_block(below);
        if block_below == BlockType::AIR {
            // Flow down - reset depth when going vertical
            world.set_block(below, block);
            queue.push(BlockPhysicsEvent::liquid_spread(below, 0));
            spread_count += 1;
            // Don't return early - liquid sources can still spread horizontally
        }
    }

    // Priority 2: Spread horizontally
    let neighbors = [
        BlockPos::new(pos.x + 1, pos.y, pos.z),
        BlockPos::new(pos.x - 1, pos.y, pos.z),
        BlockPos::new(pos.x, pos.y, pos.z + 1),
        BlockPos::new(pos.x, pos.y, pos.z - 1),
    ];

    for neighbor in neighbors {
        let neighbor_block = world.get_block(neighbor);
        if neighbor_block == BlockType::AIR {
            world.set_block(neighbor, block);
            queue.push(BlockPhysicsEvent::liquid_spread(neighbor, depth + 1));
            spread_count += 1;
        }
    }

    spread_count > 0
}

/// Check if a liquid block at the given position should spread.
/// Returns a liquid spread event if the block can spread, None otherwise.
pub fn detect_liquid_event<W: BlockWorld>(pos: BlockPos, world: &W) -> Option<BlockPhysicsEvent> {
    let block = world.get_block(pos);

    // Only liquid blocks can spread
    if !block.is_liquid() {
        return None;
    }

    // Check if any direction is available for spreading
    let below = BlockPos::new(pos.x, pos.y - 1, pos.z);
    if below.y >= 0 && world.get_block(below) == BlockType::AIR {
        return Some(BlockPhysicsEvent::liquid_spread(pos, 0));
    }

    // Check horizontal neighbors
    let neighbors = [
        BlockPos::new(pos.x + 1, pos.y, pos.z),
        BlockPos::new(pos.x - 1, pos.y, pos.z),
        BlockPos::new(pos.x, pos.y, pos.z + 1),
        BlockPos::new(pos.x, pos.y, pos.z - 1),
    ];

    for neighbor in neighbors {
        if world.get_block(neighbor) == BlockType::AIR {
            return Some(BlockPhysicsEvent::liquid_spread(pos, 0));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::world::ChunkedWorld;

    fn default_config() -> BlockPhysicsConfig {
        BlockPhysicsConfig::with_settings(true, true, 100, 7)
    }

    #[test]
    fn test_liquid_spreads_down() {
        let mut world = ChunkedWorld::new();
        let mut queue = BlockPhysicsQueue::new();
        let config = default_config();

        // Place water at y=5
        world.set_block(BlockPos::new(0, 5, 0), BlockType::WATER);

        let event = BlockPhysicsEvent::liquid_spread(BlockPos::new(0, 5, 0), 0);
        let spread = resolve_liquid_spread(&event, &mut world, &mut queue, &config);

        assert!(spread);
        // Water should spread down
        assert_eq!(world.get_block(BlockPos::new(0, 4, 0)), BlockType::WATER);
    }

    #[test]
    fn test_liquid_spreads_horizontally() {
        let mut world = ChunkedWorld::new();
        let mut queue = BlockPhysicsQueue::new();
        let config = default_config();

        // Place floor and water on top
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(1, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(-1, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(0, 0, 1), BlockType::STONE);
        world.set_block(BlockPos::new(0, 0, -1), BlockType::STONE);
        world.set_block(BlockPos::new(0, 1, 0), BlockType::WATER);

        let event = BlockPhysicsEvent::liquid_spread(BlockPos::new(0, 1, 0), 0);
        let spread = resolve_liquid_spread(&event, &mut world, &mut queue, &config);

        assert!(spread);
        // Check horizontal spread
        assert_eq!(world.get_block(BlockPos::new(1, 1, 0)), BlockType::WATER);
        assert_eq!(world.get_block(BlockPos::new(-1, 1, 0)), BlockType::WATER);
        assert_eq!(world.get_block(BlockPos::new(0, 1, 1)), BlockType::WATER);
        assert_eq!(world.get_block(BlockPos::new(0, 1, -1)), BlockType::WATER);
    }

    #[test]
    fn test_liquid_stops_at_max_distance() {
        let mut world = ChunkedWorld::new();
        let mut queue = BlockPhysicsQueue::new();
        let config = BlockPhysicsConfig::with_settings(true, true, 100, 3);

        // Place water at depth = max
        world.set_block(BlockPos::new(0, 1, 0), BlockType::WATER);

        let event = BlockPhysicsEvent::liquid_spread(BlockPos::new(0, 1, 0), 3);
        let spread = resolve_liquid_spread(&event, &mut world, &mut queue, &config);

        // Should not spread (at max distance)
        assert!(!spread);
    }

    #[test]
    fn test_detect_liquid_event() {
        let mut world = ChunkedWorld::new();

        // Water with air below should trigger
        world.set_block(BlockPos::new(0, 5, 0), BlockType::WATER);
        let event = detect_liquid_event(BlockPos::new(0, 5, 0), &world);
        assert!(event.is_some());

        // Non-liquid should not trigger
        world.set_block(BlockPos::new(1, 5, 0), BlockType::STONE);
        let event = detect_liquid_event(BlockPos::new(1, 5, 0), &world);
        assert!(event.is_none());
    }

    #[test]
    fn test_liquid_does_not_spread_into_solid() {
        let mut world = ChunkedWorld::new();
        let mut queue = BlockPhysicsQueue::new();
        let config = default_config();

        // Surround water with stone
        world.set_block(BlockPos::new(0, 1, 0), BlockType::WATER);
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE); // below
        world.set_block(BlockPos::new(1, 1, 0), BlockType::STONE);
        world.set_block(BlockPos::new(-1, 1, 0), BlockType::STONE);
        world.set_block(BlockPos::new(0, 1, 1), BlockType::STONE);
        world.set_block(BlockPos::new(0, 1, -1), BlockType::STONE);

        let event = BlockPhysicsEvent::liquid_spread(BlockPos::new(0, 1, 0), 0);
        let spread = resolve_liquid_spread(&event, &mut world, &mut queue, &config);

        // Should not spread (blocked by stone)
        assert!(!spread);
    }
}
