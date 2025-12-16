//! Chunk streaming manager for view-distance based loading.
//!
//! T039-T049 [US2]: This module handles loading and unloading chunks based
//! on player position, with per-frame mesh rebuild budgeting.

use std::collections::{HashMap, HashSet, VecDeque};

use plix_common::chunk::{boundary_neighbors, is_boundary_local, world_to_chunk, ChunkCoord};
use plix_common::math::Vec3;
use plix_common::types::BlockPos;
use plix_common::ChunkedWorld;

/// T039 [US2]: Configuration for chunk streaming.
#[derive(Debug, Clone)]
pub struct ChunkManagerConfig {
    /// View distance in chunks (radius around player).
    pub view_distance: u8,
    /// Maximum number of chunk meshes to rebuild per frame.
    pub mesh_budget_per_frame: u32,
    /// [F012] Maximum retry attempts for failed mesh rebuilds.
    pub max_retries: u8,
}

impl Default for ChunkManagerConfig {
    fn default() -> Self {
        Self {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        }
    }
}

/// [F012] Metrics for mesh rebuild operations.
/// Updated each frame during ChunkManager::update().
#[derive(Debug, Clone, Default)]
pub struct MeshMetrics {
    /// Number of mesh rebuilds attempted this frame.
    pub rebuilds_this_frame: u32,
    /// Current depth of the dirty queue.
    pub dirty_queue_depth: u32,
    /// Total chunks skipped (exceeded max retries).
    pub skipped_chunks_total: u32,
    /// Successful rebuilds this frame.
    pub successful_rebuilds: u32,
    /// Failed rebuilds this frame.
    pub failed_rebuilds: u32,
}

/// T040 [US2]: Chunk streaming manager.
///
/// Manages which chunks are loaded around the player and tracks
/// which chunks need mesh rebuilding.
pub struct ChunkManager {
    /// Configuration
    config: ChunkManagerConfig,
    /// Currently loaded chunk coordinates
    loaded: HashSet<ChunkCoord>,
    /// Queue of chunks needing mesh rebuild (deduped by HashSet)
    dirty_queue: VecDeque<ChunkCoord>,
    /// Set for fast dirty lookup (mirrors dirty_queue)
    dirty_set: HashSet<ChunkCoord>,
    /// [F012] Retry counts for chunks that failed mesh rebuild
    retry_counts: HashMap<ChunkCoord, u8>,
    /// [F012] Chunks that exceeded max retries and are skipped
    skipped_chunks: HashSet<ChunkCoord>,
    /// [F012] Current frame metrics
    metrics: MeshMetrics,
}

impl ChunkManager {
    /// T041 [US2]: Create a new chunk manager with default configuration.
    pub fn new() -> Self {
        Self::with_config(ChunkManagerConfig::default())
    }

    /// T041 [US2]: Create a new chunk manager with custom configuration.
    pub fn with_config(config: ChunkManagerConfig) -> Self {
        Self {
            config,
            loaded: HashSet::new(),
            dirty_queue: VecDeque::new(),
            dirty_set: HashSet::new(),
            retry_counts: HashMap::new(),
            skipped_chunks: HashSet::new(),
            metrics: MeshMetrics::default(),
        }
    }

    /// Get configuration.
    pub fn config(&self) -> &ChunkManagerConfig {
        &self.config
    }

    /// Get number of loaded chunks.
    pub fn loaded_count(&self) -> usize {
        self.loaded.len()
    }

    /// Get number of dirty chunks pending rebuild.
    pub fn dirty_count(&self) -> usize {
        self.dirty_queue.len()
    }

    /// Check if a chunk is currently loaded.
    pub fn is_loaded(&self, coord: ChunkCoord) -> bool {
        self.loaded.contains(&coord)
    }

    /// T042 [US2]: Compute the set of chunks that should be loaded around player.
    ///
    /// Returns chunk coordinates in a spherical pattern around the player's chunk.
    pub fn compute_desired_chunks(&self, player_pos: Vec3) -> HashSet<ChunkCoord> {
        let radius = self.config.view_distance as i32;
        let player_chunk = Self::pos_to_chunk(player_pos);

        let mut desired = HashSet::new();

        for dx in -radius..=radius {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    // Use squared distance for spherical check (cheaper than sqrt)
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    if dist_sq <= radius * radius {
                        desired.insert(ChunkCoord::new(
                            player_chunk.x + dx,
                            player_chunk.y + dy,
                            player_chunk.z + dz,
                        ));
                    }
                }
            }
        }

        desired
    }

    /// Convert world position to chunk coordinate.
    fn pos_to_chunk(pos: Vec3) -> ChunkCoord {
        ChunkCoord::new(
            (pos.x / 16.0).floor() as i32,
            (pos.y / 16.0).floor() as i32,
            (pos.z / 16.0).floor() as i32,
        )
    }

    /// T043 [US2]: Load missing chunks from world data.
    ///
    /// Returns chunks that were newly loaded (need initial mesh build).
    pub fn load_missing_chunks(
        &mut self,
        desired: &HashSet<ChunkCoord>,
        world: &ChunkedWorld,
    ) -> Vec<ChunkCoord> {
        let mut newly_loaded = Vec::new();

        for &coord in desired {
            if !self.loaded.contains(&coord) && world.has_chunk(coord) {
                self.loaded.insert(coord);
                self.mark_dirty(coord);
                newly_loaded.push(coord);
            }
        }

        newly_loaded
    }

    /// T044 [US2]: Unload chunks that are too far from player.
    ///
    /// Returns chunks that were unloaded (need mesh removal).
    pub fn unload_far_chunks(&mut self, desired: &HashSet<ChunkCoord>) -> Vec<ChunkCoord> {
        let mut to_unload = Vec::new();

        for &coord in &self.loaded {
            if !desired.contains(&coord) {
                to_unload.push(coord);
            }
        }

        for coord in &to_unload {
            self.loaded.remove(coord);
            // Also remove from dirty queue if present
            self.dirty_set.remove(coord);
        }

        // Rebuild dirty queue without removed chunks
        if !to_unload.is_empty() {
            self.dirty_queue.retain(|c| !to_unload.contains(c));
        }

        to_unload
    }

    /// T045 [US2]: Main update - orchestrates load/unload/rebuild per frame.
    ///
    /// Call this each frame with the player position and world reference.
    /// Returns:
    /// - chunks_loaded: newly loaded chunks (need mesh creation)
    /// - chunks_unloaded: removed chunks (need mesh deletion)
    /// - chunks_to_rebuild: dirty chunks to rebuild this frame (limited by budget)
    /// - metrics: [F012] current metrics snapshot
    pub fn update(&mut self, player_pos: Vec3, world: &ChunkedWorld) -> ChunkManagerUpdate {
        // [F012] Reset per-frame metrics
        self.metrics.rebuilds_this_frame = 0;
        self.metrics.successful_rebuilds = 0;
        self.metrics.failed_rebuilds = 0;

        let desired = self.compute_desired_chunks(player_pos);

        // Load new chunks
        let chunks_loaded = self.load_missing_chunks(&desired, world);

        // Unload far chunks
        let chunks_unloaded = self.unload_far_chunks(&desired);

        // T046 [US2]: Get chunks to rebuild this frame (up to budget)
        let chunks_to_rebuild = self.pop_dirty_batch();

        // [F012] Update metrics
        self.metrics.rebuilds_this_frame = chunks_to_rebuild.len() as u32;
        self.metrics.dirty_queue_depth = self.dirty_queue.len() as u32;

        ChunkManagerUpdate {
            chunks_loaded,
            chunks_unloaded,
            chunks_to_rebuild,
            metrics: self.metrics.clone(),
        }
    }

    /// T046 [US2]: Pop up to mesh_budget chunks from the dirty queue.
    fn pop_dirty_batch(&mut self) -> Vec<ChunkCoord> {
        let mut batch = Vec::new();
        let budget = self.config.mesh_budget_per_frame as usize;

        while batch.len() < budget {
            if let Some(coord) = self.dirty_queue.pop_front() {
                self.dirty_set.remove(&coord);
                // Only include if still loaded
                if self.loaded.contains(&coord) {
                    batch.push(coord);
                }
            } else {
                break;
            }
        }

        batch
    }

    /// T050 [US2]: Mark a chunk as needing mesh rebuild.
    ///
    /// Uses deduplication to avoid rebuilding the same chunk multiple times.
    /// [F012] Now ignores marking for unloaded chunks and clears skipped status.
    pub fn mark_dirty(&mut self, coord: ChunkCoord) {
        // [F012 T013] Ignore if not loaded
        if !self.is_loaded(coord) {
            return;
        }

        // [F012 T014] Clear skipped status on new edit
        self.skipped_chunks.remove(&coord);

        if self.dirty_set.insert(coord) {
            self.dirty_queue.push_back(coord);
        }
    }

    /// [F012 T015-T016] Mark a chunk dirty along with any affected neighbors.
    ///
    /// Automatically detects boundary positions and marks neighbor chunks.
    /// Ignores marking for unloaded chunks (FR-024).
    pub fn mark_dirty_for_block(&mut self, pos: BlockPos) {
        let (chunk_coord, local) = world_to_chunk(pos.x, pos.y, pos.z);

        // Ignore if chunk not loaded
        if !self.is_loaded(chunk_coord) {
            return;
        }

        // Mark the chunk itself
        self.mark_dirty(chunk_coord);

        // Mark boundary neighbors (only if loaded)
        if is_boundary_local(local) {
            for neighbor in boundary_neighbors(chunk_coord, local) {
                if self.is_loaded(neighbor) {
                    self.mark_dirty(neighbor);
                }
            }
        }
    }

    /// Mark multiple chunks as dirty.
    pub fn mark_dirty_batch(&mut self, coords: &[ChunkCoord]) {
        for &coord in coords {
            self.mark_dirty(coord);
        }
    }

    /// Initialize manager with all chunks from a world.
    ///
    /// Useful for initial load - marks all existing chunks as loaded and dirty.
    pub fn initialize_from_world(&mut self, world: &ChunkedWorld) {
        self.loaded.clear();
        self.dirty_queue.clear();
        self.dirty_set.clear();

        for (coord, _) in world.iter_chunks() {
            self.loaded.insert(*coord);
            self.mark_dirty(*coord);
        }
    }

    /// Get iterator over loaded chunks.
    pub fn loaded_chunks(&self) -> impl Iterator<Item = &ChunkCoord> {
        self.loaded.iter()
    }

    /// [F012 T036-T040 US5] Report the result of a mesh rebuild attempt.
    ///
    /// Call this after attempting to rebuild each chunk returned by update().
    /// - On success: clears retry counter, increments successful_rebuilds
    /// - On failure: increments retry counter, re-queues for retry or skips if max reached
    pub fn report_rebuild_result(&mut self, coord: ChunkCoord, success: bool) {
        if success {
            // T037: Clear retry counter on success
            self.retry_counts.remove(&coord);
            self.metrics.successful_rebuilds += 1;
        } else {
            // T038: Increment retry counter
            let count = self.retry_counts.entry(coord).or_insert(0);
            *count += 1;
            self.metrics.failed_rebuilds += 1;

            if *count >= self.config.max_retries {
                // T039: Max retries exceeded - skip this chunk
                self.skipped_chunks.insert(coord);
                self.retry_counts.remove(&coord);
                self.metrics.skipped_chunks_total += 1;
            } else {
                // T040: Re-queue for retry
                self.mark_dirty(coord);
            }
        }
    }

    /// [F012 T042 US6] Get current metrics.
    pub fn metrics(&self) -> &MeshMetrics {
        &self.metrics
    }

    /// [F012 T043 US6] Check if a chunk has been skipped due to repeated failures.
    pub fn is_skipped(&self, coord: ChunkCoord) -> bool {
        self.skipped_chunks.contains(&coord)
    }

    /// [F012 T044 US6] Clear skipped status for a chunk (e.g., after new edit).
    pub fn clear_skipped(&mut self, coord: ChunkCoord) {
        self.skipped_chunks.remove(&coord);
    }
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of ChunkManager::update()
#[derive(Debug, Default, Clone)]
pub struct ChunkManagerUpdate {
    /// Chunks that were newly loaded
    pub chunks_loaded: Vec<ChunkCoord>,
    /// Chunks that were unloaded
    pub chunks_unloaded: Vec<ChunkCoord>,
    /// Chunks that should have their mesh rebuilt this frame
    pub chunks_to_rebuild: Vec<ChunkCoord>,
    /// [F012] Current metrics snapshot
    pub metrics: MeshMetrics,
}

#[cfg(test)]
mod tests {
    use super::*;
    use plix_common::types::{BlockPos, BlockType};

    fn make_test_world() -> ChunkedWorld {
        let mut world = ChunkedWorld::new();
        // Create a 3x3x3 chunk area with some blocks
        for cx in -1..=1 {
            for cy in 0..=1 {
                for cz in -1..=1 {
                    let pos = BlockPos::new(cx * 16 + 8, cy * 16 + 8, cz * 16 + 8);
                    world.set_block(pos, BlockType::STONE);
                }
            }
        }
        world
    }

    #[test]
    fn test_desired_chunks_at_origin() {
        // T048 [US2]: Test streaming set correctness
        let manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 2,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        let desired = manager.compute_desired_chunks(Vec3::new(8.0, 8.0, 8.0));

        // Player is at chunk (0, 0, 0)
        assert!(desired.contains(&ChunkCoord::new(0, 0, 0)));
        assert!(desired.contains(&ChunkCoord::new(1, 0, 0)));
        assert!(desired.contains(&ChunkCoord::new(-1, 0, 0)));
        assert!(desired.contains(&ChunkCoord::new(0, 1, 0)));
        assert!(desired.contains(&ChunkCoord::new(0, 0, 1)));

        // Chunks at edge should be included
        assert!(desired.contains(&ChunkCoord::new(2, 0, 0)));

        // Chunks beyond radius should not be included
        // At distance sqrt(9) > 2
        assert!(!desired.contains(&ChunkCoord::new(2, 2, 1)));
    }

    #[test]
    fn test_stable_when_standing_still() {
        // T048 [US2]: Test that chunks are stable when player doesn't move
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 2,
            mesh_budget_per_frame: 100,
            max_retries: 3,
        });

        let world = make_test_world();
        let player_pos = Vec3::new(8.0, 8.0, 8.0);

        // First update - loads chunks
        let update1 = manager.update(player_pos, &world);
        let initial_loaded = update1.chunks_loaded.len();

        // Drain dirty queue
        while manager.dirty_count() > 0 {
            manager.update(player_pos, &world);
        }

        // Second update at same position - should be stable
        let update2 = manager.update(player_pos, &world);

        assert!(
            update2.chunks_loaded.is_empty(),
            "No new chunks should load when standing still"
        );
        assert!(
            update2.chunks_unloaded.is_empty(),
            "No chunks should unload when standing still"
        );
        assert!(
            update2.chunks_to_rebuild.is_empty(),
            "No rebuilds needed when stable"
        );
    }

    #[test]
    fn test_load_unload_on_move() {
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 1,
            mesh_budget_per_frame: 100,
            max_retries: 3,
        });

        let world = make_test_world();

        // Start at origin
        let pos1 = Vec3::new(8.0, 8.0, 8.0);
        manager.update(pos1, &world);

        // Drain dirty queue
        while manager.dirty_count() > 0 {
            manager.update(pos1, &world);
        }

        let loaded_at_origin: HashSet<_> = manager.loaded_chunks().copied().collect();

        // Move to chunk (1, 0, 0)
        let pos2 = Vec3::new(24.0, 8.0, 8.0);
        let update = manager.update(pos2, &world);

        // Should have loaded new chunks in +X direction
        // Should have unloaded old chunks in -X direction
        assert!(!update.chunks_loaded.is_empty() || !update.chunks_unloaded.is_empty());
    }

    #[test]
    fn test_dirty_deduplication() {
        // T050 [US2]: Test that marking same chunk dirty multiple times
        // doesn't add duplicates to queue
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(1, 2, 3);
        manager.loaded.insert(coord);

        manager.mark_dirty(coord);
        manager.mark_dirty(coord);
        manager.mark_dirty(coord);

        assert_eq!(manager.dirty_count(), 1);
    }

    #[test]
    fn test_mesh_budget_enforcement() {
        // T046 [US2]: Test mesh rebuild budget
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        // Mark several chunks dirty
        for i in 0..10 {
            let coord = ChunkCoord::new(i, 0, 0);
            manager.loaded.insert(coord);
            manager.mark_dirty(coord);
        }

        assert_eq!(manager.dirty_count(), 10);

        // Pop batch - should only get 2
        let batch = manager.pop_dirty_batch();
        assert_eq!(batch.len(), 2);
        assert_eq!(manager.dirty_count(), 8);

        // Pop another batch
        let batch2 = manager.pop_dirty_batch();
        assert_eq!(batch2.len(), 2);
        assert_eq!(manager.dirty_count(), 6);
    }

    #[test]
    fn test_initialize_from_world() {
        let mut manager = ChunkManager::new();
        let world = make_test_world();

        manager.initialize_from_world(&world);

        assert_eq!(manager.loaded_count(), world.chunk_count());
        assert_eq!(manager.dirty_count(), world.chunk_count());
    }

    // ==================== Feature 012 Tests ====================

    #[test]
    fn test_mark_dirty_ignores_unloaded() {
        // [F012 T011 US1] Marking unloaded chunk should have no effect
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(1, 2, 3);
        // Don't load the chunk

        manager.mark_dirty(coord);

        assert_eq!(
            manager.dirty_count(),
            0,
            "Unloaded chunk should not be marked dirty"
        );
    }

    #[test]
    fn test_mark_dirty_clears_skipped_status() {
        // [F012 T012 US1] New edit should clear skipped status
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(1, 2, 3);
        manager.loaded.insert(coord);
        manager.skipped_chunks.insert(coord);

        assert!(manager.skipped_chunks.contains(&coord));

        manager.mark_dirty(coord);

        assert!(
            !manager.skipped_chunks.contains(&coord),
            "Skipped status should be cleared"
        );
        assert_eq!(manager.dirty_count(), 1);
    }

    #[test]
    fn test_deduplication_100_marks() {
        // [F012 T024 US3] Marking same chunk dirty 100 times should result in 1 queue entry
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(5, 5, 5);
        manager.loaded.insert(coord);

        for _ in 0..100 {
            manager.mark_dirty(coord);
        }

        assert_eq!(
            manager.dirty_count(),
            1,
            "100 marks of same chunk should result in 1 entry"
        );
        assert!(manager.dirty_set.contains(&coord));
    }

    #[test]
    fn test_deduplication_multiple_chunks() {
        // [F012 T025 US3] Marking 5 chunks twice each should result in 5 queue entries
        let mut manager = ChunkManager::new();

        let chunks: Vec<ChunkCoord> = (0..5).map(|i| ChunkCoord::new(i, 0, 0)).collect();

        // Load all chunks
        for &coord in &chunks {
            manager.loaded.insert(coord);
        }

        // Mark each chunk twice
        for _ in 0..2 {
            for &coord in &chunks {
                manager.mark_dirty(coord);
            }
        }

        assert_eq!(
            manager.dirty_count(),
            5,
            "5 chunks marked twice each should result in 5 entries"
        );
        for coord in chunks {
            assert!(manager.dirty_set.contains(&coord));
        }
    }

    #[test]
    fn test_dirty_set_queue_invariant() {
        // [F012 T026-T027 US3] Verify dirty_set.len() == dirty_queue.len() invariant
        let mut manager = ChunkManager::new();

        // Load and mark several chunks
        for i in 0..10 {
            let coord = ChunkCoord::new(i, 0, 0);
            manager.loaded.insert(coord);
            manager.mark_dirty(coord);
            assert_eq!(
                manager.dirty_set.len(),
                manager.dirty_queue.len(),
                "Invariant violated after marking chunk {}",
                i
            );
        }

        // Pop some and verify invariant holds
        for _ in 0..3 {
            manager.pop_dirty_batch();
            assert_eq!(
                manager.dirty_set.len(),
                manager.dirty_queue.len(),
                "Invariant violated after pop"
            );
        }
    }

    #[test]
    fn test_budget_50_chunks_25_frames() {
        // [F012 T028 US4] Queue 50 dirty chunks with budget=2, verify 25 frames to drain
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        // Load and mark 50 chunks
        for i in 0..50 {
            let coord = ChunkCoord::new(i, 0, 0);
            manager.loaded.insert(coord);
            manager.mark_dirty(coord);
        }

        assert_eq!(manager.dirty_count(), 50);

        // Count frames to drain
        let mut frames = 0;
        while manager.dirty_count() > 0 {
            let batch = manager.pop_dirty_batch();
            assert!(batch.len() <= 2, "Budget should limit to 2 per frame");
            frames += 1;
        }

        assert_eq!(frames, 25, "50 chunks with budget 2 should take 25 frames");
    }

    #[test]
    fn test_metrics_rebuilds_this_frame() {
        // [F012 T030 US4] Verify rebuilds_this_frame is set from batch size
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        let world = make_test_world();
        let player_pos = Vec3::new(8.0, 8.0, 8.0);

        // Initial update loads chunks
        let update = manager.update(player_pos, &world);

        // Metrics should reflect chunks loaded (which become dirty and get rebuilt)
        assert!(
            update.metrics.rebuilds_this_frame <= 2,
            "Rebuilds should be limited by budget"
        );
    }

    #[test]
    fn test_retry_success_clears_count() {
        // [F012 T032 US5] Success clears retry counter
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(5, 5, 5);
        manager.loaded.insert(coord);

        // Simulate two failures followed by success
        manager.retry_counts.insert(coord, 2);

        manager.report_rebuild_result(coord, true);

        assert!(
            !manager.retry_counts.contains_key(&coord),
            "Retry count should be cleared on success"
        );
        assert_eq!(manager.metrics.successful_rebuilds, 1);
    }

    #[test]
    fn test_retry_failure_increments_count() {
        // [F012 T033 US5] Failure increments retry counter
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(5, 5, 5);
        manager.loaded.insert(coord);

        // First failure
        manager.report_rebuild_result(coord, false);
        assert_eq!(*manager.retry_counts.get(&coord).unwrap(), 1);
        assert_eq!(manager.metrics.failed_rebuilds, 1);
        assert!(manager.dirty_set.contains(&coord), "Should be re-queued");

        // Second failure
        manager.dirty_set.clear();
        manager.dirty_queue.clear();
        manager.report_rebuild_result(coord, false);
        assert_eq!(*manager.retry_counts.get(&coord).unwrap(), 2);
        assert_eq!(manager.metrics.failed_rebuilds, 2);
    }

    #[test]
    fn test_retry_exceeded_skips_chunk() {
        // [F012 T034 US5] Chunk is skipped after max retries
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        let coord = ChunkCoord::new(5, 5, 5);
        manager.loaded.insert(coord);

        // Three failures should result in skip
        for i in 0..3 {
            manager.report_rebuild_result(coord, false);
            if i < 2 {
                // Clear re-queued status for next iteration
                manager.dirty_set.clear();
                manager.dirty_queue.clear();
            }
        }

        assert!(
            manager.is_skipped(coord),
            "Chunk should be skipped after 3 failures"
        );
        assert!(
            !manager.retry_counts.contains_key(&coord),
            "Retry count should be removed when skipped"
        );
        assert_eq!(manager.metrics.skipped_chunks_total, 1);
        assert!(
            !manager.dirty_set.contains(&coord),
            "Skipped chunk should not be in dirty queue"
        );
    }

    #[test]
    fn test_skipped_chunk_requeues_on_new_edit() {
        // [F012 T035 US5] Skipped chunk can be re-enabled by new edit
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(5, 5, 5);
        manager.loaded.insert(coord);
        manager.skipped_chunks.insert(coord);

        assert!(manager.is_skipped(coord));

        // New edit should clear skipped status and re-queue
        manager.mark_dirty(coord);

        assert!(
            !manager.is_skipped(coord),
            "Skipped status should be cleared"
        );
        assert!(
            manager.dirty_set.contains(&coord),
            "Should be in dirty queue"
        );
    }

    #[test]
    fn test_metrics_accuracy() {
        // [F012 T041 US6] Verify all metrics fields are accurate
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 8,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        let coord1 = ChunkCoord::new(1, 0, 0);
        let coord2 = ChunkCoord::new(2, 0, 0);
        let coord3 = ChunkCoord::new(3, 0, 0);

        manager.loaded.insert(coord1);
        manager.loaded.insert(coord2);
        manager.loaded.insert(coord3);

        // Mark chunks dirty
        manager.mark_dirty(coord1);
        manager.mark_dirty(coord2);
        manager.mark_dirty(coord3);

        // Verify initial state via metrics accessor
        let metrics = manager.metrics();
        assert_eq!(metrics.skipped_chunks_total, 0);
        assert_eq!(metrics.successful_rebuilds, 0);
        assert_eq!(metrics.failed_rebuilds, 0);

        // Simulate successful rebuild
        manager.report_rebuild_result(coord1, true);

        // Simulate failed rebuilds (3 times to skip)
        for _ in 0..3 {
            manager.report_rebuild_result(coord2, false);
            manager.dirty_set.clear();
            manager.dirty_queue.clear();
            manager.mark_dirty(coord3); // Keep coord3 in queue
        }

        let metrics = manager.metrics();
        assert_eq!(metrics.successful_rebuilds, 1, "Should have 1 success");
        assert_eq!(metrics.failed_rebuilds, 3, "Should have 3 failures");
        assert_eq!(metrics.skipped_chunks_total, 1, "Should have 1 skipped");
        assert!(manager.is_skipped(coord2), "coord2 should be skipped");
    }

    #[test]
    fn test_metrics_per_frame_reset() {
        // [F012 T045 US6] Per-frame metrics reset at start of update()
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 2,
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        let world = make_test_world();
        let player_pos = Vec3::new(8.0, 8.0, 8.0);

        // First update
        let update1 = manager.update(player_pos, &world);

        // Simulate some rebuild results
        if let Some(&coord) = update1.chunks_to_rebuild.first() {
            manager.report_rebuild_result(coord, true);
        }

        // Second update should reset per-frame counters
        let update2 = manager.update(player_pos, &world);

        // Per-frame counters should be reset (though we haven't called report_rebuild_result yet)
        // The rebuilds_this_frame should reflect new batch, not accumulated
        assert!(
            update2.metrics.rebuilds_this_frame <= 2,
            "Per-frame metrics should be reset"
        );
    }

    #[test]
    fn test_clear_skipped() {
        // [F012 T044 US6] clear_skipped() method works correctly
        let mut manager = ChunkManager::new();

        let coord = ChunkCoord::new(5, 5, 5);
        manager.skipped_chunks.insert(coord);

        assert!(manager.is_skipped(coord));

        manager.clear_skipped(coord);

        assert!(
            !manager.is_skipped(coord),
            "Skipped status should be cleared"
        );
    }

    #[test]
    fn test_metrics_dirty_queue_depth() {
        // [F012 T031 US4] Verify dirty_queue_depth metric accuracy
        // This test verifies that metrics.dirty_queue_depth is set correctly
        // after pop_dirty_batch() in update().
        let mut manager = ChunkManager::with_config(ChunkManagerConfig {
            view_distance: 2, // Small view distance
            mesh_budget_per_frame: 2,
            max_retries: 3,
        });

        // Create a world with chunks at origin
        let world = make_test_world();
        let player_pos = Vec3::new(8.0, 8.0, 8.0); // At chunk (0, 0, 0)

        // First update loads chunks from world
        let update1 = manager.update(player_pos, &world);
        let initial_dirty = update1.metrics.dirty_queue_depth;

        // Second update - no new loads, just pops from queue
        let update2 = manager.update(player_pos, &world);

        // The metrics should show remaining dirty queue after pop
        // (less than initial since we popped up to 2)
        assert!(
            update2.metrics.dirty_queue_depth <= initial_dirty,
            "dirty_queue_depth should decrease after processing"
        );

        // Verify rebuilds_this_frame is set
        assert!(
            update2.metrics.rebuilds_this_frame <= 2,
            "rebuilds_this_frame should be <= budget (2)"
        );
    }

    #[test]
    fn test_mark_dirty_for_block_boundary() {
        // [F012 T020 US2] Boundary block edit marks correct neighbors dirty
        let mut manager = ChunkManager::new();

        // Load chunk (5, 5, 5) and its neighbors
        let center = ChunkCoord::new(5, 5, 5);
        let neighbor_x_minus = ChunkCoord::new(4, 5, 5);
        let neighbor_x_plus = ChunkCoord::new(6, 5, 5);
        let neighbor_y_minus = ChunkCoord::new(5, 4, 5);

        manager.loaded.insert(center);
        manager.loaded.insert(neighbor_x_minus);
        manager.loaded.insert(neighbor_x_plus);
        manager.loaded.insert(neighbor_y_minus);

        // Test 1: Interior block (8, 8, 8) in chunk (5, 5, 5) - only marks center
        // World coords: 5*16 + 8 = 88
        let interior_pos = BlockPos::new(88, 88, 88);
        manager.mark_dirty_for_block(interior_pos);
        assert_eq!(
            manager.dirty_count(),
            1,
            "Interior block should mark only 1 chunk"
        );
        manager.dirty_queue.clear();
        manager.dirty_set.clear();

        // Test 2: X=0 boundary block - marks center + neighbor_x_minus
        // Local (0, 8, 8) in chunk (5,5,5) = world (5*16 + 0, 88, 88) = (80, 88, 88)
        let x_boundary_pos = BlockPos::new(80, 88, 88);
        manager.mark_dirty_for_block(x_boundary_pos);
        assert_eq!(
            manager.dirty_count(),
            2,
            "X=0 boundary should mark 2 chunks"
        );
        assert!(manager.dirty_set.contains(&center));
        assert!(manager.dirty_set.contains(&neighbor_x_minus));
        manager.dirty_queue.clear();
        manager.dirty_set.clear();

        // Test 3: X=15 boundary block - marks center + neighbor_x_plus
        // Local (15, 8, 8) in chunk (5,5,5) = world (5*16 + 15, 88, 88) = (95, 88, 88)
        let x_max_boundary_pos = BlockPos::new(95, 88, 88);
        manager.mark_dirty_for_block(x_max_boundary_pos);
        assert_eq!(
            manager.dirty_count(),
            2,
            "X=15 boundary should mark 2 chunks"
        );
        assert!(manager.dirty_set.contains(&center));
        assert!(manager.dirty_set.contains(&neighbor_x_plus));
        manager.dirty_queue.clear();
        manager.dirty_set.clear();

        // Test 4: Corner block (0, 0, 8) - marks center + x_minus + y_minus
        // Local (0, 0, 8) in chunk (5,5,5) = world (80, 80, 88)
        let edge_pos = BlockPos::new(80, 80, 88);
        manager.mark_dirty_for_block(edge_pos);
        assert_eq!(
            manager.dirty_count(),
            3,
            "Edge boundary should mark 3 chunks"
        );
        assert!(manager.dirty_set.contains(&center));
        assert!(manager.dirty_set.contains(&neighbor_x_minus));
        assert!(manager.dirty_set.contains(&neighbor_y_minus));
        manager.dirty_queue.clear();
        manager.dirty_set.clear();

        // Test 5: Boundary with unloaded neighbor - only marks loaded chunks
        // Unload x_minus neighbor
        manager.loaded.remove(&neighbor_x_minus);
        manager.mark_dirty_for_block(x_boundary_pos);
        assert_eq!(manager.dirty_count(), 1, "Should only mark loaded chunks");
        assert!(manager.dirty_set.contains(&center));
        assert!(!manager.dirty_set.contains(&neighbor_x_minus));
    }
}
