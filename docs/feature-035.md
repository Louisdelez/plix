# Feature 035: Sandboxed Mod Runtime (WASM)

This document describes the WASM-based mod runtime for plix servers.

## Overview

The plix mod runtime provides a secure, sandboxed environment for executing server-side mods using WebAssembly. Mods run in complete isolation from the host operating system with configurable resource limits.

## Architecture

```
plix-server
└── ModManager
    ├── ModRegistry (manifest storage)
    ├── EventBus (event dispatch)
    └── WasmBridge
        └── WasmRuntime (plix-mod-runtime-wasm)
            └── wasmtime Engine
                └── ModInstance (per-mod)
```

## Security Model

### Sandbox Guarantees

- **No filesystem access**: Mods cannot read or write files
- **No network access**: Mods cannot open sockets or make HTTP requests
- **No OS access**: Mods cannot execute shell commands or access environment variables
- **No WASI**: WASI imports are explicitly rejected

### Resource Limits

| Resource | Default Limit | Configurable |
|----------|---------------|--------------|
| CPU per handler | 5 ms | Yes |
| Memory per mod | 32 MiB | Yes |
| Network messages | 20/sec per mod | Via plix-mod-core |
| Timers per mod | 32 | Via plix-mod-core |

### Capability System

Mods must declare capabilities in their `mod.toml`:

```toml
[capabilities]
world_read = true
world_write = false
entity_read = true
entity_write = false
net_send = false
event_cancel_chat = true
event_cancel_blocks = false
```

All API calls check capabilities before execution.

## ABI v1 Specification

See `specs/035-sandboxed-mod-runtime/contracts/abi-v1.md` for the complete ABI specification.

### Required Exports

Mods must export:

```rust
#[no_mangle] pub extern "C" fn mod_init() -> i32;
#[no_mangle] pub extern "C" fn mod_on_event(event_id: i32, ptr: i32, len: i32) -> i32;
#[no_mangle] pub extern "C" fn mod_shutdown() -> i32;
// Plus: memory export
```

### Host Functions

Available in the `plix` namespace:

- `plix_log(level, ptr, len) -> i32` - Log a message
- `plix_get_api_version() -> i32` - Get engine API version
- `plix_get_abi_version() -> i32` - Get ABI version
- `plix_has_capability(cap_id) -> i32` - Check capability
- `plix_subscribe_event(event_type) -> i32` - Subscribe to events
- `plix_cancel_event() -> i32` - Cancel current event
- `plix_world_call(op, ptr, len) -> i32` - World API calls
- `plix_entity_call(op, ptr, len) -> i32` - Entity API calls
- `plix_net_call(op, ptr, len) -> i32` - Network API calls
- `plix_timer_call(op, ptr, len) -> i32` - Timer API calls
- `plix_response_ptr() -> i32` - Get response buffer pointer
- `plix_response_len() -> i32` - Get response length

## Error Codes

| Code | Name | Description |
|------|------|-------------|
| EMOD001 | InvalidArgument | Invalid parameter value |
| EMOD002 | PermissionDenied | Missing required capability |
| EMOD003 | NotFound | Entity/timer/resource not found |
| EMOD004 | OutOfBounds | Position/value outside valid range |
| EMOD005 | RateLimited | Rate limit or quota exceeded |
| EMOD006 | WorldNotReady | Chunk not loaded |
| EMOD007 | Unsupported | API version mismatch |

## Creating a Mod

### 1. Project Setup

```toml
# Cargo.toml
[package]
name = "my-mod"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
lto = true
opt-level = "s"
```

### 2. Manifest

```toml
# mod.toml
id = "my-mod"
name = "My Mod"
version = "0.1.0"
api_version = 1

[capabilities]
world_read = true
event_cancel_chat = true
```

### 3. Implementation

```rust
// src/lib.rs
extern "C" {
    fn plix_log(level: i32, ptr: i32, len: i32) -> i32;
    fn plix_subscribe_event(event_type: i32) -> i32;
}

#[no_mangle]
pub extern "C" fn mod_init() -> i32 {
    let msg = b"My mod initialized";
    unsafe { plix_log(2, msg.as_ptr() as i32, msg.len() as i32) };
    unsafe { plix_subscribe_event(0x05) }; // PlayerChat
    0
}

#[no_mangle]
pub extern "C" fn mod_on_event(event_id: i32, _ptr: i32, _len: i32) -> i32 {
    if event_id == 0x05 {
        let msg = b"Chat event received";
        unsafe { plix_log(2, msg.as_ptr() as i32, msg.len() as i32) };
    }
    0
}

#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 {
    0
}
```

### 4. Build

```bash
rustup target add wasm32-unknown-unknown
cargo build --target wasm32-unknown-unknown --release
```

### 5. Package

```
my-mod/
├── mod.toml
└── mod.wasm  # from target/wasm32-unknown-unknown/release/
```

## Server Configuration

### Enabling WASM Mods

WASM mod support is enabled by default when using `ModManager::with_wasm_runtime()`:

```rust
let mut mod_manager = ModManager::with_wasm_runtime(server_root);
mod_manager.load_mods()?;
```

### Custom Configuration

```rust
use plix_mod_runtime_wasm::RuntimeConfig;

let config = RuntimeConfig::default()
    .with_handler_budget_ms(10)      // 10ms per handler
    .with_memory_limit(64 * 1024 * 1024)  // 64 MiB
    .with_violation_threshold(10);   // Auto-disable after 10 errors

let mut mod_manager = ModManager::with_wasm_config(server_root, config);
```

## Auto-Disable Behavior

Mods are automatically disabled after consecutive errors:

1. Handler traps (out-of-bounds, division by zero)
2. Fuel exhaustion (CPU budget exceeded)
3. Memory limit exceeded

Default threshold: 5 consecutive errors. Each successful handler call resets the counter.

## Metrics

Per-mod metrics are available:

```rust
if let Some(metrics) = mod_manager.wasm_mod_metrics("my-mod") {
    println!("CPU time: {:.2}ms", metrics.cpu_time_ms());
    println!("Host calls: {}", metrics.host_call_count);
    println!("Traps: {}", metrics.trap_count);
}
```

## Debugging

Enable debug mode for verbose logging:

```rust
let config = RuntimeConfig::default().with_debug(true);
```

This logs all host function calls with parameters and results.

## Testing

Test mods should:

1. Verify in isolation using `plix-mod-runtime-wasm` directly
2. Test via `ModManager` for integration
3. Use fixtures in `crates/plix-mod-runtime-wasm/tests/fixtures/`

Example test:

```rust
#[test]
fn test_my_mod() {
    let mut runtime = WasmRuntime::new(RuntimeConfig::default()).unwrap();
    let wasm_bytes = include_bytes!("../fixtures/my_mod/mod.wasm");

    runtime.load_mod("my-mod", wasm_bytes, Capability::WORLD_READ).unwrap();

    let event = GameEvent::player_chat(1, "Player1", "hello", 100);
    let cancelled = runtime.dispatch_game_event(0x05, &payload, event).unwrap();

    assert!(!cancelled);
}
```

## Related Documentation

- [ABI v1 Specification](../specs/035-sandboxed-mod-runtime/contracts/abi-v1.md)
- [Quickstart Guide](../specs/035-sandboxed-mod-runtime/quickstart.md)
- [Feature 034: Mod API Core](./feature-034.md)
