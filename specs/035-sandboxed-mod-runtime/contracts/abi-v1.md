# Plix Mod ABI v1 Specification

**Version**: 1.0.0
**Date**: 2025-12-18
**Status**: Draft

## Overview

This document specifies the Application Binary Interface (ABI) between the plix game engine (host) and WASM mod modules. All mods compiled for plix must conform to this interface.

## Conventions

### Data Types

| Type | Size | Description |
|------|------|-------------|
| i32 | 4 bytes | 32-bit signed integer |
| u32 | 4 bytes | 32-bit unsigned integer |
| ptr | 4 bytes | Pointer into WASM linear memory (i32) |
| len | 4 bytes | Length of data at pointer (i32) |

### Memory Layout

- All pointers reference the module's linear memory
- Data is little-endian
- Strings are UTF-8 encoded without null terminator
- Complex data is bincode-serialized

### Return Codes

All host functions return i32:
- `0` = Success
- `1` = EMOD001 (Invalid Argument)
- `2` = EMOD002 (Permission Denied)
- `3` = EMOD003 (Not Found)
- `4` = EMOD004 (Out of Bounds)
- `5` = EMOD005 (Rate Limited)
- `6` = EMOD006 (World Not Ready)
- `7` = EMOD007 (Unsupported)

## Required Exports

Mods must export these functions:

### mod_init

```wasm
(func (export "mod_init") (result i32))
```

Called once after module instantiation. Initialize mod state here.

**Returns**: 0 on success, error code on failure

### mod_on_event

```wasm
(func (export "mod_on_event") (param $event_id i32) (param $payload_ptr i32) (param $payload_len i32) (result i32))
```

Called when a subscribed event occurs.

**Parameters**:
- `event_id`: Event type identifier
- `payload_ptr`: Pointer to serialized event data
- `payload_len`: Length of event data

**Returns**: 0 on success, error code on failure

### mod_shutdown

```wasm
(func (export "mod_shutdown") (result i32))
```

Called before module unload. Clean up resources here.

**Returns**: 0 on success, error code on failure

### memory

```wasm
(memory (export "memory") 1)
```

Linear memory must be exported for host access.

## Host Functions

All host functions are in the "plix" namespace.

### Logging

#### plix_log

```wasm
(import "plix" "log" (func $plix_log (param $level i32) (param $msg_ptr i32) (param $msg_len i32) (result i32)))
```

Log a message to the server log.

**Parameters**:
- `level`: Log level (0=Error, 1=Warn, 2=Info, 3=Debug, 4=Trace)
- `msg_ptr`: Pointer to UTF-8 message
- `msg_len`: Length of message

**Returns**: 0 on success

**Capability**: None required

### Version Queries

#### plix_get_api_version

```wasm
(import "plix" "get_api_version" (func $plix_get_api_version (result i32)))
```

Get the engine API version.

**Returns**: API version number (currently 1)

**Capability**: None required

#### plix_get_abi_version

```wasm
(import "plix" "get_abi_version" (func $plix_get_abi_version (result i32)))
```

Get the ABI version.

**Returns**: ABI version number (currently 1)

**Capability**: None required

### Capabilities

#### plix_has_capability

```wasm
(import "plix" "has_capability" (func $plix_has_capability (param $cap_id i32) (result i32)))
```

Check if mod has a specific capability.

**Parameters**:
- `cap_id`: Capability identifier (see Capability IDs below)

**Returns**: 1 if granted, 0 if not

**Capability**: None required

### Events

#### plix_subscribe_event

```wasm
(import "plix" "subscribe_event" (func $plix_subscribe_event (param $event_type i32) (result i32)))
```

Subscribe to an event type.

**Parameters**:
- `event_type`: Event type identifier (see Event Types below)

**Returns**: 0 on success

**Capability**: None required

#### plix_cancel_event

```wasm
(import "plix" "cancel_event" (func $plix_cancel_event (result i32)))
```

Cancel the current event (if cancellable and permitted).

**Returns**: 0 if cancelled, EMOD002 if not permitted

**Capability**: EVENT_CANCEL_CHAT (for chat events) or EVENT_CANCEL_BLOCKS (for block events)

### Response Buffer

#### plix_response_ptr

```wasm
(import "plix" "response_ptr" (func $plix_response_ptr (result i32)))
```

Get pointer to host response buffer.

**Returns**: Pointer to start of response buffer

#### plix_response_len

```wasm
(import "plix" "response_len" (func $plix_response_len (result i32)))
```

Get length of data in response buffer (after a host call).

**Returns**: Length of response data

### World API

#### plix_world_call

```wasm
(import "plix" "world_call" (func $plix_world_call (param $op i32) (param $req_ptr i32) (param $req_len i32) (result i32)))
```

Call a World API operation.

**Parameters**:
- `op`: Operation code (0x30-0x3F)
- `req_ptr`: Pointer to bincode-serialized request
- `req_len`: Length of request

**Returns**: 0 on success, response in response buffer

**Capability**: WORLD_READ (for reads) or WORLD_WRITE (for writes)

**Operations**:
| Op | Name | Capability | Request | Response |
|----|------|-----------|---------|----------|
| 0x30 | get_block | WORLD_READ | IVec3 | BlockId |
| 0x31 | set_block | WORLD_WRITE | (IVec3, BlockId) | () |
| 0x32 | raycast | WORLD_READ | RaycastRequest | Option<RaycastHit> |
| 0x33 | query_aabb | WORLD_READ | AabbRequest | Vec<IVec3> |

### Entity API

#### plix_entity_call

```wasm
(import "plix" "entity_call" (func $plix_entity_call (param $op i32) (param $req_ptr i32) (param $req_len i32) (result i32)))
```

Call an Entity API operation.

**Parameters**:
- `op`: Operation code (0x40-0x4F)
- `req_ptr`: Pointer to bincode-serialized request
- `req_len`: Length of request

**Returns**: 0 on success, response in response buffer

**Capability**: ENTITY_READ (for reads) or ENTITY_WRITE (for writes)

**Operations**:
| Op | Name | Capability | Request | Response |
|----|------|-----------|---------|----------|
| 0x40 | get_transform | ENTITY_READ | EntityHandle | Transform |
| 0x41 | get_health | ENTITY_READ | EntityHandle | u32 |
| 0x42 | apply_damage | ENTITY_WRITE | (EntityHandle, u32) | () |
| 0x43 | apply_impulse | ENTITY_WRITE | (EntityHandle, Vec3) | () |

### Net API

#### plix_net_call

```wasm
(import "plix" "net_call" (func $plix_net_call (param $op i32) (param $req_ptr i32) (param $req_len i32) (result i32)))
```

Call a Net API operation.

**Parameters**:
- `op`: Operation code (0x50-0x5F)
- `req_ptr`: Pointer to bincode-serialized request
- `req_len`: Length of request

**Returns**: 0 on success, EMOD005 if rate limited

**Capability**: NET_SEND

**Operations**:
| Op | Name | Request | Response |
|----|------|---------|----------|
| 0x50 | send | (PlayerId, Channel, Payload) | () |
| 0x51 | broadcast | (Channel, Payload) | () |

**Limits**:
- Max payload: 8192 bytes
- Rate limit: 20 messages/second per mod

### Timer API

#### plix_timer_call

```wasm
(import "plix" "timer_call" (func $plix_timer_call (param $op i32) (param $req_ptr i32) (param $req_len i32) (result i32)))
```

Call a Timer API operation.

**Parameters**:
- `op`: Operation code (0x60-0x6F)
- `req_ptr`: Pointer to bincode-serialized request
- `req_len`: Length of request

**Returns**: 0 on success, timer handle in response buffer

**Capability**: None required

**Operations**:
| Op | Name | Request | Response |
|----|------|---------|----------|
| 0x60 | set_timeout | (u64 ms, u32 callback_id) | TimerHandle |
| 0x61 | set_interval | (u64 ms, u32 callback_id) | TimerHandle |
| 0x62 | clear | TimerHandle | () |

**Limits**:
- Min interval: 50ms
- Max timers per mod: 32

## Capability IDs

| ID | Name | Description |
|----|------|-------------|
| 0x01 | WORLD_READ | Read world state |
| 0x02 | WORLD_WRITE | Modify world state |
| 0x04 | ENTITY_READ | Read entity state |
| 0x08 | ENTITY_WRITE | Modify entities |
| 0x10 | NET_SEND | Send network messages |
| 0x20 | EVENT_CANCEL_CHAT | Cancel chat events |
| 0x40 | EVENT_CANCEL_BLOCKS | Cancel block events |

## Event Types

| ID | Name | Cancellable | Payload |
|----|------|-------------|---------|
| 0x01 | ServerStart | No | tick: u64 |
| 0x02 | ServerStop | No | tick: u64 |
| 0x03 | PlayerJoin | No | player_id: u64, name: String |
| 0x04 | PlayerLeave | No | player_id: u64 |
| 0x05 | PlayerChat | Yes | player_id: u64, message: String |
| 0x06 | BlockPlaced | Yes | player_id: Option<u64>, pos: IVec3, block_id: u8 |
| 0x07 | BlockBroken | Yes | player_id: Option<u64>, pos: IVec3, block_id: u8 |
| 0x08 | EntitySpawned | No | entity_id: u64, entity_type: u8 |
| 0x09 | EntityDespawned | No | entity_id: u64 |

## Security Considerations

1. **Pointer Validation**: All pointer/length pairs are validated against linear memory bounds
2. **Capability Enforcement**: All API calls check required capabilities
3. **Rate Limiting**: Net API is rate limited to prevent spam
4. **CPU Budget**: Execution is interrupted if fuel is exhausted
5. **Memory Limit**: memory.grow fails if limit exceeded

## Versioning

- ABI version changes are backward-incompatible
- API version changes within same ABI are backward-compatible
- Mods should check both versions in mod_init

## Example

```rust
// Minimal mod in Rust
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    // Subscribe to chat events
    unsafe { plix_subscribe_event(0x05) };
    0
}

#[no_mangle]
pub extern "C" fn mod_on_event(event_id: i32, ptr: i32, len: i32) -> i32 {
    if event_id == 0x05 {
        // Log that we received a chat event
        let msg = b"Chat event received";
        unsafe { plix_log(2, msg.as_ptr() as i32, msg.len() as i32) };
    }
    0
}

#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0
}

extern "C" {
    fn plix_log(level: i32, ptr: i32, len: i32) -> i32;
    fn plix_subscribe_event(event_type: i32) -> i32;
}
```
