//! Entity API - read and modify entity state
//!
//! Provides functions to query and modify game entities (players, NPCs, etc).
//! Requires ENTITY_READ capability for reading, ENTITY_WRITE for modification.
//!
//! # Example
//!
//! ```rust,ignore
//! use plix_mod_sdk::entities::{get_transform, apply_damage, apply_impulse, EntityHandle};
//! use glam::Vec3;
//!
//! // Get entity position
//! let transform = get_transform(entity)?;
//! info!("Entity at {:?}", transform.position);
//!
//! // Apply damage
//! apply_damage(entity, 10.0, Some("mod"))?;
//!
//! // Push entity upward
//! apply_impulse(entity, Vec3::new(0.0, 10.0, 0.0))?;
//! ```

use crate::abi::{self, HostCallOp};
use crate::codec::{
    self, decode, encode, ApplyDamageRequest, ApplyImpulseRequest, GetTransformRequest,
    TransformResponse,
};
use crate::error::Result;
use alloc::string::String;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Handle to an entity in the game world
///
/// Entity handles use a generation-based system to detect stale references.
/// If an entity is despawned and the slot is reused, the generation will differ
/// and operations will fail with NotFound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityHandle {
    /// Index in the entity array
    pub index: u32,
    /// Generation counter (increments when slot is reused)
    pub generation: u32,
}

impl EntityHandle {
    /// Create a new entity handle
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Create a handle from a raw u64 (packed index + generation)
    pub fn from_u64(value: u64) -> Self {
        Self {
            index: value as u32,
            generation: (value >> 32) as u32,
        }
    }

    /// Convert to a raw u64 (packed index + generation)
    pub fn to_u64(&self) -> u64 {
        (self.index as u64) | ((self.generation as u64) << 32)
    }
}

/// Entity transform (position, rotation, scale)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    /// World position
    pub position: Vec3,
    /// Rotation quaternion
    pub rotation: Quat,
    /// Scale factor
    pub scale: Vec3,
}

impl Transform {
    /// Create a new transform at the given position
    pub fn at_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Get the forward direction vector
    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    /// Get the right direction vector
    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    /// Get the up direction vector
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

/// Get the transform of an entity
///
/// Requires ENTITY_READ capability.
///
/// # Arguments
///
/// * `entity` - Entity handle
///
/// # Returns
///
/// The entity's current transform (position, rotation, scale)
///
/// # Errors
///
/// - `NotFound` if the entity doesn't exist or handle is stale
/// - `PermissionDenied` if ENTITY_READ capability not granted
///
/// # Example
///
/// ```rust,ignore
/// let transform = get_transform(player_entity)?;
/// info!("Player is at {:?}", transform.position);
/// ```
pub fn get_transform(entity: EntityHandle) -> Result<Transform> {
    let request = GetTransformRequest {
        entity_index: entity.index,
        entity_generation: entity.generation,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::EntityGetTransform, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    let result = response.into_result()?;
    let transform_response: TransformResponse = decode(&result)?;

    Ok(Transform {
        position: Vec3::new(
            transform_response.pos_x,
            transform_response.pos_y,
            transform_response.pos_z,
        ),
        rotation: Quat::from_xyzw(
            transform_response.rot_x,
            transform_response.rot_y,
            transform_response.rot_z,
            transform_response.rot_w,
        ),
        scale: Vec3::new(
            transform_response.scale_x,
            transform_response.scale_y,
            transform_response.scale_z,
        ),
    })
}

/// Apply damage to an entity
///
/// Requires ENTITY_WRITE capability.
///
/// # Arguments
///
/// * `entity` - Entity handle
/// * `amount` - Damage amount (positive)
/// * `source` - Optional damage source type (e.g., "melee", "fall", "mod")
///
/// # Errors
///
/// - `NotFound` if the entity doesn't exist or handle is stale
/// - `PermissionDenied` if ENTITY_WRITE capability not granted
/// - `InvalidArgument` if amount is negative
///
/// # Example
///
/// ```rust,ignore
/// // Apply 10 damage from a mod
/// apply_damage(enemy, 10.0, Some("mod"))?;
///
/// // Apply environmental damage
/// apply_damage(player, 5.0, Some("environment"))?;
/// ```
pub fn apply_damage(entity: EntityHandle, amount: f32, source: Option<&str>) -> Result<()> {
    if amount < 0.0 {
        return Err(crate::error::err_invalid("Damage amount cannot be negative"));
    }

    let request = ApplyDamageRequest {
        entity_index: entity.index,
        entity_generation: entity.generation,
        amount,
        source_type: source.map(String::from),
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::EntityApplyDamage, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

/// Apply an impulse to an entity
///
/// Requires ENTITY_WRITE capability.
///
/// # Arguments
///
/// * `entity` - Entity handle
/// * `impulse` - Impulse vector (velocity change)
///
/// # Errors
///
/// - `NotFound` if the entity doesn't exist or handle is stale
/// - `PermissionDenied` if ENTITY_WRITE capability not granted
///
/// # Example
///
/// ```rust,ignore
/// // Launch entity upward
/// apply_impulse(entity, Vec3::new(0.0, 20.0, 0.0))?;
///
/// // Push entity backward
/// let backward = -transform.forward() * 10.0;
/// apply_impulse(entity, backward)?;
/// ```
pub fn apply_impulse(entity: EntityHandle, impulse: Vec3) -> Result<()> {
    let request = ApplyImpulseRequest {
        entity_index: entity.index,
        entity_generation: entity.generation,
        impulse_x: impulse.x,
        impulse_y: impulse.y,
        impulse_z: impulse.z,
    };
    let request_bytes = encode(&request)?;

    let response_bytes = abi::host_call(HostCallOp::EntityApplyImpulse, &request_bytes)
        .map_err(|code| crate::error::ModError::from_abi(code as u16, None))?;

    let response: codec::AbiResponse = decode(&response_bytes)?;
    response.into_result()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_handle_u64_roundtrip() {
        let handle = EntityHandle::new(42, 7);
        let packed = handle.to_u64();
        let unpacked = EntityHandle::from_u64(packed);

        assert_eq!(unpacked.index, 42);
        assert_eq!(unpacked.generation, 7);
    }

    #[test]
    fn test_transform_default() {
        let t = Transform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_transform_directions() {
        let t = Transform::default();
        assert_eq!(t.forward(), Vec3::NEG_Z);
        assert_eq!(t.right(), Vec3::X);
        assert_eq!(t.up(), Vec3::Y);
    }
}
