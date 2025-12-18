//! SaveScheduler API Contract
//!
//! This file defines the public API for auto-save scheduling.
//! Implementation in: crates/plix-server/src/persist/scheduler.rs

use std::time::{Duration, Instant};

use crate::persist::{PersistError, SaveMetrics, WorldStore};
use crate::world::ChunkedWorld;
use crate::chunk::ChunkCoord;

/// Configuration for the save scheduler.
#[derive(Debug, Clone)]
pub struct SaveSchedulerConfig {
    /// Time between auto-save cycles.
    /// Default: 5 minutes (300 seconds)
    pub interval: Duration,

    /// Maximum chunks to save per cycle.
    /// Limits I/O to prevent blocking game loop.
    /// Default: 100
    pub max_chunks_per_cycle: usize,

    /// Whether auto-save is enabled.
    /// Default: true for server, configurable for solo
    pub enabled: bool,
}

impl Default for SaveSchedulerConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(300),
            max_chunks_per_cycle: 100,
            enabled: true,
        }
    }
}

/// Manages periodic auto-saves and tracks dirty chunks.
///
/// # Usage
///
/// ```rust,ignore
/// let config = SaveSchedulerConfig::default();
/// let mut scheduler = SaveScheduler::new(config);
///
/// // In game loop:
/// loop {
///     // ... game logic ...
///
///     // After block modifications:
///     for coord in modified_chunks {
///         scheduler.mark_dirty(coord);
///     }
///
///     // Check for auto-save (non-blocking if not due)
///     scheduler.tick(&mut world, &store)?;
/// }
///
/// // On shutdown:
/// scheduler.flush(&mut world, &store)?;
/// ```
pub struct SaveScheduler {
    // Private fields - implementation detail
}

impl SaveScheduler {
    // =========================================================================
    // LIFECYCLE
    // =========================================================================

    /// Create a new save scheduler with the given configuration.
    ///
    /// # Arguments
    /// * `config` - Scheduler configuration
    ///
    /// # Returns
    /// New scheduler with no dirty chunks and last_save = now.
    pub fn new(config: SaveSchedulerConfig) -> Self;

    /// Create a scheduler with default configuration.
    pub fn with_defaults() -> Self;

    // =========================================================================
    // DIRTY TRACKING
    // =========================================================================

    /// Mark a chunk as needing to be saved.
    ///
    /// # Arguments
    /// * `coord` - Chunk coordinates
    ///
    /// # Thread Safety
    /// Not thread-safe. Call from main game thread only.
    ///
    /// # Deduplication
    /// Idempotent - marking an already-dirty chunk has no effect.
    pub fn mark_dirty(&mut self, coord: ChunkCoord);

    /// Mark multiple chunks as dirty.
    ///
    /// # Arguments
    /// * `coords` - Iterator of chunk coordinates
    pub fn mark_dirty_many(&mut self, coords: impl IntoIterator<Item = ChunkCoord>);

    /// Check if a chunk is marked dirty.
    pub fn is_dirty(&self, coord: ChunkCoord) -> bool;

    /// Get count of dirty chunks pending save.
    pub fn dirty_count(&self) -> usize;

    /// Clear dirty flag for a chunk (called after successful save).
    ///
    /// # Note
    /// Normally called internally by tick/flush. Exposed for testing.
    pub fn clear_dirty(&mut self, coord: ChunkCoord);

    // =========================================================================
    // SAVE OPERATIONS
    // =========================================================================

    /// Check if auto-save is due and perform incremental save.
    ///
    /// # Arguments
    /// * `world` - World to read chunks from
    /// * `store` - Storage to write chunks to
    ///
    /// # Returns
    /// * `Ok(saved_count)` - Number of chunks saved this tick (0 if not due)
    /// * `Err(PersistError)` - If critical I/O error (individual chunk errors logged but don't fail)
    ///
    /// # Behavior
    /// 1. If disabled or interval not elapsed, return Ok(0)
    /// 2. Take up to `max_chunks_per_cycle` dirty chunks
    /// 3. Save each chunk, clearing dirty flag on success
    /// 4. Log errors for failed chunks but continue
    /// 5. Update metrics
    ///
    /// # Performance
    /// * Non-blocking if interval not elapsed (just timestamp check)
    /// * Bounded I/O when saving (max_chunks_per_cycle)
    /// * Remaining dirty chunks saved in subsequent ticks
    pub fn tick(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
    ) -> Result<usize, PersistError>;

    /// Save all dirty chunks immediately (shutdown flush).
    ///
    /// # Arguments
    /// * `world` - World to read chunks from
    /// * `store` - Storage to write chunks to
    ///
    /// # Returns
    /// * `Ok(saved_count)` - Total chunks saved
    /// * `Err(PersistError)` - If critical I/O error
    ///
    /// # Behavior
    /// 1. Save ALL dirty chunks (ignores max_chunks_per_cycle)
    /// 2. Update world metadata with last_saved timestamp
    /// 3. Clear all dirty flags
    /// 4. Update metrics
    ///
    /// # When to Call
    /// * Server graceful shutdown
    /// * Solo game quit/menu
    /// * Manual save command
    pub fn flush(
        &mut self,
        world: &ChunkedWorld,
        store: &mut WorldStore,
    ) -> Result<usize, PersistError>;

    /// Force immediate save regardless of interval.
    ///
    /// Like flush but respects max_chunks_per_cycle.
    /// Useful for testing or manual save that shouldn't block.
    pub fn force_save(
        &mut self,
        world: &ChunkedWorld,
        store: &WorldStore,
    ) -> Result<usize, PersistError>;

    // =========================================================================
    // METRICS & STATUS
    // =========================================================================

    /// Get current metrics.
    pub fn metrics(&self) -> &SaveMetrics;

    /// Get time since last save.
    pub fn time_since_last_save(&self) -> Duration;

    /// Get time until next auto-save (if enabled).
    ///
    /// # Returns
    /// * `Some(duration)` - Time until next auto-save
    /// * `None` - If disabled or overdue
    pub fn time_until_next_save(&self) -> Option<Duration>;

    /// Check if scheduler is enabled.
    pub fn is_enabled(&self) -> bool;

    /// Enable or disable auto-save.
    pub fn set_enabled(&mut self, enabled: bool);

    /// Update configuration at runtime.
    ///
    /// # Note
    /// Does not reset last_save time. Changes take effect on next tick.
    pub fn set_config(&mut self, config: SaveSchedulerConfig);
}

// =============================================================================
// METRICS
// =============================================================================

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
    pub fn reset(&mut self);
}
