//! SaveScheduler - Auto-save management.
//!
//! Manages periodic auto-saves with bounded I/O per cycle.

use std::collections::HashSet;
use std::time::{Duration, Instant};

use plix_common::chunk::ChunkCoord;
use plix_common::persist::PersistError;
use plix_common::world::ChunkedWorld;

use super::world_store::WorldStore;

/// Configuration for the save scheduler.
#[derive(Debug, Clone)]
pub struct SaveSchedulerConfig {
    /// Time between auto-save cycles.
    /// Default: 5 minutes (300 seconds).
    pub interval: Duration,

    /// Maximum chunks to save per cycle.
    /// Limits I/O to prevent blocking game loop.
    /// Default: 100.
    pub max_chunks_per_cycle: usize,

    /// Whether auto-save is enabled.
    /// Default: true for server, configurable for solo.
    pub enabled: bool,
}

impl Default for SaveSchedulerConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300), // 5 minutes
            max_chunks_per_cycle: 100,
            enabled: true,
        }
    }
}

impl SaveSchedulerConfig {
    /// Create config with auto-save disabled.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create config with custom interval.
    pub fn with_interval(interval: Duration) -> Self {
        Self {
            interval,
            ..Default::default()
        }
    }
}

/// Metrics tracked by the save scheduler.
#[derive(Debug, Default, Clone)]
pub struct SaveMetrics {
    /// Total chunks saved since scheduler creation.
    pub chunks_saved_total: u64,

    /// Chunks saved in the most recent cycle.
    pub chunks_saved_last_cycle: u32,

    /// Total auto-save cycles completed.
    pub save_cycles_total: u64,

    /// Total chunk save failures (individual, not cycles).
    pub save_failures_total: u64,

    /// Duration of last save cycle in milliseconds.
    pub last_cycle_duration_ms: u64,

    /// Current number of dirty chunks pending save.
    pub chunks_dirty_pending: u32,
}

impl SaveMetrics {
    /// Reset all metrics to zero.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Manages periodic auto-saves and tracks dirty chunks.
pub struct SaveScheduler {
    config: SaveSchedulerConfig,
    last_save: Instant,
    dirty_chunks: HashSet<ChunkCoord>,
    metrics: SaveMetrics,
}

impl SaveScheduler {
    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Create a new save scheduler with the given configuration.
    pub fn new(config: SaveSchedulerConfig) -> Self {
        Self {
            config,
            last_save: Instant::now(),
            dirty_chunks: HashSet::new(),
            metrics: SaveMetrics::default(),
        }
    }

    /// Create a scheduler with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(SaveSchedulerConfig::default())
    }

    // =========================================================================
    // DIRTY TRACKING
    // =========================================================================

    /// Mark a chunk as needing to be saved.
    ///
    /// Idempotent - marking an already-dirty chunk has no effect.
    pub fn mark_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.insert(coord);
        self.metrics.chunks_dirty_pending = self.dirty_chunks.len() as u32;
    }

    /// Mark multiple chunks as dirty.
    pub fn mark_dirty_many(&mut self, coords: impl IntoIterator<Item = ChunkCoord>) {
        for coord in coords {
            self.dirty_chunks.insert(coord);
        }
        self.metrics.chunks_dirty_pending = self.dirty_chunks.len() as u32;
    }

    /// Check if a chunk is marked dirty.
    pub fn is_dirty(&self, coord: ChunkCoord) -> bool {
        self.dirty_chunks.contains(&coord)
    }

    /// Get count of dirty chunks pending save.
    pub fn dirty_count(&self) -> usize {
        self.dirty_chunks.len()
    }

    /// Clear dirty flag for a chunk (called after successful save).
    pub fn clear_dirty(&mut self, coord: ChunkCoord) {
        self.dirty_chunks.remove(&coord);
        self.metrics.chunks_dirty_pending = self.dirty_chunks.len() as u32;
    }

    // =========================================================================
    // SAVE OPERATIONS
    // =========================================================================

    /// Check if auto-save is due and perform incremental save.
    ///
    /// # Returns
    /// * `Ok(saved_count)` - Number of chunks saved this tick (0 if not due).
    /// * `Err(PersistError)` - If critical I/O error.
    ///
    /// # Behavior
    /// 1. If disabled or interval not elapsed, return Ok(0).
    /// 2. Take up to `max_chunks_per_cycle` dirty chunks.
    /// 3. Save each chunk, clearing dirty flag on success.
    /// 4. Log errors for failed chunks but continue.
    /// 5. Update metrics.
    pub fn tick(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
    ) -> Result<usize, PersistError> {
        if !self.config.enabled {
            return Ok(0);
        }

        let now = Instant::now();
        if now.duration_since(self.last_save) < self.config.interval {
            return Ok(0);
        }

        self.do_save(world, store, self.config.max_chunks_per_cycle)
    }

    /// Save all dirty chunks immediately (shutdown flush).
    ///
    /// # Returns
    /// * `Ok(saved_count)` - Total chunks saved.
    /// * `Err(PersistError)` - If critical I/O error.
    pub fn flush(
        &mut self,
        world: &ChunkedWorld,
        store: &mut WorldStore,
    ) -> Result<usize, PersistError> {
        let count = self.do_save(world, store, usize::MAX)?;

        // Update metadata timestamp
        store.save_meta()?;

        Ok(count)
    }

    /// Force immediate save regardless of interval.
    ///
    /// Like flush but respects max_chunks_per_cycle.
    pub fn force_save(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
    ) -> Result<usize, PersistError> {
        self.do_save(world, store, self.config.max_chunks_per_cycle)
    }

    /// Internal save implementation.
    fn do_save(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
        max_chunks: usize,
    ) -> Result<usize, PersistError> {
        let start = Instant::now();
        let mut saved = 0;

        // Collect chunks to save (up to max)
        let to_save: Vec<ChunkCoord> = self.dirty_chunks.iter().copied().take(max_chunks).collect();

        for coord in to_save {
            if let Some(chunk) = world.get_chunk(coord) {
                match store.save_chunk(coord, chunk) {
                    Ok(()) => {
                        self.dirty_chunks.remove(&coord);
                        saved += 1;
                        self.metrics.chunks_saved_total += 1;
                    }
                    Err(e) => {
                        tracing::error!(coord = ?coord, error = %e, "Failed to save chunk");
                        self.metrics.save_failures_total += 1;
                    }
                }
            } else {
                // Chunk no longer loaded, clear from dirty set
                self.dirty_chunks.remove(&coord);
            }
        }

        self.last_save = Instant::now();
        self.metrics.save_cycles_total += 1;
        self.metrics.chunks_saved_last_cycle = saved as u32;
        self.metrics.last_cycle_duration_ms = start.elapsed().as_millis() as u64;
        self.metrics.chunks_dirty_pending = self.dirty_chunks.len() as u32;

        if saved > 0 {
            tracing::info!(
                chunks_saved = saved,
                duration_ms = self.metrics.last_cycle_duration_ms,
                remaining = self.dirty_chunks.len(),
                "Auto-save complete"
            );
        }

        Ok(saved)
    }

    // =========================================================================
    // METRICS & STATUS
    // =========================================================================

    /// Get current metrics.
    pub fn metrics(&self) -> &SaveMetrics {
        &self.metrics
    }

    /// Get time since last save.
    pub fn time_since_last_save(&self) -> Duration {
        self.last_save.elapsed()
    }

    /// Get time until next auto-save (if enabled).
    ///
    /// # Returns
    /// * `Some(duration)` - Time until next auto-save.
    /// * `None` - If disabled or overdue.
    pub fn time_until_next_save(&self) -> Option<Duration> {
        if !self.config.enabled {
            return None;
        }

        let elapsed = self.last_save.elapsed();
        if elapsed >= self.config.interval {
            None
        } else {
            Some(self.config.interval - elapsed)
        }
    }

    /// Check if scheduler is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Enable or disable auto-save.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Update configuration at runtime.
    ///
    /// Does not reset last_save time. Changes take effect on next tick.
    pub fn set_config(&mut self, config: SaveSchedulerConfig) {
        self.config = config;
    }

    /// Get current configuration.
    pub fn config(&self) -> &SaveSchedulerConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::chunk::Chunk;
    use plix_common::persist::WorldMetadata;
    use plix_common::types::BlockType;
    use tempfile::TempDir;

    fn setup_test_world() -> (TempDir, WorldStore, ChunkedWorld) {
        let temp_dir = TempDir::new().unwrap();
        let metadata =
            WorldMetadata::new_generated("test_world".to_string(), "Test World".to_string(), 12345);
        let store = WorldStore::create_world(temp_dir.path(), metadata).unwrap();
        let world = ChunkedWorld::new();
        (temp_dir, store, world)
    }

    #[test]
    fn test_mark_dirty() {
        let mut scheduler = SaveScheduler::with_defaults();

        assert_eq!(scheduler.dirty_count(), 0);

        scheduler.mark_dirty(ChunkCoord::new(0, 0, 0));
        assert_eq!(scheduler.dirty_count(), 1);
        assert!(scheduler.is_dirty(ChunkCoord::new(0, 0, 0)));

        // Marking again is idempotent
        scheduler.mark_dirty(ChunkCoord::new(0, 0, 0));
        assert_eq!(scheduler.dirty_count(), 1);
    }

    #[test]
    fn test_mark_dirty_many() {
        let mut scheduler = SaveScheduler::with_defaults();

        scheduler.mark_dirty_many([
            ChunkCoord::new(0, 0, 0),
            ChunkCoord::new(1, 0, 0),
            ChunkCoord::new(2, 0, 0),
        ]);

        assert_eq!(scheduler.dirty_count(), 3);
    }

    #[test]
    fn test_clear_dirty() {
        let mut scheduler = SaveScheduler::with_defaults();

        scheduler.mark_dirty(ChunkCoord::new(0, 0, 0));
        scheduler.mark_dirty(ChunkCoord::new(1, 0, 0));

        scheduler.clear_dirty(ChunkCoord::new(0, 0, 0));

        assert_eq!(scheduler.dirty_count(), 1);
        assert!(!scheduler.is_dirty(ChunkCoord::new(0, 0, 0)));
        assert!(scheduler.is_dirty(ChunkCoord::new(1, 0, 0)));
    }

    #[test]
    fn test_tick_disabled() {
        let (_temp_dir, store, world) = setup_test_world();

        let config = SaveSchedulerConfig::disabled();
        let mut scheduler = SaveScheduler::new(config);

        scheduler.mark_dirty(ChunkCoord::new(0, 0, 0));

        let saved = scheduler.tick(&world, &store).unwrap();
        assert_eq!(saved, 0);
        assert_eq!(scheduler.dirty_count(), 1); // Still dirty
    }

    #[test]
    fn test_tick_interval_not_elapsed() {
        let (_temp_dir, store, world) = setup_test_world();

        let config = SaveSchedulerConfig {
            interval: Duration::from_secs(3600), // 1 hour
            enabled: true,
            ..Default::default()
        };
        let mut scheduler = SaveScheduler::new(config);

        scheduler.mark_dirty(ChunkCoord::new(0, 0, 0));

        let saved = scheduler.tick(&world, &store).unwrap();
        assert_eq!(saved, 0); // Not time yet
    }

    #[test]
    fn test_force_save() {
        let (_temp_dir, store, mut world) = setup_test_world();

        let mut scheduler = SaveScheduler::with_defaults();

        // Add chunk to world
        let coord = ChunkCoord::new(0, 0, 0);
        let mut chunk = Chunk::new(coord);
        chunk.set_block(0, 0, 0, BlockType::STONE);
        world.insert_chunk(coord, chunk);

        scheduler.mark_dirty(coord);

        let saved = scheduler.force_save(&world, &store).unwrap();
        assert_eq!(saved, 1);
        assert_eq!(scheduler.dirty_count(), 0);

        // Verify saved to disk
        assert!(store.chunk_exists(coord));
    }

    #[test]
    fn test_flush() {
        let (_temp_dir, mut store, mut world) = setup_test_world();

        let mut scheduler = SaveScheduler::with_defaults();

        // Add multiple chunks
        for i in 0..5 {
            let coord = ChunkCoord::new(i, 0, 0);
            let chunk = Chunk::new(coord);
            world.insert_chunk(coord, chunk);
            scheduler.mark_dirty(coord);
        }

        let saved = scheduler.flush(&world, &mut store).unwrap();
        assert_eq!(saved, 5);
        assert_eq!(scheduler.dirty_count(), 0);
    }

    #[test]
    fn test_max_chunks_per_cycle() {
        let (_temp_dir, store, mut world) = setup_test_world();

        let config = SaveSchedulerConfig {
            interval: Duration::from_millis(0), // Immediate
            max_chunks_per_cycle: 3,
            enabled: true,
        };
        let mut scheduler = SaveScheduler::new(config);

        // Add more chunks than max
        for i in 0..10 {
            let coord = ChunkCoord::new(i, 0, 0);
            let chunk = Chunk::new(coord);
            world.insert_chunk(coord, chunk);
            scheduler.mark_dirty(coord);
        }

        let saved = scheduler.tick(&world, &store).unwrap();
        assert_eq!(saved, 3); // Only saves max_chunks_per_cycle
        assert_eq!(scheduler.dirty_count(), 7); // 10 - 3 remaining
    }

    #[test]
    fn test_metrics() {
        let (_temp_dir, store, mut world) = setup_test_world();

        let config = SaveSchedulerConfig {
            interval: Duration::from_millis(0),
            max_chunks_per_cycle: 100,
            enabled: true,
        };
        let mut scheduler = SaveScheduler::new(config);

        let coord = ChunkCoord::new(0, 0, 0);
        world.insert_chunk(coord, Chunk::new(coord));
        scheduler.mark_dirty(coord);

        scheduler.tick(&world, &store).unwrap();

        let metrics = scheduler.metrics();
        assert_eq!(metrics.chunks_saved_total, 1);
        assert_eq!(metrics.chunks_saved_last_cycle, 1);
        assert_eq!(metrics.save_cycles_total, 1);
        assert_eq!(metrics.save_failures_total, 0);
    }

    #[test]
    fn test_time_until_next_save() {
        let config = SaveSchedulerConfig {
            interval: Duration::from_secs(60),
            enabled: true,
            ..Default::default()
        };
        let scheduler = SaveScheduler::new(config);

        let time_until = scheduler.time_until_next_save();
        assert!(time_until.is_some());
        assert!(time_until.unwrap() <= Duration::from_secs(60));
    }

    #[test]
    fn test_time_until_next_save_disabled() {
        let config = SaveSchedulerConfig::disabled();
        let scheduler = SaveScheduler::new(config);

        assert!(scheduler.time_until_next_save().is_none());
    }
}
