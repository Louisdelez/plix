//! Entity API: reading and modifying entity state

use crate::capabilities::{require, Capability};
use crate::errors::{err_invalid, err_not_found, err_perm, ModApiError};
use crate::events::DamageSource;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Transform component (position + rotation)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    /// World position
    pub position: Vec3,
    /// Rotation quaternion
    pub rotation: Quat,
}

impl Transform {
    /// Create a new transform at position with no rotation
    pub fn at_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
        }
    }

    /// Create a transform with position and rotation
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self { position, rotation }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
        }
    }
}

/// Opaque handle for safe entity reference
///
/// Uses a generation counter to detect stale handles (use-after-despawn).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityHandle {
    /// Entity index
    pub index: u32,
    /// Generation counter for validity checking
    pub generation: u32,
}

impl EntityHandle {
    /// Create a new entity handle
    pub fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }
}

/// Types of entities that can be spawned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// A projectile (bullet, arrow, etc.)
    Projectile,
    /// A dropped item
    DroppedItem,
    /// A visual effect
    Effect,
    /// Custom entity type defined by ID
    Custom(u32),
}

/// Trait for entity access
///
/// This trait is implemented by the game engine to provide actual entity access.
/// Mods call through this interface with capability checks enforced.
pub trait EntityApi {
    /// Get entity transform (position + rotation)
    ///
    /// # Capability Required
    /// - `entity.read` (ENTITY_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found (stale handle)
    fn get_entity_transform(&self, entity: EntityHandle) -> Result<Transform, ModApiError>;

    /// Get entity velocity
    ///
    /// # Capability Required
    /// - `entity.read` (ENTITY_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    /// - EMOD001: Entity has no velocity component
    fn get_entity_velocity(&self, entity: EntityHandle) -> Result<Vec3, ModApiError>;

    /// Get entity health
    ///
    /// # Capability Required
    /// - `entity.read` (ENTITY_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    /// - EMOD001: Entity has no health component
    fn get_entity_health(&self, entity: EntityHandle) -> Result<f32, ModApiError>;

    /// Get entity owner player ID
    ///
    /// # Capability Required
    /// - `entity.read` (ENTITY_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    fn get_entity_owner(&self, entity: EntityHandle) -> Result<Option<u64>, ModApiError>;

    /// Get entity team ID
    ///
    /// # Capability Required
    /// - `entity.read` (ENTITY_READ)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    fn get_entity_team(&self, entity: EntityHandle) -> Result<Option<u8>, ModApiError>;

    /// Apply damage to an entity
    ///
    /// # Capability Required
    /// - `entity.write` (ENTITY_WRITE)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    /// - EMOD001: Negative damage amount, or entity has no health
    fn apply_damage(
        &mut self,
        entity: EntityHandle,
        amount: f32,
        source: Option<DamageSource>,
    ) -> Result<(), ModApiError>;

    /// Apply physics impulse to an entity
    ///
    /// # Capability Required
    /// - `entity.write` (ENTITY_WRITE)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    /// - EMOD001: Entity has no physics component
    fn apply_impulse(&mut self, entity: EntityHandle, impulse: Vec3) -> Result<(), ModApiError>;

    /// Spawn a new entity
    ///
    /// # Capability Required
    /// - `entity.write` (ENTITY_WRITE)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD001: Invalid entity type
    /// - EMOD004: Transform position outside world bounds
    fn spawn_entity(
        &mut self,
        entity_type: EntityType,
        transform: Transform,
    ) -> Result<EntityHandle, ModApiError>;

    /// Remove an entity from the world
    ///
    /// # Capability Required
    /// - `entity.write` (ENTITY_WRITE)
    ///
    /// # Errors
    /// - EMOD002: Missing capability
    /// - EMOD003: Entity not found
    /// - EMOD002: Cannot despawn protected entities (players)
    fn despawn_entity(&mut self, entity: EntityHandle) -> Result<(), ModApiError>;

    /// Check if an entity handle is valid
    fn is_entity_valid(&self, entity: EntityHandle) -> bool;

    /// Check if an entity is protected (cannot be despawned by mods)
    fn is_entity_protected(&self, entity: EntityHandle) -> bool;
}

/// Wrapper that adds capability checks to an EntityApi implementation
pub struct CapabilityCheckedEntities<'a, E: EntityApi> {
    inner: &'a mut E,
    capabilities: Capability,
}

impl<'a, E: EntityApi> CapabilityCheckedEntities<'a, E> {
    /// Create a new capability-checked wrapper
    pub fn new(inner: &'a mut E, capabilities: Capability) -> Self {
        Self {
            inner,
            capabilities,
        }
    }

    /// Get entity transform with capability check
    pub fn get_entity_transform(&self, entity: EntityHandle) -> Result<Transform, ModApiError> {
        require(self.capabilities, Capability::ENTITY_READ)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.get_entity_transform(entity)
    }

    /// Get entity velocity with capability check
    pub fn get_entity_velocity(&self, entity: EntityHandle) -> Result<Vec3, ModApiError> {
        require(self.capabilities, Capability::ENTITY_READ)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.get_entity_velocity(entity)
    }

    /// Get entity health with capability check
    pub fn get_entity_health(&self, entity: EntityHandle) -> Result<f32, ModApiError> {
        require(self.capabilities, Capability::ENTITY_READ)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.get_entity_health(entity)
    }

    /// Get entity owner with capability check
    pub fn get_entity_owner(&self, entity: EntityHandle) -> Result<Option<u64>, ModApiError> {
        require(self.capabilities, Capability::ENTITY_READ)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.get_entity_owner(entity)
    }

    /// Get entity team with capability check
    pub fn get_entity_team(&self, entity: EntityHandle) -> Result<Option<u8>, ModApiError> {
        require(self.capabilities, Capability::ENTITY_READ)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.get_entity_team(entity)
    }

    /// Apply damage with capability check
    pub fn apply_damage(
        &mut self,
        entity: EntityHandle,
        amount: f32,
        source: Option<DamageSource>,
    ) -> Result<(), ModApiError> {
        require(self.capabilities, Capability::ENTITY_WRITE)?;

        if amount < 0.0 {
            return Err(err_invalid("Damage amount cannot be negative"));
        }

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.apply_damage(entity, amount, source)
    }

    /// Apply impulse with capability check
    pub fn apply_impulse(
        &mut self,
        entity: EntityHandle,
        impulse: Vec3,
    ) -> Result<(), ModApiError> {
        require(self.capabilities, Capability::ENTITY_WRITE)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        self.inner.apply_impulse(entity, impulse)
    }

    /// Spawn entity with capability check
    pub fn spawn_entity(
        &mut self,
        entity_type: EntityType,
        transform: Transform,
    ) -> Result<EntityHandle, ModApiError> {
        require(self.capabilities, Capability::ENTITY_WRITE)?;
        self.inner.spawn_entity(entity_type, transform)
    }

    /// Despawn entity with capability check
    pub fn despawn_entity(&mut self, entity: EntityHandle) -> Result<(), ModApiError> {
        require(self.capabilities, Capability::ENTITY_WRITE)?;

        if !self.inner.is_entity_valid(entity) {
            return Err(err_not_found("Entity"));
        }

        if self.inner.is_entity_protected(entity) {
            return Err(err_perm("Cannot despawn protected entities (players)"));
        }

        self.inner.despawn_entity(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock EntityApi for testing
    struct MockEntities {
        entities: HashMap<u32, MockEntity>,
        next_index: u32,
    }

    struct MockEntity {
        generation: u32,
        transform: Transform,
        velocity: Option<Vec3>,
        health: Option<f32>,
        owner: Option<u64>,
        team: Option<u8>,
        protected: bool,
    }

    impl Default for MockEntities {
        fn default() -> Self {
            Self {
                entities: HashMap::new(),
                next_index: 0,
            }
        }
    }

    impl MockEntities {
        fn spawn_mock(&mut self, protected: bool) -> EntityHandle {
            let index = self.next_index;
            self.next_index += 1;
            let generation = 1;

            self.entities.insert(
                index,
                MockEntity {
                    generation,
                    transform: Transform::default(),
                    velocity: Some(Vec3::ZERO),
                    health: Some(100.0),
                    owner: None,
                    team: None,
                    protected,
                },
            );

            EntityHandle::new(index, generation)
        }
    }

    impl EntityApi for MockEntities {
        fn get_entity_transform(&self, entity: EntityHandle) -> Result<Transform, ModApiError> {
            self.entities
                .get(&entity.index)
                .filter(|e| e.generation == entity.generation)
                .map(|e| e.transform)
                .ok_or_else(|| err_not_found("Entity"))
        }

        fn get_entity_velocity(&self, entity: EntityHandle) -> Result<Vec3, ModApiError> {
            self.entities
                .get(&entity.index)
                .filter(|e| e.generation == entity.generation)
                .and_then(|e| e.velocity)
                .ok_or_else(|| err_not_found("Entity"))
        }

        fn get_entity_health(&self, entity: EntityHandle) -> Result<f32, ModApiError> {
            self.entities
                .get(&entity.index)
                .filter(|e| e.generation == entity.generation)
                .and_then(|e| e.health)
                .ok_or_else(|| err_not_found("Entity"))
        }

        fn get_entity_owner(&self, entity: EntityHandle) -> Result<Option<u64>, ModApiError> {
            self.entities
                .get(&entity.index)
                .filter(|e| e.generation == entity.generation)
                .map(|e| e.owner)
                .ok_or_else(|| err_not_found("Entity"))
        }

        fn get_entity_team(&self, entity: EntityHandle) -> Result<Option<u8>, ModApiError> {
            self.entities
                .get(&entity.index)
                .filter(|e| e.generation == entity.generation)
                .map(|e| e.team)
                .ok_or_else(|| err_not_found("Entity"))
        }

        fn apply_damage(
            &mut self,
            entity: EntityHandle,
            amount: f32,
            _source: Option<DamageSource>,
        ) -> Result<(), ModApiError> {
            if let Some(e) = self.entities.get_mut(&entity.index) {
                if e.generation == entity.generation {
                    if let Some(ref mut health) = e.health {
                        *health = (*health - amount).max(0.0);
                        return Ok(());
                    }
                }
            }
            Err(err_not_found("Entity"))
        }

        fn apply_impulse(
            &mut self,
            entity: EntityHandle,
            _impulse: Vec3,
        ) -> Result<(), ModApiError> {
            if self.is_entity_valid(entity) {
                Ok(())
            } else {
                Err(err_not_found("Entity"))
            }
        }

        fn spawn_entity(
            &mut self,
            _entity_type: EntityType,
            transform: Transform,
        ) -> Result<EntityHandle, ModApiError> {
            let index = self.next_index;
            self.next_index += 1;
            let generation = 1;

            self.entities.insert(
                index,
                MockEntity {
                    generation,
                    transform,
                    velocity: Some(Vec3::ZERO),
                    health: Some(100.0),
                    owner: None,
                    team: None,
                    protected: false,
                },
            );

            Ok(EntityHandle::new(index, generation))
        }

        fn despawn_entity(&mut self, entity: EntityHandle) -> Result<(), ModApiError> {
            if let Some(e) = self.entities.get(&entity.index) {
                if e.generation == entity.generation {
                    self.entities.remove(&entity.index);
                    return Ok(());
                }
            }
            Err(err_not_found("Entity"))
        }

        fn is_entity_valid(&self, entity: EntityHandle) -> bool {
            self.entities
                .get(&entity.index)
                .map(|e| e.generation == entity.generation)
                .unwrap_or(false)
        }

        fn is_entity_protected(&self, entity: EntityHandle) -> bool {
            self.entities
                .get(&entity.index)
                .map(|e| e.protected)
                .unwrap_or(false)
        }
    }

    #[test]
    fn test_entity_handle_equality() {
        let h1 = EntityHandle::new(1, 1);
        let h2 = EntityHandle::new(1, 1);
        let h3 = EntityHandle::new(1, 2);

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_get_transform_with_capability() {
        let mut entities = MockEntities::default();
        let handle = entities.spawn_mock(false);

        let caps = Capability::ENTITY_READ;
        let checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.get_entity_transform(handle);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_transform_without_capability() {
        let mut entities = MockEntities::default();
        let handle = entities.spawn_mock(false);

        let caps = Capability::empty();
        let checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.get_entity_transform(handle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_stale_handle() {
        let mut entities = MockEntities::default();
        let handle = EntityHandle::new(999, 1); // Non-existent

        let caps = Capability::ENTITY_READ;
        let checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.get_entity_transform(handle);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, crate::errors::ErrorCode::NotFound);
    }

    #[test]
    fn test_apply_damage_negative() {
        let mut entities = MockEntities::default();
        let handle = entities.spawn_mock(false);

        let caps = Capability::ENTITY_WRITE;
        let mut checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.apply_damage(handle, -10.0, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::InvalidArgument
        );
    }

    #[test]
    fn test_despawn_protected() {
        let mut entities = MockEntities::default();
        let handle = entities.spawn_mock(true); // Protected

        let caps = Capability::ENTITY_WRITE;
        let mut checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.despawn_entity(handle);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            crate::errors::ErrorCode::PermissionDenied
        );
    }

    #[test]
    fn test_spawn_and_despawn() {
        let mut entities = MockEntities::default();

        let caps = Capability::ENTITY_WRITE;
        let mut checked = CapabilityCheckedEntities::new(&mut entities, caps);

        let result = checked.spawn_entity(EntityType::Projectile, Transform::default());
        assert!(result.is_ok());
        let handle = result.unwrap();

        let result = checked.despawn_entity(handle);
        assert!(result.is_ok());

        // Handle should now be invalid
        assert!(!entities.is_entity_valid(handle));
    }
}
