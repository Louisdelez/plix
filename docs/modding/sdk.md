# Plix Mod SDK Reference

Complete API documentation for the Plix Mod SDK.

## Overview

The Plix Mod SDK provides a safe, `#![no_std]` compatible Rust API for building WASM mods. All host interactions go through a capability-checked ABI.

## Module Structure

```
plix_mod_sdk
├── prelude      # Convenient re-exports
├── version      # SDK version info
├── error        # Error types
├── abi          # Low-level FFI (internal)
├── codec        # Serialization (internal)
├── caps         # Capability checking
├── events       # Event types and payloads
├── log          # Logging functions
├── world        # World API (blocks, raycast)
├── entities     # Entity API
├── net          # Networking API
└── timers       # Timer API
```

## Prelude

Import everything you need:

```rust
use plix_mod_sdk::prelude::*;
```

This includes:
- `EventType`, `EventContext`, all payload types
- `subscribe()`, logging macros (`info!`, `error!`, etc.)
- `Vec3`, `IVec3` (from glam)
- `ModResult`, `ModError`, `ErrorCode`
- World, entity, net, and timer functions

## Macros

### `#[plix_mod]`

Marks a struct as the mod entry point. Apply to both the struct and its impl block:

```rust
#[plix_mod]
struct MyMod;

#[plix_mod]
impl MyMod {
    fn init(&self) {
        // Called when mod loads
    }

    fn shutdown(&self) {
        // Called when mod unloads
    }
}
```

### `#[on_event("event_name")]`

Registers a method as an event handler:

```rust
#[on_event("on_player_chat")]
fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
    // Handle chat event
}
```

## Events

### Subscribing

```rust
subscribe(EventType::PlayerChat)?;
```

### Event Types

```rust
pub enum EventType {
    ServerStart = 0x01,
    ServerStop = 0x02,
    PlayerJoin = 0x03,
    PlayerLeave = 0x04,
    PlayerChat = 0x05,
    BlockPlaced = 0x06,
    BlockBroken = 0x07,
    EntitySpawned = 0x08,
    EntityDespawned = 0x09,
}
```

### EventContext

```rust
pub struct EventContext {
    pub event_type: EventType,
    pub tick: u64,
    pub cancellable: bool,
}

impl EventContext {
    /// Cancel the event (if cancellable)
    /// Requires appropriate capability
    pub fn cancel(&self) -> ModResult<()>;
}
```

### Payloads

```rust
pub struct PlayerJoinPayload {
    pub player_id: u64,
}

pub struct PlayerLeavePayload {
    pub player_id: u64,
}

pub struct PlayerChatPayload {
    pub player_id: u64,
    pub text: String,
}

pub struct BlockPayload {
    pub position: IVec3,
    pub block_id: u16,
    pub player_id: Option<u64>,
}

pub struct EntityPayload {
    pub entity_id: u64,
    pub entity_type: u16,
    pub position: Vec3,
}

pub struct ServerStartPayload {
    pub tick: u64,
}

pub struct ServerStopPayload {
    pub reason: String,
}
```

## Logging

```rust
error!("Critical error: {}", msg);
warn!("Warning: {}", msg);
info!("Info: {}", msg);
debug!("Debug: {}", msg);
trace!("Trace: {}", msg);
```

Log levels:
- `Error` (1) - Always shown
- `Warn` (2) - Warnings
- `Info` (3) - General info
- `Debug` (4) - Debug info (dev only)
- `Trace` (5) - Verbose tracing (dev only)

## Capabilities

### Checking

```rust
if has_capability(Capability::WorldWrite) {
    // Can modify blocks
}
```

### Available Capabilities

```rust
pub enum Capability {
    WorldRead = 0x01,       // Read blocks
    WorldWrite = 0x02,      // Modify blocks
    EntityRead = 0x04,      // Query entities
    EntityWrite = 0x08,     // Damage/push entities
    NetSend = 0x10,         // Send messages
    EventCancelChat = 0x20, // Cancel chat events
    EventCancelBlocks = 0x40, // Cancel block events
}
```

## World API

Requires: `world_read` or `world_write` capability

### Reading Blocks

```rust
/// Get block ID at position
pub fn get_block(pos: IVec3) -> ModResult<u16>;

// Example
let block_id = get_block(IVec3::new(10, 64, 10))?;
```

### Modifying Blocks

```rust
/// Set block at position (requires world_write)
pub fn set_block(pos: IVec3, block_id: u16) -> ModResult<()>;

// Example
set_block(IVec3::new(10, 64, 10), 1)?; // Place stone
```

### Raycasting

```rust
pub struct RaycastHit {
    pub position: IVec3,
    pub block_id: u16,
    pub normal: IVec3,
    pub distance: f32,
}

/// Cast a ray and find first block hit
pub fn raycast(origin: Vec3, direction: Vec3, max_distance: f32) -> ModResult<Option<RaycastHit>>;

// Example
if let Some(hit) = raycast(origin, direction, 100.0)? {
    info!("Hit block {} at {:?}", hit.block_id, hit.position);
}
```

### Area Queries

```rust
/// Query blocks in an axis-aligned bounding box
pub fn query_aabb(min: IVec3, max: IVec3) -> ModResult<Vec<(IVec3, u16)>>;

// Example - get all blocks in 5x5x5 area
let blocks = query_aabb(
    IVec3::new(0, 60, 0),
    IVec3::new(5, 65, 5)
)?;
```

## Entity API

Requires: `entity_read` or `entity_write` capability

### Entity Handle

```rust
pub struct EntityHandle(pub u64);
```

### Reading Entity Data

```rust
pub struct Transform {
    pub position: Vec3,
    pub rotation: Vec3,
    pub velocity: Vec3,
}

/// Get entity transform
pub fn get_transform(entity: EntityHandle) -> ModResult<Transform>;

// Example
let transform = get_transform(EntityHandle(player_id))?;
info!("Player at {:?}", transform.position);
```

### Modifying Entities

```rust
/// Apply damage to entity (requires entity_write)
pub fn apply_damage(entity: EntityHandle, amount: f32) -> ModResult<()>;

/// Apply impulse/knockback (requires entity_write)
pub fn apply_impulse(entity: EntityHandle, impulse: Vec3) -> ModResult<()>;

// Examples
apply_damage(EntityHandle(entity_id), 10.0)?;
apply_impulse(EntityHandle(entity_id), Vec3::new(0.0, 5.0, 0.0))?;
```

## Networking API

Requires: `net_send` capability

### Message Target

```rust
pub enum MessageTarget {
    Player(u64),
    AllPlayers,
    AllExcept(u64),
}
```

### Sending Messages

```rust
/// Send message to specific player
pub fn send(target: MessageTarget, message: &str) -> ModResult<()>;

/// Broadcast to all players
pub fn broadcast(message: &str) -> ModResult<()>;

// Examples
send(MessageTarget::Player(player_id), "Hello!")?;
broadcast("Server announcement")?;
send(MessageTarget::AllExcept(player_id), "Whisper to others")?;
```

## Timer API

### Timer Handle

```rust
pub struct TimerHandle(pub u64);
```

### One-shot Timer

```rust
/// Set a one-shot timer
pub fn set_timeout(delay_ms: u64) -> ModResult<TimerHandle>;

// Example - fire in 5 seconds
let handle = set_timeout(5000)?;
```

### Repeating Timer

```rust
/// Set a repeating timer
pub fn set_interval(interval_ms: u64) -> ModResult<TimerHandle>;

// Example - fire every minute
let handle = set_interval(60000)?;
```

### Canceling Timers

```rust
/// Cancel a timer
pub fn clear_timer(handle: TimerHandle) -> ModResult<()>;

// Example
clear_timer(handle)?;
```

## Error Handling

### ModResult

```rust
pub type ModResult<T> = Result<T, ModError>;
```

### ModError

```rust
pub struct ModError {
    pub code: ErrorCode,
    pub message: String,
}
```

### Error Codes

```rust
pub enum ErrorCode {
    InvalidArgument = 1,   // EMOD001 - Bad input
    PermissionDenied = 2,  // EMOD002 - Missing capability
    NotFound = 3,          // EMOD003 - Entity/block not found
    OutOfBounds = 4,       // EMOD004 - Position out of range
    RateLimited = 5,       // EMOD005 - Too many calls
    WorldNotReady = 6,     // EMOD006 - World not loaded
    Unsupported = 7,       // EMOD007 - Feature not available
}
```

### Handling Errors

```rust
match get_block(pos) {
    Ok(block_id) => info!("Block: {}", block_id),
    Err(e) => match e.code {
        ErrorCode::OutOfBounds => warn!("Position out of bounds"),
        ErrorCode::WorldNotReady => warn!("World not ready yet"),
        _ => error!("Unexpected error: {:?}", e),
    }
}
```

## Version Info

```rust
/// SDK ABI version (binary compatibility)
pub const SDK_ABI_VERSION: u8 = 1;

/// SDK API version (source compatibility)
pub const SDK_API_VERSION: u8 = 1;

/// Get host ABI version
pub fn get_host_abi_version() -> u8;

/// Get host API version
pub fn get_host_api_version() -> u8;

/// Check if SDK is compatible with host
pub fn is_compatible() -> bool;
```

## Best Practices

### Memory Management

Since mods run in `#![no_std]`, use `alloc`:

```rust
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
```

### Error Handling

Always handle errors gracefully:

```rust
if let Err(e) = subscribe(EventType::PlayerChat) {
    error!("Failed to subscribe: {:?}", e);
    // Mod can still function, just won't receive chat events
}
```

### Capability Checking

Check capabilities before use:

```rust
if has_capability(Capability::WorldWrite) {
    set_block(pos, block_id)?;
} else {
    warn!("Missing world_write capability");
}
```

### Performance

- Minimize host calls in hot paths
- Cache frequently accessed data
- Use batch operations when available
- Log at appropriate levels (avoid `debug!` in production)
