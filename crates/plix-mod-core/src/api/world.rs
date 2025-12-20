//! World API: block access and raycasting

use crate::capabilities::{require, Capability};
use crate::errors::{err_bounds, err_invalid, err_world_not_ready, ModApiError};
use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

/// Maximum raycast distance (256 blocks)
pub const MAX_RAYCAST_DISTANCE: f32 = 256.0;

/// Maximum AABB query results (128)
pub const MAX_AABB_RESULTS: u32 = 128;

/// Block face hit by raycast
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockFace {
    /// +X face
    PosX,
    /// -X face
    NegX,
    /// +Y face (top)
    PosY,
    /// -Y face (bottom)
    NegY,
    /// +Z face
    PosZ,
    /// -Z face
    NegZ,
}

/// Result of a raycast operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastHit {
    /// Block position that was hit
    pub pos: IVec3,
    /// Block type ID at that position
    pub block_id: u16,
    /// Which face of the block was hit
    pub face: BlockFace,
    /// Distance from ray origin
    pub distance: f32,
}

/// Result of an AABB block query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockQuery {
    /// Block position
    pub pos: IVec3,
    /// Block type ID
    pub block_id: u16,
}

/// Trait for world state access
///
/// This trait is implemented by the game engine to provide actual world access.
/// Mods call through this interface with capability checks enforced.
pub trait WorldApi {
    /// Get the block type at a position
    ///
    /// # Capability Required
    /// - `world.read` (WORLD_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD004: Position outside world bounds
    /// - EMOD006: Chunk not loaded
    fn get_block(&self, pos: IVec3) -> Result<u16, ModApiError>;

    /// Set the block type at a position
    ///
    /// # Capability Required
    /// - `world.write` (WORLD_WRITE)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD001: Invalid block_id
    /// - EMOD004: Position outside world bounds
    /// - EMOD006: Chunk not loaded
    fn set_block(&mut self, pos: IVec3, block_id: u16) -> Result<(), ModApiError>;

    /// Cast a ray and find the first block hit
    ///
    /// # Capability Required
    /// - `world.read` (WORLD_READ)
    ///
    /// # Parameters
    /// - `origin`: Ray start position
    /// - `dir`: Ray direction (will be normalized)
    /// - `max_dist`: Maximum distance (clamped to 256 blocks)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD001: Invalid direction (zero vector)
    fn raycast(
        &self,
        origin: Vec3,
        dir: Vec3,
        max_dist: f32,
    ) -> Result<Option<RaycastHit>, ModApiError>;

    /// Query non-air blocks in an axis-aligned bounding box
    ///
    /// # Capability Required
    /// - `world.read` (WORLD_READ)
    ///
    /// # Parameters
    /// - `min`: Minimum corner
    /// - `max`: Maximum corner
    /// - `limit`: Maximum results (clamped to 128)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD001: min > max on any axis
    /// - EMOD006: Any chunk in range not loaded
    fn query_aabb(
        &self,
        min: IVec3,
        max: IVec3,
        limit: u32,
    ) -> Result<Vec<BlockQuery>, ModApiError>;

    /// Check if a chunk at the given position is loaded
    fn is_chunk_loaded(&self, pos: IVec3) -> bool;

    /// Check if a position is within world bounds
    fn is_in_bounds(&self, pos: IVec3) -> bool;
}

/// Helper to validate and clamp raycast distance
pub fn clamp_raycast_distance(max_dist: f32) -> f32 {
    max_dist.clamp(0.0, MAX_RAYCAST_DISTANCE)
}

/// Helper to validate and clamp AABB result limit
pub fn clamp_aabb_limit(limit: u32) -> u32 {
    limit.min(MAX_AABB_RESULTS)
}

/// Helper to validate direction vector
pub fn validate_direction(dir: Vec3) -> Result<Vec3, ModApiError> {
    let length_sq = dir.length_squared();
    if length_sq < f32::EPSILON {
        return Err(err_invalid("Direction vector cannot be zero"));
    }
    Ok(dir.normalize())
}

/// Helper to validate AABB bounds
pub fn validate_aabb(min: IVec3, max: IVec3) -> Result<(), ModApiError> {
    if min.x > max.x || min.y > max.y || min.z > max.z {
        return Err(err_invalid("AABB min must be <= max on all axes"));
    }
    Ok(())
}

/// Wrapper that adds capability checks to a WorldApi implementation
pub struct CapabilityCheckedWorld<'a, W: WorldApi> {
    inner: &'a mut W,
    capabilities: Capability,
}

impl<'a, W: WorldApi> CapabilityCheckedWorld<'a, W> {
    /// Create a new capability-checked wrapper
    pub fn new(inner: &'a mut W, capabilities: Capability) -> Self {
        Self {
            inner,
            capabilities,
        }
    }

    /// Get block with capability check
    pub fn get_block(&self, pos: IVec3) -> Result<u16, ModApiError> {
        require(self.capabilities, Capability::WORLD_READ)?;

        if !self.inner.is_in_bounds(pos) {
            return Err(err_bounds(format!(
                "Position {:?} is outside world bounds",
                pos
            )));
        }

        if !self.inner.is_chunk_loaded(pos) {
            return Err(err_world_not_ready(format!(
                "Chunk at {:?} is not loaded",
                pos
            )));
        }

        self.inner.get_block(pos)
    }

    /// Set block with capability check
    pub fn set_block(&mut self, pos: IVec3, block_id: u16) -> Result<(), ModApiError> {
        require(self.capabilities, Capability::WORLD_WRITE)?;

        if !self.inner.is_in_bounds(pos) {
            return Err(err_bounds(format!(
                "Position {:?} is outside world bounds",
                pos
            )));
        }

        if !self.inner.is_chunk_loaded(pos) {
            return Err(err_world_not_ready(format!(
                "Chunk at {:?} is not loaded",
                pos
            )));
        }

        self.inner.set_block(pos, block_id)
    }

    /// Raycast with capability check and distance clamp
    pub fn raycast(
        &self,
        origin: Vec3,
        dir: Vec3,
        max_dist: f32,
    ) -> Result<Option<RaycastHit>, ModApiError> {
        require(self.capabilities, Capability::WORLD_READ)?;

        let dir = validate_direction(dir)?;
        let max_dist = clamp_raycast_distance(max_dist);

        self.inner.raycast(origin, dir, max_dist)
    }

    /// Query AABB with capability check and limit clamp
    pub fn query_aabb(
        &self,
        min: IVec3,
        max: IVec3,
        limit: u32,
    ) -> Result<Vec<BlockQuery>, ModApiError> {
        require(self.capabilities, Capability::WORLD_READ)?;
        validate_aabb(min, max)?;

        let limit = clamp_aabb_limit(limit);
        self.inner.query_aabb(min, max, limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock WorldApi for testing
    struct MockWorld {
        blocks: std::collections::HashMap<IVec3, u16>,
        loaded_chunks: std::collections::HashSet<IVec3>,
        world_min: IVec3,
        world_max: IVec3,
    }

    impl Default for MockWorld {
        fn default() -> Self {
            Self {
                blocks: std::collections::HashMap::new(),
                loaded_chunks: std::collections::HashSet::new(),
                world_min: IVec3::new(-1000, -64, -1000),
                world_max: IVec3::new(1000, 320, 1000),
            }
        }
    }

    impl MockWorld {
        fn with_block(mut self, pos: IVec3, block_id: u16) -> Self {
            self.blocks.insert(pos, block_id);
            self.loaded_chunks.insert(pos / 16);
            self
        }

        fn with_loaded_chunk(mut self, chunk_pos: IVec3) -> Self {
            self.loaded_chunks.insert(chunk_pos);
            self
        }
    }

    impl WorldApi for MockWorld {
        fn get_block(&self, pos: IVec3) -> Result<u16, ModApiError> {
            Ok(*self.blocks.get(&pos).unwrap_or(&0))
        }

        fn set_block(&mut self, pos: IVec3, block_id: u16) -> Result<(), ModApiError> {
            self.blocks.insert(pos, block_id);
            Ok(())
        }

        fn raycast(
            &self,
            _origin: Vec3,
            _dir: Vec3,
            _max_dist: f32,
        ) -> Result<Option<RaycastHit>, ModApiError> {
            // Simple mock - just return None
            Ok(None)
        }

        fn query_aabb(
            &self,
            min: IVec3,
            max: IVec3,
            limit: u32,
        ) -> Result<Vec<BlockQuery>, ModApiError> {
            let mut results = Vec::new();
            for x in min.x..=max.x {
                for y in min.y..=max.y {
                    for z in min.z..=max.z {
                        if results.len() >= limit as usize {
                            return Ok(results);
                        }
                        let pos = IVec3::new(x, y, z);
                        if let Some(&block_id) = self.blocks.get(&pos) {
                            if block_id != 0 {
                                results.push(BlockQuery { pos, block_id });
                            }
                        }
                    }
                }
            }
            Ok(results)
        }

        fn is_chunk_loaded(&self, pos: IVec3) -> bool {
            self.loaded_chunks.contains(&(pos / 16))
        }

        fn is_in_bounds(&self, pos: IVec3) -> bool {
            pos.x >= self.world_min.x
                && pos.x <= self.world_max.x
                && pos.y >= self.world_min.y
                && pos.y <= self.world_max.y
                && pos.z >= self.world_min.z
                && pos.z <= self.world_max.z
        }
    }

    #[test]
    fn test_get_block_with_capability() {
        let mut world = MockWorld::default()
            .with_block(IVec3::new(0, 0, 0), 1)
            .with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::WORLD_READ;
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.get_block(IVec3::new(0, 0, 0));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1);
    }

    #[test]
    fn test_get_block_without_capability() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::empty();
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.get_block(IVec3::new(0, 0, 0));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_set_block_without_write_capability() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        // Only read capability
        let caps = Capability::WORLD_READ;
        let mut checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.set_block(IVec3::new(0, 0, 0), 1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_set_block_with_capability() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::WORLD_WRITE;
        let mut checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.set_block(IVec3::new(0, 0, 0), 5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chunk_not_loaded() {
        let mut world = MockWorld::default(); // No chunks loaded

        let caps = Capability::WORLD_READ;
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.get_block(IVec3::new(0, 0, 0));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::WorldNotReady
        );
    }

    #[test]
    fn test_out_of_bounds() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::WORLD_READ;
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.get_block(IVec3::new(10000, 0, 0));
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::OutOfBounds
        );
    }

    #[test]
    fn test_raycast_distance_clamp() {
        assert_eq!(clamp_raycast_distance(100.0), 100.0);
        assert_eq!(clamp_raycast_distance(300.0), 256.0);
        assert_eq!(clamp_raycast_distance(-10.0), 0.0);
    }

    #[test]
    fn test_aabb_limit_clamp() {
        assert_eq!(clamp_aabb_limit(50), 50);
        assert_eq!(clamp_aabb_limit(200), 128);
        assert_eq!(clamp_aabb_limit(128), 128);
    }

    #[test]
    fn test_validate_direction() {
        let result = validate_direction(Vec3::new(1.0, 0.0, 0.0));
        assert!(result.is_ok());

        let result = validate_direction(Vec3::ZERO);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn test_validate_aabb() {
        // Valid AABB
        assert!(validate_aabb(IVec3::new(0, 0, 0), IVec3::new(10, 10, 10)).is_ok());

        // Invalid - min > max
        assert!(validate_aabb(IVec3::new(10, 0, 0), IVec3::new(0, 10, 10)).is_err());
    }

    #[test]
    fn test_raycast_zero_direction() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::WORLD_READ;
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.raycast(Vec3::ZERO, Vec3::ZERO, 100.0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn test_query_aabb_invalid_bounds() {
        let mut world = MockWorld::default().with_loaded_chunk(IVec3::ZERO);

        let caps = Capability::WORLD_READ;
        let checked = CapabilityCheckedWorld::new(&mut world, caps);

        let result = checked.query_aabb(IVec3::new(10, 10, 10), IVec3::new(0, 0, 0), 100);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::InvalidArgument
        );
    }
}
