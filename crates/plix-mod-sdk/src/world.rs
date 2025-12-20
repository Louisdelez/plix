//! World API - read and modify block data
//!
//! Provides functions to query and modify the voxel world.
//! Requires WORLD_READ capability for reading, WORLD_WRITE for modification.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::world::{get_block, set_block, raycast};
//! use glam::IVec3;
//!
//! // Read a block
//! let block_id = get_block(IVec3::new(0, 64, 0))?;
//!
//! // Set a block
//! set_block(IVec3::new(0, 65, 0), 1)?;
//!
//! // Raycast to find first solid block
//! if let Some(hit) = raycast(origin, direction, 100.0)? {
//!     info!("Hit block {} at {:?}", hit.block_id, hit.pos);
//! }
//! ```
//!
//! # Constraints
//!
//! - Raycast max distance: 256 blocks
//! - AABB query max results: 128 blocks
//! - Operations on unloaded chunks return WorldNotReady error

use crate::abi::{self, HostCallOp};
use crate::codec::{
    self, decode, encode, GetBlockRequest, GetBlockResponse, QueryAabbRequest, QueryAabbResponse,
    RaycastHitResponse, RaycastRequest, SetBlockRequest,
};
use crate::error::{err_bounds, Result};
use alloc::vec::Vec;
use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

/// Maximum raycast distance in blocks
pub const MAX_RAYCAST_DISTANCE: f32 = 256.0;

/// Maximum AABB query results
pub const MAX_AABB_RESULTS: u32 = 128;

/// Block face (for raycast hits)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum BlockFace {
    /// Top face (+Y)
    Top = 0,
    /// Bottom face (-Y)
    Bottom = 1,
    /// North face (-Z)
    North = 2,
    /// South face (+Z)
    South = 3,
    /// East face (+X)
    East = 4,
    /// West face (-X)
    West = 5,
}

impl BlockFace {
    /// Parse from u8
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(BlockFace::Top),
            1 => Some(BlockFace::Bottom),
            2 => Some(BlockFace::North),
            3 => Some(BlockFace::South),
            4 => Some(BlockFace::East),
            5 => Some(BlockFace::West),
            _ => None,
        }
    }

    /// Get the normal vector for this face
    pub fn normal(&self) -> Vec3 {
        match self {
            BlockFace::Top => Vec3::Y,
            BlockFace::Bottom => Vec3::NEG_Y,
            BlockFace::North => Vec3::NEG_Z,
            BlockFace::South => Vec3::Z,
            BlockFace::East => Vec3::X,
            BlockFace::West => Vec3::NEG_X,
        }
    }
}

/// Result of a raycast operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastHit {
    /// Block position that was hit
    pub pos: IVec3,
    /// Block type ID
    pub block_id: u16,
    /// Face that was hit
    pub face: BlockFace,
    /// Distance from origin to hit point
    pub distance: f32,
}

/// Result of an AABB query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockQuery {
    /// Block position
    pub pos: IVec3,
    /// Block type ID
    pub block_id: u16,
}

/// Get the block ID at a position
///
/// Requires WORLD_READ capability.
///
/// # Arguments
///
/// * `pos` - Block position
///
/// # Returns
///
/// The block type ID (0 = air)
///
/// # Errors
///
/// - `WorldNotReady` if the chunk is not loaded
/// - `PermissionDenied` if WORLD_READ capability not granted
///
/// # Example
///
/// ```rust,ignore
/// let block_id = get_block(IVec3::new(0, 64, 0))?;
/// if block_id != 0 {
///     info!("Found solid block: {}", block_id);
/// }
/// ```
pub fn get_block(pos: IVec3) -> Result<u16> {
    let request = GetBlockRequest {
        x: pos.x,
        y: pos.y,
        z: pos.z,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::WorldGetBlock, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let block_response: GetBlockResponse = decode(&result)?;

    Ok(block_response.block_id)
}

/// Set the block ID at a position
///
/// Requires WORLD_WRITE capability.
///
/// # Arguments
///
/// * `pos` - Block position
/// * `block_id` - New block type ID (0 = air)
///
/// # Errors
///
/// - `WorldNotReady` if the chunk is not loaded
/// - `PermissionDenied` if WORLD_WRITE capability not granted
/// - `OutOfBounds` if position is outside world bounds
///
/// # Example
///
/// ```rust,ignore
/// // Place a stone block
/// set_block(IVec3::new(0, 65, 0), 1)?;
///
/// // Remove a block (set to air)
/// set_block(IVec3::new(0, 65, 0), 0)?;
/// ```
pub fn set_block(pos: IVec3, block_id: u16) -> Result<()> {
    let request = SetBlockRequest {
        x: pos.x,
        y: pos.y,
        z: pos.z,
        block_id,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::WorldSetBlock, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

/// Cast a ray and find the first solid block hit
///
/// Requires WORLD_READ capability.
///
/// # Arguments
///
/// * `origin` - Ray origin position
/// * `direction` - Ray direction (will be normalized)
/// * `max_distance` - Maximum distance to check (clamped to 256)
///
/// # Returns
///
/// `Some(RaycastHit)` if a block was hit, `None` if ray didn't hit anything
///
/// # Errors
///
/// - `WorldNotReady` if any chunk along the ray is not loaded
/// - `PermissionDenied` if WORLD_READ capability not granted
/// - `OutOfBounds` if max_distance <= 0
///
/// # Example
///
/// ```rust,ignore
/// let hit = raycast(
///     Vec3::new(0.0, 64.0, 0.0),
///     Vec3::new(1.0, 0.0, 0.0),
///     100.0
/// )?;
///
/// if let Some(hit) = hit {
///     info!("Hit {} at {:?}, face {:?}", hit.block_id, hit.pos, hit.face);
/// }
/// ```
pub fn raycast(origin: Vec3, direction: Vec3, max_distance: f32) -> Result<Option<RaycastHit>> {
    if max_distance <= 0.0 {
        return Err(err_bounds("max_distance must be positive"));
    }

    let clamped_distance = max_distance.min(MAX_RAYCAST_DISTANCE);
    let dir = direction.normalize_or_zero();

    let request = RaycastRequest {
        origin_x: origin.x,
        origin_y: origin.y,
        origin_z: origin.z,
        dir_x: dir.x,
        dir_y: dir.y,
        dir_z: dir.z,
        max_distance: clamped_distance,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::WorldRaycast, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let hit_response: RaycastHitResponse = decode(&result)?;

    if !hit_response.hit {
        return Ok(None);
    }

    let face = BlockFace::from_u8(hit_response.face).unwrap_or(BlockFace::Top);

    Ok(Some(RaycastHit {
        pos: IVec3::new(hit_response.pos_x, hit_response.pos_y, hit_response.pos_z),
        block_id: hit_response.block_id,
        face,
        distance: hit_response.distance,
    }))
}

/// Query all blocks within an axis-aligned bounding box
///
/// Requires WORLD_READ capability.
///
/// # Arguments
///
/// * `min` - Minimum corner of the AABB
/// * `max` - Maximum corner of the AABB
///
/// # Returns
///
/// A vector of `BlockQuery` for each non-air block in the region.
/// Results are limited to 128 blocks.
///
/// # Errors
///
/// - `WorldNotReady` if any chunk in the region is not loaded
/// - `PermissionDenied` if WORLD_READ capability not granted
/// - `OutOfBounds` if min > max
///
/// # Example
///
/// ```rust,ignore
/// let blocks = query_aabb(
///     IVec3::new(0, 60, 0),
///     IVec3::new(10, 70, 10)
/// )?;
///
/// for block in blocks {
///     info!("Block {} at {:?}", block.block_id, block.pos);
/// }
/// ```
pub fn query_aabb(min: IVec3, max: IVec3) -> Result<Vec<BlockQuery>> {
    if min.x > max.x || min.y > max.y || min.z > max.z {
        return Err(err_bounds("min must be <= max for all axes"));
    }

    let request = QueryAabbRequest {
        min_x: min.x,
        min_y: min.y,
        min_z: min.z,
        max_x: max.x,
        max_y: max.y,
        max_z: max.z,
        limit: MAX_AABB_RESULTS,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::WorldQueryAabb, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let aabb_response: QueryAabbResponse = decode(&result)?;

    let blocks = aabb_response
        .blocks
        .into_iter()
        .map(|b| BlockQuery {
            pos: IVec3::new(b.x, b.y, b.z),
            block_id: b.block_id,
        })
        .collect();

    Ok(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_face_from_u8() {
        assert_eq!(BlockFace::from_u8(0), Some(BlockFace::Top));
        assert_eq!(BlockFace::from_u8(1), Some(BlockFace::Bottom));
        assert_eq!(BlockFace::from_u8(5), Some(BlockFace::West));
        assert_eq!(BlockFace::from_u8(99), None);
    }

    #[test]
    fn test_block_face_normal() {
        assert_eq!(BlockFace::Top.normal(), Vec3::Y);
        assert_eq!(BlockFace::Bottom.normal(), Vec3::NEG_Y);
        assert_eq!(BlockFace::East.normal(), Vec3::X);
    }
}
