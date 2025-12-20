# Research: Tooling Mods (SDK, Templates, CLI, Hot-Reload)

**Feature**: 038-tooling-mods
**Date**: 2025-12-19
**Status**: Complete

## R-001: Proc-Macro Pattern for WASM Exports

### Decision
Use a `#[plix_mod]` attribute macro on the mod's main struct/impl to generate required WASM exports, plus `#[on_event("event_name")]` for event handlers.

### Rationale
- Proc-macros can generate `#[no_mangle] pub extern "C" fn` declarations
- Event routing table can be generated at compile time via a static dispatch match
- Pattern aligns with wasmtime/wasmer plugin ecosystems (extism, wasm-bindgen)

### Implementation Pattern
```rust
// User writes:
#[plix_mod]
struct MyMod;

#[plix_mod]
impl MyMod {
    #[on_event("on_player_chat")]
    fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
        // handler code
    }
}

// Macro generates:
#[no_mangle]
pub extern "C" fn mod_init() -> i32 { ... }

#[no_mangle]
pub extern "C" fn mod_on_event(event_type: i32, payload_ptr: i32, payload_len: i32) -> i32 {
    match event_type {
        0x05 => { /* dispatch to handle_chat */ }
        _ => 0
    }
}

#[no_mangle]
pub extern "C" fn mod_shutdown() -> i32 { ... }
```

### Alternatives Considered
1. **Manual exports**: Rejected - too error-prone for modders
2. **Runtime registration**: Rejected - adds complexity, not needed for WASM
3. **Inventory/linkme patterns**: Rejected - doesn't work reliably in WASM

---

## R-002: Deterministic ZIP Creation

### Decision
Use the `zip` crate with explicit configuration for determinism:
- Sort all file entries alphabetically by path
- Set all timestamps to Unix epoch (1980-01-01 for ZIP format)
- Use Deflate compression level 6 (default)
- Disable extra fields and comments

### Rationale
- The `zip` crate is mature and widely used
- Determinism is achievable with careful configuration
- ZIP format requires timestamps >= 1980, so epoch is 1980-01-01

### Implementation Pattern
```rust
use zip::{ZipWriter, write::FileOptions, CompressionMethod, DateTime};

let options = FileOptions::default()
    .compression_method(CompressionMethod::Deflated)
    .compression_level(Some(6))
    .last_modified_time(DateTime::from_date_and_time(1980, 1, 1, 0, 0, 0).unwrap());

let mut entries: Vec<_> = walkdir.collect();
entries.sort_by(|a, b| a.path().cmp(b.path()));

for entry in entries {
    zip.start_file(relative_path, options)?;
    // write contents
}
```

### Alternatives Considered
1. **tar.gz**: Rejected - ZIP is the established format for .plixmod bundles (Feature 036)
2. **No compression**: Rejected - would increase bundle sizes significantly
3. **Custom format**: Rejected - adds complexity, ZIP is standard

---

## R-003: Filesystem Watcher for Hot-Reload

### Decision
Use the `notify` crate (v6.x) with a custom debouncer implementation integrated with tokio.

### Rationale
- `notify` is the de-facto standard for cross-platform file watching in Rust
- Supports Linux (inotify), macOS (FSEvents), Windows (ReadDirectoryChangesW)
- Debounce can be implemented via tokio timer with 200ms default

### Implementation Pattern
```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use std::time::Duration;

let (tx, mut rx) = mpsc::channel(100);

let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
    if let Ok(event) = res {
        if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_)) {
            let _ = tx.blocking_send(event);
        }
    }
})?;

watcher.watch(Path::new("mods/"), RecursiveMode::Recursive)?;

// Debounce loop
let mut pending: HashMap<PathBuf, Instant> = HashMap::new();
loop {
    tokio::select! {
        Some(event) = rx.recv() => {
            for path in event.paths {
                pending.insert(path, Instant::now());
            }
        }
        _ = tokio::time::sleep(Duration::from_millis(50)) => {
            let now = Instant::now();
            for (path, time) in pending.drain() {
                if now.duration_since(time) >= Duration::from_millis(200) {
                    trigger_reload(&path).await;
                }
            }
        }
    }
}
```

### Alternatives Considered
1. **Polling**: Rejected - inefficient, high CPU usage
2. **notify-debouncer-full**: Considered but custom debounce gives more control
3. **inotify directly**: Rejected - not cross-platform

---

## R-004: Feature 034/035 ABI Surface

### Decision
SDK must wrap all host functions defined in plix-mod-runtime-wasm ABI v1.

### Complete Host Function List

| Function | Op Code | Category | Capability Required |
|----------|---------|----------|---------------------|
| `plix_log` | 0x01-0x0F | Logging | None |
| `plix_get_api_version` | 0x11 | Version | None |
| `plix_get_abi_version` | 0x12 | Version | None |
| `plix_has_capability` | 0x10 | Caps | None |
| `plix_subscribe_event` | 0x20 | Events | None |
| `plix_cancel_event` | 0x21 | Events | EVENT_CANCEL_* |
| `plix_response_ptr` | - | Utility | None |
| `plix_response_len` | - | Utility | None |
| `plix_world_call` (GetBlock) | 0x30 | World | WORLD_READ |
| `plix_world_call` (SetBlock) | 0x31 | World | WORLD_WRITE |
| `plix_world_call` (Raycast) | 0x32 | World | WORLD_READ |
| `plix_world_call` (QueryAabb) | 0x33 | World | WORLD_READ |
| `plix_entity_call` (GetTransform) | 0x40 | Entity | ENTITY_READ |
| `plix_entity_call` (GetHealth) | 0x41 | Entity | ENTITY_READ |
| `plix_entity_call` (ApplyDamage) | 0x42 | Entity | ENTITY_WRITE |
| `plix_entity_call` (ApplyImpulse) | 0x43 | Entity | ENTITY_WRITE |
| `plix_net_call` (Send) | 0x50 | Network | NET_SEND |
| `plix_net_call` (Broadcast) | 0x51 | Network | NET_SEND |
| `plix_timer_call` (SetTimeout) | 0x60 | Timers | None |
| `plix_timer_call` (SetInterval) | 0x61 | Timers | None |
| `plix_timer_call` (Clear) | 0x62 | Timers | None |

### Capability IDs (Bitmask)

| ID | Name | Hex |
|----|------|-----|
| WORLD_READ | Read world state | 0x01 |
| WORLD_WRITE | Modify world state | 0x02 |
| ENTITY_READ | Read entity state | 0x04 |
| ENTITY_WRITE | Modify entities | 0x08 |
| NET_SEND | Send network messages | 0x10 |
| EVENT_CANCEL_CHAT | Cancel chat events | 0x20 |
| EVENT_CANCEL_BLOCKS | Cancel block events | 0x40 |

### Error Codes (EMOD Series)

| Code | Name | Description |
|------|------|-------------|
| 1 | EMOD001 | InvalidArgument |
| 2 | EMOD002 | PermissionDenied |
| 3 | EMOD003 | NotFound |
| 4 | EMOD004 | OutOfBounds |
| 5 | EMOD005 | RateLimited |
| 6 | EMOD006 | WorldNotReady |
| 7 | EMOD007 | Unsupported |

### Event Types

| ID | Name | Cancellable |
|----|------|-------------|
| 0x01 | ServerStart | No |
| 0x02 | ServerStop | No |
| 0x03 | PlayerJoin | No |
| 0x04 | PlayerLeave | No |
| 0x05 | PlayerChat | Yes (EVENT_CANCEL_CHAT) |
| 0x06 | BlockPlaced | Yes (EVENT_CANCEL_BLOCKS) |
| 0x07 | BlockBroken | Yes (EVENT_CANCEL_BLOCKS) |
| 0x08 | EntitySpawned | No |
| 0x09 | EntityDespawned | No |

### Constraints

| Resource | Limit |
|----------|-------|
| Log message | 4096 bytes |
| Network payload | 8 KB |
| Network rate | 20 msg/s per mod |
| Timer min interval | 50 ms |
| Max timers per mod | 32 |
| Raycast max distance | 256 blocks |
| QueryAabb max results | 128 |

### Serialization
- Protocol: bincode (binary, little-endian)
- Math types: glam (Vec3, IVec3, Quat)
- Response buffer: 64 KB at offset 0x10000

---

## Summary

All research questions resolved. Key decisions:

1. **Proc-macros**: `#[plix_mod]` + `#[on_event]` pattern with static dispatch
2. **Deterministic ZIP**: `zip` crate with sorted entries, epoch timestamps, deflate-6
3. **File watcher**: `notify` crate with custom tokio-integrated debouncer
4. **ABI coverage**: 19 host functions across 6 categories, 7 capabilities, 7 error codes, 9 event types

Ready for Phase 1 design artifacts.
