//! Block Physics System
//!
//! Main physics simulation system for server-side block physics.

use plix_common::block_physics::{
    BlockPhysicsConfig, BlockPhysicsEvent, BlockPhysicsEventKind, BlockPhysicsMetrics,
    BlockPhysicsQueue, BlockWorld,
};
use plix_common::types::BlockPos;

use super::gravity::{detect_fall_event, resolve_fall};
use super::liquid::{detect_liquid_event, resolve_liquid_spread};

/// Main block physics simulation system.
///
/// Manages the event queue, processes physics events, and updates metrics.
/// Should be created at server startup and called each tick.
pub struct BlockPhysicsSystem {
    /// Physics configuration
    config: BlockPhysicsConfig,
    /// Event queue
    queue: BlockPhysicsQueue,
    /// Metrics
    metrics: BlockPhysicsMetrics,
}

impl BlockPhysicsSystem {
    /// Create a new physics system with the given configuration.
    pub fn new(config: BlockPhysicsConfig) -> Self {
        Self {
            config,
            queue: BlockPhysicsQueue::new(),
            metrics: BlockPhysicsMetrics::new(),
        }
    }

    /// Get current configuration (immutable).
    pub fn config(&self) -> &BlockPhysicsConfig {
        &self.config
    }

    /// Get mutable configuration (for runtime toggle).
    pub fn config_mut(&mut self) -> &mut BlockPhysicsConfig {
        &mut self.config
    }

    /// Get current metrics.
    pub fn metrics(&self) -> &BlockPhysicsMetrics {
        &self.metrics
    }

    /// Get current queue depth.
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }

    /// Queue a physics event for processing.
    pub fn queue_event(&mut self, event: BlockPhysicsEvent) {
        self.queue.push(event);
    }

    /// Detect and queue physics events at a position.
    ///
    /// Called after block edits to check if physics should trigger.
    /// Also checks the block above for gravity cascades.
    pub fn detect_events_at<W: BlockWorld>(&mut self, pos: BlockPos, world: &W) {
        // Check for gravity event at this position
        if self.config.gravity_enabled() {
            if let Some(event) = detect_fall_event(pos, world) {
                self.queue.push(event);
            }
        }

        // Check for liquid event at this position
        if self.config.liquids_enabled() {
            if let Some(event) = detect_liquid_event(pos, world) {
                self.queue.push(event);
            }
        }
    }

    /// Process physics for one tick.
    ///
    /// Drains the queue up to the configured budget, resolves events,
    /// and may add new events (e.g., continued falling).
    ///
    /// Returns the number of events processed.
    pub fn tick<W: BlockWorld>(&mut self, world: &mut W) -> u32 {
        // If physics disabled, do nothing
        if !self.config.gravity_enabled() && !self.config.liquids_enabled() {
            self.metrics.set_events_processed(0);
            self.metrics.set_queue_depth(self.queue.len() as u32);
            return 0;
        }

        let budget = self.config.max_events_per_tick();
        let mut processed = 0u32;

        while processed < budget {
            let Some(event) = self.queue.pop() else {
                break;
            };

            match event.kind {
                BlockPhysicsEventKind::Fall => {
                    if self.config.gravity_enabled() {
                        let fell = resolve_fall(&event, world, &mut self.queue);
                        if fell {
                            self.metrics.increment_blocks_fallen();
                        }
                    }
                }
                BlockPhysicsEventKind::LiquidSpread { .. } => {
                    if self.config.liquids_enabled() {
                        let spread =
                            resolve_liquid_spread(&event, world, &mut self.queue, &self.config);
                        if spread {
                            self.metrics.increment_liquid_updates();
                        }
                    }
                }
            }

            processed += 1;
        }

        // Update metrics
        self.metrics.set_events_processed(processed);
        self.metrics.set_queue_depth(self.queue.len() as u32);

        processed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::BlockType;
    use plix_common::world::ChunkedWorld;

    #[test]
    fn test_new_system() {
        let config = BlockPhysicsConfig::default();
        let system = BlockPhysicsSystem::new(config.clone());

        assert!(system.config().gravity_enabled());
        assert!(!system.config().liquids_enabled());
        assert_eq!(system.queue_depth(), 0);
    }

    #[test]
    fn test_sand_falls_when_unsupported() {
        let mut world = ChunkedWorld::new();
        // Use budget of 1 to test single-step falling
        let config = BlockPhysicsConfig::with_settings(true, false, 1, 7);
        let mut system = BlockPhysicsSystem::new(config);

        // Place sand at y=5 (nothing below)
        let sand_pos = BlockPos::new(0, 5, 0);
        world.set_block(sand_pos, BlockType::SAND);

        // Detect events
        system.detect_events_at(sand_pos, &world);
        assert_eq!(system.queue_depth(), 1);

        // Process tick - sand should fall one block (budget = 1)
        let processed = system.tick(&mut world);
        assert_eq!(processed, 1);

        // Sand moved down one block
        assert_eq!(world.get_block(sand_pos), BlockType::AIR);
        assert_eq!(world.get_block(BlockPos::new(0, 4, 0)), BlockType::SAND);

        // Re-queued for continued falling
        assert_eq!(system.queue_depth(), 1);
    }

    #[test]
    fn test_sand_lands_on_solid() {
        let mut world = ChunkedWorld::new();
        let mut system = BlockPhysicsSystem::new(BlockPhysicsConfig::default());

        // Place stone floor, sand above
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        world.set_block(BlockPos::new(0, 2, 0), BlockType::SAND);

        // Let sand fall until it lands
        system.detect_events_at(BlockPos::new(0, 2, 0), &world);

        for _ in 0..10 {
            system.tick(&mut world);
        }

        // Sand should be at y=1 (on top of stone)
        assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::SAND);
        assert_eq!(world.get_block(BlockPos::new(0, 2, 0)), BlockType::AIR);
        assert_eq!(system.queue_depth(), 0);
    }

    #[test]
    fn test_cascade_when_support_removed() {
        let mut world = ChunkedWorld::new();
        let mut system = BlockPhysicsSystem::new(BlockPhysicsConfig::default());

        // Place stone floor
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);
        // Place column of sand
        world.set_block(BlockPos::new(0, 1, 0), BlockType::SAND);
        world.set_block(BlockPos::new(0, 2, 0), BlockType::SAND);
        world.set_block(BlockPos::new(0, 3, 0), BlockType::SAND);

        // Remove the stone (simulating player break)
        world.set_block(BlockPos::new(0, 0, 0), BlockType::AIR);

        // Detect events at removed position AND above
        system.detect_events_at(BlockPos::new(0, 0, 0), &world);
        system.detect_events_at(BlockPos::new(0, 1, 0), &world);

        // Process until stable
        for _ in 0..20 {
            system.tick(&mut world);
        }

        // All sand should have fallen to the void (below y=0)
        assert_eq!(world.get_block(BlockPos::new(0, 0, 0)), BlockType::SAND);
        assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::AIR);
    }

    #[test]
    fn test_physics_disabled_blocks_stay() {
        let mut world = ChunkedWorld::new();
        let mut system = BlockPhysicsSystem::new(BlockPhysicsConfig::disabled());

        // Place sand in air
        let sand_pos = BlockPos::new(0, 5, 0);
        world.set_block(sand_pos, BlockType::SAND);

        // Detect events - should not queue because physics disabled
        system.detect_events_at(sand_pos, &world);

        // Even if we manually queue, tick should not process
        system.queue_event(BlockPhysicsEvent::fall(sand_pos));

        let processed = system.tick(&mut world);
        assert_eq!(processed, 0);

        // Sand stays in place
        assert_eq!(world.get_block(sand_pos), BlockType::SAND);
    }

    #[test]
    fn test_budget_limits_events_per_tick() {
        let mut world = ChunkedWorld::new();
        let config = BlockPhysicsConfig::with_settings(true, false, 5, 7);
        let mut system = BlockPhysicsSystem::new(config);

        // Queue 10 fall events
        for i in 0..10 {
            world.set_block(BlockPos::new(i, 10, 0), BlockType::SAND);
            system.queue_event(BlockPhysicsEvent::fall(BlockPos::new(i, 10, 0)));
        }

        assert_eq!(system.queue_depth(), 10);

        // Process tick - should only process 5 (budget)
        let processed = system.tick(&mut world);
        assert_eq!(processed, 5);

        // 5 original events processed + 5 re-queued for falling = 10 total in queue
        // Actually: 5 original processed, 5 remaining + 5 re-queued = 10
        // Wait, let me think: 10 queued, process 5, 5 remain.
        // Each processed creates 1 re-queue (if fell), so +5. Total = 5 + 5 = 10.
        assert_eq!(system.queue_depth(), 10);
    }

    #[test]
    fn test_queued_events_not_lost() {
        let mut world = ChunkedWorld::new();
        let config = BlockPhysicsConfig::with_settings(true, false, 2, 7);
        let mut system = BlockPhysicsSystem::new(config);

        // Queue 5 events
        for i in 0..5 {
            world.set_block(BlockPos::new(i, 10, 0), BlockType::SAND);
            system.queue_event(BlockPhysicsEvent::fall(BlockPos::new(i, 10, 0)));
        }

        // Process multiple ticks until all done
        let mut total_processed = 0;
        for _ in 0..50 {
            total_processed += system.tick(&mut world);
            if system.queue_depth() == 0 {
                break;
            }
        }

        // All events should eventually be processed (sand falls to y=0)
        assert!(total_processed >= 5);
        assert_eq!(system.queue_depth(), 0);
    }

    #[test]
    fn test_metrics_updated() {
        let mut world = ChunkedWorld::new();
        let mut system = BlockPhysicsSystem::new(BlockPhysicsConfig::default());

        // Place sand and let it fall
        world.set_block(BlockPos::new(0, 3, 0), BlockType::SAND);
        system.detect_events_at(BlockPos::new(0, 3, 0), &world);

        // Process ticks until sand lands at y=0
        for _ in 0..10 {
            system.tick(&mut world);
        }

        // Should have fallen 3 times (y=3 -> y=2 -> y=1 -> y=0)
        assert_eq!(system.metrics().total_blocks_fallen(), 3);
    }

    #[test]
    fn test_cross_chunk_falling() {
        let mut world = ChunkedWorld::new();
        let mut system = BlockPhysicsSystem::new(BlockPhysicsConfig::default());

        // Place stone floor in chunk (0, 0, 0)
        world.set_block(BlockPos::new(0, 0, 0), BlockType::STONE);

        // Place sand at y=17 (chunk boundary is at y=16)
        world.set_block(BlockPos::new(0, 17, 0), BlockType::SAND);
        system.detect_events_at(BlockPos::new(0, 17, 0), &world);

        // Let it fall across chunk boundary
        for _ in 0..30 {
            system.tick(&mut world);
        }

        // Sand should have landed at y=1
        assert_eq!(world.get_block(BlockPos::new(0, 1, 0)), BlockType::SAND);
        assert_eq!(world.get_block(BlockPos::new(0, 17, 0)), BlockType::AIR);
    }
}
