# SDK API Contract

**Crate**: `plix-mod-sdk`
**Version**: 0.1.0
**ABI Version**: 1
**API Version**: 1

## Public API Surface

### Prelude (`plix_mod_sdk::prelude`)

Re-exports all commonly used types:
```rust
pub use crate::{
    log::{info, warn, error, debug},
    caps::{has_capability, Capability},
    events::{subscribe, cancel, EventContext, EventType},
    world::{get_block, set_block, raycast, query_aabb},
    entities::{get_transform, apply_damage, apply_impulse},
    net::{send, broadcast},
    timers::{set_timeout, set_interval, clear_timer},
    error::{ModError, ErrorCode, Result},
    version::{SDK_ABI_VERSION, SDK_API_VERSION},
};
```

### Version Constants (`plix_mod_sdk::version`)

```rust
pub const SDK_ABI_VERSION: u8 = 1;
pub const SDK_API_VERSION: u8 = 1;

pub fn check_compatibility() -> bool;
pub fn assert_compatible();  // Logs warning if mismatch
```

### Logging (`plix_mod_sdk::log`)

```rust
pub fn log(level: LogLevel, message: &str) -> Result<()>;

// Convenience macros
info!("message");
info!("format {}", arg);
warn!("message");
error!("message");
debug!("message");

pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}
```

### Capabilities (`plix_mod_sdk::caps`)

```rust
pub fn has_capability(cap: Capability) -> bool;

#[repr(u32)]
pub enum Capability {
    WorldRead = 0x01,
    WorldWrite = 0x02,
    EntityRead = 0x04,
    EntityWrite = 0x08,
    NetSend = 0x10,
    EventCancelChat = 0x20,
    EventCancelBlocks = 0x40,
}
```

### Events (`plix_mod_sdk::events`)

```rust
pub fn subscribe(event: EventType) -> Result<()>;
pub fn cancel() -> Result<()>;  // Only in event handler, requires capability

pub struct EventContext {
    pub event_type: EventType,
    pub tick: u64,
    pub cancellable: bool,
}

impl EventContext {
    pub fn cancel(&self) -> Result<()>;
}

#[repr(u8)]
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

// Payload types
pub struct PlayerChatPayload {
    pub player_id: u64,
    pub text: String,
}

pub struct BlockPlacedPayload {
    pub player_id: Option<u64>,
    pub pos: IVec3,
    pub block_id: u16,
}

// ... similar for other events
```

### World (`plix_mod_sdk::world`)

```rust
// Requires WORLD_READ
pub fn get_block(pos: IVec3) -> Result<u16>;
pub fn raycast(origin: Vec3, direction: Vec3, max_distance: f32) -> Result<Option<RaycastHit>>;
pub fn query_aabb(min: IVec3, max: IVec3) -> Result<Vec<BlockQuery>>;

// Requires WORLD_WRITE
pub fn set_block(pos: IVec3, block_id: u16) -> Result<()>;

pub struct RaycastHit {
    pub pos: IVec3,
    pub block_id: u16,
    pub face: BlockFace,
    pub distance: f32,
}

pub struct BlockQuery {
    pub pos: IVec3,
    pub block_id: u16,
}

pub enum BlockFace {
    Top, Bottom, North, South, East, West,
}
```

### Entities (`plix_mod_sdk::entities`)

```rust
// Requires ENTITY_READ
pub fn get_transform(entity: EntityHandle) -> Result<Transform>;
pub fn get_health(entity: EntityHandle) -> Result<f32>;
pub fn get_velocity(entity: EntityHandle) -> Result<Vec3>;

// Requires ENTITY_WRITE
pub fn apply_damage(entity: EntityHandle, amount: f32, source: Option<DamageSource>) -> Result<()>;
pub fn apply_impulse(entity: EntityHandle, impulse: Vec3) -> Result<()>;

pub struct EntityHandle {
    pub index: u32,
    pub generation: u32,
}

pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

### Network (`plix_mod_sdk::net`)

```rust
// Requires NET_SEND
pub fn send(channel: &str, target: MessageTarget, payload: &[u8]) -> Result<()>;
pub fn broadcast(channel: &str, payload: &[u8]) -> Result<()>;

pub enum MessageTarget {
    Server,
    Client(u64),
    AllClients,
    Team(u8),
}
```

### Timers (`plix_mod_sdk::timers`)

```rust
pub fn set_timeout(delay_ms: u32, callback_id: u32) -> Result<TimerHandle>;
pub fn set_interval(interval_ms: u32, callback_id: u32) -> Result<TimerHandle>;
pub fn clear_timer(handle: TimerHandle) -> Result<()>;

pub struct TimerHandle {
    pub id: u32,
}
```

### Errors (`plix_mod_sdk::error`)

```rust
pub type Result<T> = std::result::Result<T, ModError>;

#[derive(Debug, Clone)]
pub struct ModError {
    pub code: ErrorCode,
    pub message: String,
}

#[repr(u8)]
pub enum ErrorCode {
    InvalidArgument = 1,
    PermissionDenied = 2,
    NotFound = 3,
    OutOfBounds = 4,
    RateLimited = 5,
    WorldNotReady = 6,
    Unsupported = 7,
}

impl std::fmt::Display for ModError { ... }
impl std::error::Error for ModError { ... }
```

### Macros (`plix_mod_sdk_macros`)

```rust
/// Marks a struct as a mod entry point.
/// Generates mod_init, mod_on_event, mod_shutdown exports.
#[plix_mod]

/// Marks a function as an event handler.
/// Must be inside a #[plix_mod] impl block.
#[on_event("event_name")]

// Usage:
#[plix_mod]
struct MyMod;

#[plix_mod]
impl MyMod {
    fn init(&self) {
        // Called on mod_init
    }

    fn shutdown(&self) {
        // Called on mod_shutdown
    }

    #[on_event("on_player_chat")]
    fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
        // Handle chat event
    }
}
```

## Constraints

| API | Constraint |
|-----|------------|
| Log message | Max 4096 bytes |
| Network payload | Max 8 KB |
| Network rate | Max 20 msg/s per mod |
| Timer interval | Min 50 ms |
| Max timers | 32 per mod |
| Raycast distance | Max 256 blocks |
| AABB query results | Max 128 |

## Error Mapping

| EMOD Code | SDK Error | Typical Cause |
|-----------|-----------|---------------|
| EMOD001 | InvalidArgument | Bad parameter value |
| EMOD002 | PermissionDenied | Missing capability |
| EMOD003 | NotFound | Entity/timer gone |
| EMOD004 | OutOfBounds | Position invalid |
| EMOD005 | RateLimited | Too many calls |
| EMOD006 | WorldNotReady | Chunk not loaded |
| EMOD007 | Unsupported | API not available |
