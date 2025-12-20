# Quickstart: Sandboxed Mod Runtime (WASM)

**Feature**: 035-sandboxed-mod-runtime
**Date**: 2025-12-18

## Prerequisites

- Rust 1.83+ installed
- wasm32-unknown-unknown target: `rustup target add wasm32-unknown-unknown`
- plix server built from source

## Creating a Simple Mod

### 1. Create Mod Project

```bash
cargo new --lib my-chat-mod
cd my-chat-mod
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-chat-mod"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
opt-level = "s"  # Optimize for size
```

### 3. Create mod.toml (Manifest)

```toml
id = "my-chat-mod"
name = "My Chat Mod"
version = "0.1.0"
author = "Your Name"
api_version = 1

[capabilities]
world_read = true
event_cancel_chat = true

[entrypoints]
server = "mod_init"
```

### 4. Implement the Mod (src/lib.rs)

```rust
//! A simple chat mod that logs all messages and optionally filters them.

// Host function imports
extern "C" {
    fn plix_log(level: i32, ptr: i32, len: i32) -> i32;
    fn plix_subscribe_event(event_type: i32) -> i32;
    fn plix_cancel_event() -> i32;
    fn plix_has_capability(cap_id: i32) -> i32;
}

// Event type constants
const EVENT_PLAYER_CHAT: i32 = 0x05;

// Capability constants
const CAP_EVENT_CANCEL_CHAT: i32 = 0x20;

// Log levels
const LOG_INFO: i32 = 2;

/// Called once when mod is loaded
#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    log_message("Chat mod initializing...");

    // Subscribe to chat events
    unsafe {
        plix_subscribe_event(EVENT_PLAYER_CHAT);
    }

    log_message("Chat mod initialized!");
    0 // Success
}

/// Called for each subscribed event
#[no_mangle]
pub extern "C" fn mod_on_event(event_id: i32, _payload_ptr: i32, _payload_len: i32) -> i32 {
    if event_id == EVENT_PLAYER_CHAT {
        log_message("Received chat event");

        // Check if we can cancel events
        let can_cancel = unsafe { plix_has_capability(CAP_EVENT_CANCEL_CHAT) };

        if can_cancel == 1 {
            // Example: cancel messages containing "spam"
            // In real mod, you'd parse the payload to check the message
            // For demo, we just log that we could cancel
            log_message("Can cancel chat events if needed");
        }
    }

    0 // Success
}

/// Called when mod is unloaded
#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    log_message("Chat mod shutting down");
    0 // Success
}

/// Helper to log messages
fn log_message(msg: &str) {
    unsafe {
        plix_log(LOG_INFO, msg.as_ptr() as i32, msg.len() as i32);
    }
}
```

### 5. Build the WASM Module

```bash
cargo build --target wasm32-unknown-unknown --release
```

The output will be at: `target/wasm32-unknown-unknown/release/my_chat_mod.wasm`

### 6. Package the Mod

Create a mod directory:

```bash
mkdir my-chat-mod-pkg
cp mod.toml my-chat-mod-pkg/
cp target/wasm32-unknown-unknown/release/my_chat_mod.wasm my-chat-mod-pkg/mod.wasm
```

### 7. Install on Server

Copy the package to the server's mods directory:

```bash
cp -r my-chat-mod-pkg /path/to/plix-server/mods/my-chat-mod/
```

### 8. Run the Server

The server will automatically load mods from the `mods/` directory:

```bash
./plix-server
```

Look for log messages like:
```
INFO  plix_server::mods: Loading mod: my-chat-mod
INFO  [my-chat-mod] Chat mod initializing...
INFO  [my-chat-mod] Chat mod initialized!
```

## Working with Payloads

To parse event payloads, you'll need to deserialize bincode data. Here's a pattern:

```rust
use core::slice;

/// Read bytes from WASM memory
fn read_payload(ptr: i32, len: i32) -> &'static [u8] {
    unsafe {
        slice::from_raw_parts(ptr as *const u8, len as usize)
    }
}

/// Minimal bincode deserialization for chat event
fn parse_chat_event(data: &[u8]) -> Option<(u64, &str)> {
    // Format: player_id (u64) + message length (u32) + message bytes
    if data.len() < 12 {
        return None;
    }

    let player_id = u64::from_le_bytes(data[0..8].try_into().ok()?);
    let msg_len = u32::from_le_bytes(data[8..12].try_into().ok()?) as usize;

    if data.len() < 12 + msg_len {
        return None;
    }

    let msg = core::str::from_utf8(&data[12..12 + msg_len]).ok()?;
    Some((player_id, msg))
}
```

## Calling World API

To read blocks from the world:

```rust
extern "C" {
    fn plix_world_call(op: i32, ptr: i32, len: i32) -> i32;
    fn plix_response_ptr() -> i32;
    fn plix_response_len() -> i32;
}

const OP_GET_BLOCK: i32 = 0x30;

fn get_block(x: i32, y: i32, z: i32) -> Option<u8> {
    // Serialize position (3 i32s = 12 bytes)
    let mut buf = [0u8; 12];
    buf[0..4].copy_from_slice(&x.to_le_bytes());
    buf[4..8].copy_from_slice(&y.to_le_bytes());
    buf[8..12].copy_from_slice(&z.to_le_bytes());

    let result = unsafe {
        plix_world_call(OP_GET_BLOCK, buf.as_ptr() as i32, 12)
    };

    if result == 0 {
        // Read response
        let ptr = unsafe { plix_response_ptr() } as *const u8;
        let len = unsafe { plix_response_len() } as usize;

        if len >= 1 {
            let block_id = unsafe { *ptr };
            return Some(block_id);
        }
    }

    None
}
```

## Limits and Best Practices

### Resource Limits

| Resource | Limit | Notes |
|----------|-------|-------|
| Memory | 32 MiB | Per-mod linear memory |
| CPU per handler | 5 ms | Interrupted if exceeded |
| Network messages | 20/sec | Rate limited per mod |
| Payload size | 8 KB | Max for net messages |
| Timers | 32 | Max per mod |
| Min timer interval | 50 ms | Clamped if lower |

### Best Practices

1. **Keep handlers fast**: Stay well under the 5ms budget
2. **Cache capability checks**: Check once in mod_init, not every event
3. **Minimize allocations**: Reuse buffers where possible
4. **Handle errors gracefully**: Check return codes from all host calls
5. **Log sparingly in hot paths**: Logging has overhead

### Error Handling

Always check return codes:

```rust
let result = unsafe { plix_world_call(op, ptr, len) };
match result {
    0 => { /* Success, read response */ }
    2 => { /* EMOD002: Permission denied */ }
    5 => { /* EMOD005: Rate limited */ }
    _ => { /* Other error */ }
}
```

## Debugging

Enable debug mode in server config:

```toml
[mods.wasm]
debug = true
```

This logs all host function calls with parameters and results.

## Next Steps

- See `contracts/abi-v1.md` for complete ABI reference
- See `data-model.md` for internal architecture
- Check example mods in `tests/fixtures/`
