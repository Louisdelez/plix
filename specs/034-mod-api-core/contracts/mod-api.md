# Mod API Contract

**Feature**: 034-mod-api-core
**API Version**: 1
**Date**: 2025-12-18

## Overview

This document defines the stable API contract for plix mods. All functions return `Result<T, ModApiError>`.

## Host API Trait

The `ModHost` trait defines the interface between the engine and mod runtime.

```rust
pub trait ModHost {
    // === Version Info ===
    fn get_api_version(&self) -> u32;
    fn get_engine_version(&self) -> String;

    // === Event System ===
    fn subscribe(&mut self, event_type: EventType) -> Result<(), ModApiError>;
    fn unsubscribe(&mut self, event_type: EventType) -> Result<(), ModApiError>;
    fn cancel_event(&mut self) -> Result<(), ModApiError>;

    // === World API ===
    fn get_block(&self, pos: IVec3) -> Result<BlockId, ModApiError>;
    fn set_block(&mut self, pos: IVec3, block_id: BlockId) -> Result<(), ModApiError>;
    fn raycast(&self, origin: Vec3, dir: Vec3, max_dist: f32) -> Result<Option<RaycastHit>, ModApiError>;
    fn query_aabb(&self, min: IVec3, max: IVec3, limit: u32) -> Result<Vec<BlockQuery>, ModApiError>;

    // === Entity API ===
    fn get_entity_transform(&self, entity: EntityHandle) -> Result<Transform, ModApiError>;
    fn get_entity_velocity(&self, entity: EntityHandle) -> Result<Vec3, ModApiError>;
    fn get_entity_health(&self, entity: EntityHandle) -> Result<f32, ModApiError>;
    fn get_entity_owner(&self, entity: EntityHandle) -> Result<Option<u64>, ModApiError>;
    fn get_entity_team(&self, entity: EntityHandle) -> Result<Option<u8>, ModApiError>;
    fn apply_damage(&mut self, entity: EntityHandle, amount: f32, source: Option<DamageSource>) -> Result<(), ModApiError>;
    fn apply_impulse(&mut self, entity: EntityHandle, impulse: Vec3) -> Result<(), ModApiError>;
    fn spawn_entity(&mut self, entity_type: EntityType, transform: Transform) -> Result<EntityHandle, ModApiError>;
    fn despawn_entity(&mut self, entity: EntityHandle) -> Result<(), ModApiError>;

    // === Network API ===
    fn send_message(&mut self, channel: &str, target: MessageTarget, payload: &[u8]) -> Result<(), ModApiError>;

    // === Timer API ===
    fn set_timeout(&mut self, delay_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError>;
    fn set_interval(&mut self, interval_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError>;
    fn clear_timer(&mut self, handle: TimerHandle) -> Result<(), ModApiError>;
}
```

---

## API Functions

### Version Info

#### `get_api_version() -> u32`
Returns the engine's current API version.

- **Capability Required**: None
- **Returns**: API version integer (MVP = 1)
- **Errors**: None

#### `get_engine_version() -> String`
Returns the engine's SemVer version string.

- **Capability Required**: None
- **Returns**: Version string (e.g., "0.1.0")
- **Errors**: None

---

### Event System

#### `subscribe(event_type: EventType) -> Result<(), ModApiError>`
Subscribe to an event type.

- **Capability Required**: None
- **Parameters**:
  - `event_type`: Event type to subscribe to
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD001`: Invalid event type

#### `unsubscribe(event_type: EventType) -> Result<(), ModApiError>`
Unsubscribe from an event type.

- **Capability Required**: None
- **Parameters**:
  - `event_type`: Event type to unsubscribe from
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD003`: Not subscribed to event

#### `cancel_event() -> Result<(), ModApiError>`
Cancel the currently processing event (only valid in event handler).

- **Capability Required**:
  - `EventCancelChat` for PlayerChat events
  - `EventCancelBlocks` for BlockPlaced/BlockBroken events
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing cancellation capability
  - `EMOD001`: Event is not cancellable or not in handler context

---

### World API

#### `get_block(pos: IVec3) -> Result<BlockId, ModApiError>`
Get block type at position.

- **Capability Required**: `WorldRead`
- **Parameters**:
  - `pos`: Block position (world coordinates)
- **Returns**: Block type ID
- **Errors**:
  - `EMOD002`: Missing `world.read` capability
  - `EMOD004`: Position outside world bounds
  - `EMOD006`: Chunk not loaded

#### `set_block(pos: IVec3, block_id: BlockId) -> Result<(), ModApiError>`
Set block type at position.

- **Capability Required**: `WorldWrite`
- **Parameters**:
  - `pos`: Block position
  - `block_id`: New block type
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing `world.write` capability
  - `EMOD001`: Invalid block_id
  - `EMOD004`: Position outside world bounds
  - `EMOD006`: Chunk not loaded

#### `raycast(origin: Vec3, dir: Vec3, max_dist: f32) -> Result<Option<RaycastHit>, ModApiError>`
Cast ray and find first block hit.

- **Capability Required**: `WorldRead`
- **Parameters**:
  - `origin`: Ray start position
  - `dir`: Ray direction (will be normalized)
  - `max_dist`: Maximum distance (**clamped to 256 blocks**)
- **Returns**: `Some(RaycastHit)` if hit, `None` if no hit
- **Errors**:
  - `EMOD002`: Missing `world.read` capability
  - `EMOD001`: Invalid direction (zero vector)

**RaycastHit**:
```rust
pub struct RaycastHit {
    pub pos: IVec3,      // Block position
    pub block_id: BlockId,
    pub face: BlockFace, // Which face was hit
    pub distance: f32,   // Distance from origin
}
```

#### `query_aabb(min: IVec3, max: IVec3, limit: u32) -> Result<Vec<BlockQuery>, ModApiError>`
Query non-air blocks in axis-aligned bounding box.

- **Capability Required**: `WorldRead`
- **Parameters**:
  - `min`: Minimum corner
  - `max`: Maximum corner
  - `limit`: Maximum results (**clamped to 128**)
- **Returns**: Vector of block queries
- **Errors**:
  - `EMOD002`: Missing `world.read` capability
  - `EMOD001`: min > max on any axis
  - `EMOD006`: Any chunk in range not loaded

**BlockQuery**:
```rust
pub struct BlockQuery {
    pub pos: IVec3,
    pub block_id: BlockId,
}
```

---

### Entity API

#### `get_entity_transform(entity: EntityHandle) -> Result<Transform, ModApiError>`
Get entity position and rotation.

- **Capability Required**: `EntityRead`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: Transform (position + rotation)
- **Errors**:
  - `EMOD002`: Missing `entity.read` capability
  - `EMOD003`: Entity not found (stale handle)

#### `get_entity_velocity(entity: EntityHandle) -> Result<Vec3, ModApiError>`
Get entity velocity.

- **Capability Required**: `EntityRead`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: Velocity vector
- **Errors**:
  - `EMOD002`: Missing `entity.read` capability
  - `EMOD003`: Entity not found
  - `EMOD001`: Entity has no velocity component

#### `get_entity_health(entity: EntityHandle) -> Result<f32, ModApiError>`
Get entity health.

- **Capability Required**: `EntityRead`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: Current health value
- **Errors**:
  - `EMOD002`: Missing `entity.read` capability
  - `EMOD003`: Entity not found
  - `EMOD001`: Entity has no health component

#### `get_entity_owner(entity: EntityHandle) -> Result<Option<u64>, ModApiError>`
Get entity owner player ID.

- **Capability Required**: `EntityRead`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: `Some(player_id)` if owned, `None` otherwise
- **Errors**:
  - `EMOD002`: Missing `entity.read` capability
  - `EMOD003`: Entity not found

#### `get_entity_team(entity: EntityHandle) -> Result<Option<u8>, ModApiError>`
Get entity team ID.

- **Capability Required**: `EntityRead`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: `Some(team_id)` if on team, `None` otherwise
- **Errors**:
  - `EMOD002`: Missing `entity.read` capability
  - `EMOD003`: Entity not found

#### `apply_damage(entity: EntityHandle, amount: f32, source: Option<DamageSource>) -> Result<(), ModApiError>`
Apply damage to entity.

- **Capability Required**: `EntityWrite`
- **Parameters**:
  - `entity`: Entity handle
  - `amount`: Damage amount (positive)
  - `source`: Optional damage source info
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing `entity.write` capability
  - `EMOD003`: Entity not found
  - `EMOD001`: Negative damage amount, or entity has no health

#### `apply_impulse(entity: EntityHandle, impulse: Vec3) -> Result<(), ModApiError>`
Apply physics impulse to entity.

- **Capability Required**: `EntityWrite`
- **Parameters**:
  - `entity`: Entity handle
  - `impulse`: Impulse vector (m/s)
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing `entity.write` capability
  - `EMOD003`: Entity not found
  - `EMOD001`: Entity has no physics component

#### `spawn_entity(entity_type: EntityType, transform: Transform) -> Result<EntityHandle, ModApiError>`
Spawn a new entity.

- **Capability Required**: `EntityWrite`
- **Parameters**:
  - `entity_type`: Type of entity to spawn
  - `transform`: Initial position and rotation
- **Returns**: Handle to new entity
- **Errors**:
  - `EMOD002`: Missing `entity.write` capability
  - `EMOD001`: Invalid entity type
  - `EMOD004`: Transform position outside world bounds

#### `despawn_entity(entity: EntityHandle) -> Result<(), ModApiError>`
Remove an entity from the world.

- **Capability Required**: `EntityWrite`
- **Parameters**:
  - `entity`: Entity handle
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing `entity.write` capability
  - `EMOD003`: Entity not found
  - `EMOD002`: Cannot despawn protected entities (players)

---

### Network API

#### `send_message(channel: &str, target: MessageTarget, payload: &[u8]) -> Result<(), ModApiError>`
Send message on mod channel.

- **Capability Required**: `NetSend`
- **Parameters**:
  - `channel`: Channel name (format: `mod:<mod_id>:<name>`)
  - `target`: Message target (Server, Client(id), AllClients, Team(id))
  - `payload`: Message bytes (**max 8KB**)
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD002`: Missing `net.send` capability
  - `EMOD001`: Payload exceeds 8KB
  - `EMOD001`: Invalid channel format
  - `EMOD005`: Rate limited (>20 msg/s)

**MessageTarget**:
```rust
pub enum MessageTarget {
    Server,
    Client(u64),
    AllClients,
    Team(u8),
}
```

---

### Timer API

#### `set_timeout(delay_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError>`
Schedule one-time callback.

- **Capability Required**: None
- **Parameters**:
  - `delay_ms`: Delay in milliseconds (**clamped to min 50ms**)
  - `callback_id`: Mod-defined callback identifier
- **Returns**: Timer handle for cancellation
- **Errors**:
  - `EMOD005`: Exceeded max timers (32 per mod)

#### `set_interval(interval_ms: u32, callback_id: u32) -> Result<TimerHandle, ModApiError>`
Schedule repeating callback.

- **Capability Required**: None
- **Parameters**:
  - `interval_ms`: Interval in milliseconds (**clamped to min 50ms**)
  - `callback_id`: Mod-defined callback identifier
- **Returns**: Timer handle for cancellation
- **Errors**:
  - `EMOD005`: Exceeded max timers (32 per mod)

#### `clear_timer(handle: TimerHandle) -> Result<(), ModApiError>`
Cancel a timer.

- **Capability Required**: None
- **Parameters**:
  - `handle`: Timer handle from set_timeout/set_interval
- **Returns**: `Ok(())` on success
- **Errors**:
  - `EMOD003`: Timer not found (already fired or invalid handle)

---

## Error Codes Reference

| Code | Name | Description |
|------|------|-------------|
| EMOD001 | InvalidArgument | Invalid parameter value |
| EMOD002 | PermissionDenied | Missing required capability |
| EMOD003 | NotFound | Entity/timer/resource not found |
| EMOD004 | OutOfBounds | Position/value outside valid range |
| EMOD005 | RateLimited | Rate limit or quota exceeded |
| EMOD006 | WorldNotReady | Chunk not loaded |
| EMOD007 | Unsupported | API version mismatch |

---

## Capability Reference

| Capability | Required For |
|------------|--------------|
| `world.read` | get_block, raycast, query_aabb |
| `world.write` | set_block |
| `entity.read` | get_entity_* functions |
| `entity.write` | apply_damage, apply_impulse, spawn/despawn |
| `net.send` | send_message |
| `event.cancel.chat` | cancel PlayerChat events |
| `event.cancel.blocks` | cancel BlockPlaced/BlockBroken events |

---

## Versioning

- **API Version**: Integer, increments on breaking changes
- **Current Version**: 1 (MVP)
- **Compatibility**: Mods declare required api_version in manifest
- **Version Check**: At mod load time, before any code executes
