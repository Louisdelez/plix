# Data Model: Mod API Core

**Feature**: 034-mod-api-core
**Date**: 2025-12-18

## Core Entities

### ModManifest

Represents the parsed and validated mod manifest from `mod.toml`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | String | Yes | Unique mod identifier (snake_case or kebab-case) |
| name | String | Yes | Human-readable display name |
| version | SemVer | Yes | Mod version (e.g., "1.0.0") |
| author | String | No | Mod author name |
| api_version | u32 | Yes | Required API version (MVP = 1) |
| min_api_version | u32 | No | Minimum compatible API version |
| max_api_version | u32 | No | Maximum compatible API version |
| capabilities | Vec<Capability> | Yes | Requested permissions |
| entrypoints | Entrypoints | No | Entry point declarations |
| dependencies | Vec<Dependency> | No | Other mod dependencies |

**Validation Rules**:
- `id` must match pattern `^[a-z][a-z0-9_-]*$`, max 64 chars
- `version` must be valid SemVer
- `api_version` must be supported by engine (MVP: 1)
- `capabilities` must all be recognized

**Example**:
```toml
id = "my-mod"
name = "My Awesome Mod"
version = "1.0.0"
author = "ModDeveloper"
api_version = 1

[capabilities]
world_read = true
world_write = true
entity_read = true
net_send = true

[entrypoints]
server = "server_main"
```

---

### Capability

Permission flags that mods request and engine grants/denies.

| Variant | Description | Required For |
|---------|-------------|--------------|
| WorldRead | Read world state | get_block, raycast, query_aabb |
| WorldWrite | Modify world state | set_block |
| EntityRead | Read entity state | get_transform, get_health, etc. |
| EntityWrite | Modify entities | apply_damage, apply_impulse, spawn/despawn |
| NetSend | Send network messages | send_message |
| EventCancelChat | Cancel chat events | cancel on_player_chat |
| EventCancelBlocks | Cancel block events | cancel on_block_placed/broken |

**Implementation**: Bitflags for efficient checking

```rust
bitflags! {
    pub struct Capability: u32 {
        const WORLD_READ = 0b0000001;
        const WORLD_WRITE = 0b0000010;
        const ENTITY_READ = 0b0000100;
        const ENTITY_WRITE = 0b0001000;
        const NET_SEND = 0b0010000;
        const EVENT_CANCEL_CHAT = 0b0100000;
        const EVENT_CANCEL_BLOCKS = 0b1000000;
    }
}
```

---

### ModContext

Runtime state for a loaded mod instance.

| Field | Type | Description |
|-------|------|-------------|
| id | ModId | Unique mod identifier |
| manifest | ModManifest | Parsed manifest |
| granted_capabilities | Capability | Effective permissions (manifest ∩ server policy) |
| state | ModState | Current mod state |
| error_count | u32 | Consecutive handler errors |
| subscriptions | HashSet<EventType> | Subscribed event types |
| active_timers | Vec<TimerHandle> | Active timer handles |
| net_rate_limiter | TokenBucket | Network rate limiter state |

**State Transitions**:
```
Loading → Active → Disabled
                ↑
         (5 consecutive errors)
```

---

### ModState

Lifecycle state of a mod.

| Variant | Description |
|---------|-------------|
| Loading | Manifest validated, initializing |
| Active | Fully operational |
| Disabled | Disabled due to errors or admin action |

---

### GameEvent

An event emitted by the engine for mod handlers.

| Field | Type | Description |
|-------|------|-------------|
| event_type | EventType | Type discriminator |
| payload | EventPayload | Type-specific data |
| timestamp | u64 | Server tick when emitted |
| cancellable | bool | Whether event can be cancelled |
| cancelled | bool | Set by handler if cancelling |

---

### EventType

Enum of all MVP event types.

| Variant | Payload | Cancellable |
|---------|---------|-------------|
| ServerStart | ServerStartPayload | No |
| ServerStop | ServerStopPayload | No |
| PlayerJoin | PlayerJoinPayload | No |
| PlayerLeave | PlayerLeavePayload | No |
| PlayerChat | PlayerChatPayload | Yes |
| BlockPlaced | BlockPlacedPayload | Yes |
| BlockBroken | BlockBrokenPayload | Yes |
| EntityDamaged | EntityDamagedPayload | No |
| ModMessage | ModMessagePayload | No |

---

### Event Payloads

#### ServerStartPayload
| Field | Type | Description |
|-------|------|-------------|
| tick | u64 | Server tick number |

#### ServerStopPayload
| Field | Type | Description |
|-------|------|-------------|
| reason | String | Shutdown reason |

#### PlayerJoinPayload
| Field | Type | Description |
|-------|------|-------------|
| player_id | u64 | Unique player ID |
| name | String | Player display name |

#### PlayerLeavePayload
| Field | Type | Description |
|-------|------|-------------|
| player_id | u64 | Unique player ID |
| reason | LeaveReason | Why player left |

#### PlayerChatPayload
| Field | Type | Description |
|-------|------|-------------|
| player_id | u64 | Sender player ID |
| text | String | Message content |

#### BlockPlacedPayload
| Field | Type | Description |
|-------|------|-------------|
| player_id | Option<u64> | Placer (None if world-generated) |
| pos | IVec3 | Block position |
| block_id | u16 | Block type ID |

#### BlockBrokenPayload
| Field | Type | Description |
|-------|------|-------------|
| player_id | Option<u64> | Breaker (None if world-generated) |
| pos | IVec3 | Block position |
| block_id | u16 | Previous block type ID |

#### EntityDamagedPayload
| Field | Type | Description |
|-------|------|-------------|
| entity_id | EntityHandle | Damaged entity |
| amount | f32 | Damage amount |
| source | Option<DamageSource> | Damage source info |

#### ModMessagePayload
| Field | Type | Description |
|-------|------|-------------|
| channel | String | Channel name (mod:id:name) |
| from | MessageSource | Sender (Server or Client(id)) |
| payload | Vec<u8> | Message bytes (max 8KB) |

---

### ModApiError

Error type returned by all API calls.

| Field | Type | Description |
|-------|------|-------------|
| code | ErrorCode | Machine-readable code |
| message | String | Human-readable description |
| context | ErrorContext | Additional context |

---

### ErrorCode

| Code | Name | Description |
|------|------|-------------|
| EMOD001 | InvalidArgument | Invalid parameter value |
| EMOD002 | PermissionDenied | Missing required capability |
| EMOD003 | NotFound | Entity/timer not found |
| EMOD004 | OutOfBounds | Position/value outside valid range |
| EMOD005 | RateLimited | Rate limit exceeded |
| EMOD006 | WorldNotReady | Chunk not loaded |
| EMOD007 | Unsupported | API version mismatch |

---

### ErrorContext

| Field | Type | Description |
|-------|------|-------------|
| mod_id | ModId | Mod that triggered error |
| api_call | String | API function name |
| params | String | Parameter summary |

---

### EntityHandle

Opaque handle for safe entity reference.

| Field | Type | Description |
|-------|------|-------------|
| index | u32 | Entity index |
| generation | u32 | Generation counter |

**Safety**: Generation counter detects use-after-despawn. Stale handles return EMOD003.

---

### TimerHandle

Handle for a registered timer.

| Field | Type | Description |
|-------|------|-------------|
| id | u32 | Unique timer ID within mod |

---

### ModChannel

Network channel for mod messaging.

| Field | Type | Description |
|-------|------|-------------|
| mod_id | ModId | Owning mod |
| name | String | Channel name |
| direction | ChannelDirection | Allowed directions |

**Format**: `mod:{mod_id}:{name}`

---

### ChannelDirection

| Variant | Description |
|---------|-------------|
| ServerToClient | Server can send to clients |
| ClientToServer | Clients can send to server |
| Bidirectional | Both directions allowed |

---

## Relationships

```
ModManifest ──1:1──► ModContext
     │
     └──1:N──► Capability

ModContext ──1:N──► EventType (subscriptions)
     │
     └──1:N──► TimerHandle (active timers)

GameEvent ──1:1──► EventPayload (variant)
     │
     └──N:M──► ModContext (handlers)

EntityHandle ──1:1──► Entity (engine internal)
```

## Bounds & Limits

| Limit | Value | Error |
|-------|-------|-------|
| Raycast max_dist | 256 blocks | Clamped |
| query_aabb limit | 128 results | Clamped |
| Timer min_interval | 50ms | Clamped |
| Timer max_count | 32 per mod | EMOD005 |
| Net payload max | 8KB | EMOD001 |
| Net rate limit | 20 msg/s | EMOD005 |
| Error threshold | 5 consecutive | Auto-disable |
| Mod ID max length | 64 chars | EMOD001 |
