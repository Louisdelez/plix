# Plix Mod Troubleshooting Guide

Common issues and their solutions when developing Plix mods.

## Build Errors

### "error: could not compile to WASM"

**Problem**: Rust can't compile your mod to WebAssembly.

**Solution**: Ensure you have the WASM target installed:
```bash
rustup target add wasm32-unknown-unknown
```

### "undefined symbol: std::..."

**Problem**: Using `std` library features in `#![no_std]` environment.

**Solution**:
1. Ensure `#![no_std]` is at the top of lib.rs
2. Use `extern crate alloc;` for heap allocations
3. Use `alloc::string::String` instead of `std::string::String`

```rust
#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
```

### "unresolved import plix_mod_sdk"

**Problem**: SDK crate not found.

**Solution**: Check your Cargo.toml:
```toml
[dependencies]
plix-mod-sdk = "0.1"
```

## Validation Errors

### E001: Invalid mod ID

**Problem**: Mod ID doesn't follow naming rules.

**Rules**:
- 3-64 characters
- Lowercase letters, digits, hyphens only
- Can't start/end with hyphen
- No consecutive hyphens

**Examples**:
```toml
# Good
id = "my-mod"
id = "chat-filter-v2"
id = "mod123"

# Bad
id = "My-Mod"      # uppercase
id = "-my-mod"     # starts with hyphen
id = "my--mod"     # consecutive hyphens
id = "ab"          # too short
```

### E003: Missing WASM exports

**Problem**: Bundle is missing required exports.

**Required exports**:
- `mod_init`
- `mod_on_event`
- `mod_shutdown`

**Solution**: Ensure you're using `#[plix_mod]` correctly:

```rust
#[plix_mod]
struct MyMod;

#[plix_mod]  // <-- This generates the exports
impl MyMod {
    fn init(&self) { }
    fn shutdown(&self) { }
}
```

### E004: Manifest error

**Problem**: mod.toml is missing or invalid.

**Required fields**:
```toml
id = "my-mod"
name = "My Mod"
version = "0.1.0"
api_version = 1
```

### E005: Bundle too large

**Problem**: .plixmod exceeds 10 MB limit.

**Solutions**:
1. Optimize WASM size with release build:
   ```toml
   [profile.release]
   opt-level = "s"
   lto = true
   ```
2. Reduce asset sizes
3. Remove unused dependencies
4. Use `wasm-opt` for additional optimization

### E006: Unknown capability

**Problem**: Using an undefined capability.

**Valid capabilities**:
- `world_read`
- `world_write`
- `entity_read`
- `entity_write`
- `net_send`
- `event_cancel_chat`
- `event_cancel_blocks`

### E007: Incompatible API version

**Problem**: Mod requires newer SDK than server supports.

**Solution**: Update your server or target an older API version.

## Runtime Errors

### EMOD001: Invalid argument

**Cause**: Passing invalid parameters to SDK functions.

**Examples**:
- Negative coordinates
- Empty message strings
- Invalid entity IDs

### EMOD002: Permission denied

**Cause**: Calling a function without the required capability.

**Solution**: Add the capability to mod.toml:
```toml
[capabilities]
world_write = true
```

### EMOD003: Not found

**Cause**: Entity or resource doesn't exist.

**Example**:
```rust
// Entity may have despawned
match get_transform(EntityHandle(entity_id)) {
    Ok(t) => { /* use transform */ }
    Err(e) if e.code == ErrorCode::NotFound => {
        debug!("Entity no longer exists");
    }
    Err(e) => error!("Unexpected: {:?}", e),
}
```

### EMOD004: Out of bounds

**Cause**: Accessing blocks outside loaded world.

**Solution**: Check if position is valid before access:
```rust
let pos = IVec3::new(x, y, z);
if y >= 0 && y < 256 {
    match get_block(pos) {
        Ok(block) => { /* use block */ }
        Err(_) => { /* handle gracefully */ }
    }
}
```

### EMOD005: Rate limited

**Cause**: Too many API calls in short time.

**Solution**: Reduce call frequency, batch operations, cache results.

### EMOD006: World not ready

**Cause**: Accessing world before it's loaded.

**Solution**: Wait for `ServerStart` event:
```rust
#[on_event("on_server_start")]
fn handle_start(&self, _ctx: &EventContext, _payload: ServerStartPayload) {
    // World is now ready
    self.world_ready = true;
}
```

## Common Mistakes

### Forgetting to subscribe to events

```rust
fn init(&self) {
    // WRONG: Handler won't be called!
    // Need to subscribe first
}

fn init(&self) {
    // CORRECT
    subscribe(EventType::PlayerChat).unwrap();
}
```

### Using std features

```rust
// WRONG
use std::collections::HashMap;

// CORRECT
extern crate alloc;
use alloc::collections::BTreeMap;
```

### Not handling errors

```rust
// WRONG - will panic on error
let block = get_block(pos).unwrap();

// CORRECT - handle gracefully
match get_block(pos) {
    Ok(block) => { /* use it */ }
    Err(e) => { /* handle error */ }
}
```

### Missing both #[plix_mod] attributes

```rust
// WRONG - missing impl attribute
#[plix_mod]
struct MyMod;

impl MyMod { ... }  // No exports generated!

// CORRECT
#[plix_mod]
struct MyMod;

#[plix_mod]  // <-- Also needed here!
impl MyMod { ... }
```

## Debug Tips

### Enable debug logging

Build with debug profile for verbose logs:
```bash
plix-mod build  # debug build, more logs
```

### Check mod loading

Server logs show mod loading:
```
[INFO] Loading mod: my-mod v0.1.0
[INFO] Mod my-mod initialized successfully
```

### Validate before deploying

Always validate bundles:
```bash
plix-mod validate my-mod-0.1.0.plixmod --json
```

### Check exports

Verify WASM exports are present:
```bash
plix-mod validate my-mod-0.1.0.plixmod
# Should show: ✓ Required exports found
```

## Getting Help

1. Check this troubleshooting guide
2. Review the [SDK Reference](sdk.md)
3. Check server logs for error details
4. File an issue on GitHub
