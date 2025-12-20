# Plix Mod Development Quickstart

Get your first mod running in under 5 minutes!

## Prerequisites

- Rust 1.75+ with `wasm32-unknown-unknown` target
- `plix-mod` CLI tool

### Install the WASM target

```bash
rustup target add wasm32-unknown-unknown
```

### Install the CLI

```bash
cargo install plix-mod-cli
```

## Create Your First Mod

### 1. Create a new mod project

```bash
plix-mod new my-first-mod --template chat-filter
cd my-first-mod
```

This creates:
```
my-first-mod/
  Cargo.toml      # Rust project config
  mod.toml        # Mod manifest (id, version, capabilities)
  src/
    lib.rs        # Your mod code
```

### 2. Customize your mod

Edit `mod.toml`:
```toml
id = "my-first-mod"
name = "My First Mod"
version = "0.1.0"
api_version = 1
author = "Your Name"
description = "My awesome Plix mod"

[capabilities]
event_cancel_chat = true
```

Edit `src/lib.rs`:
```rust
#![no_std]
extern crate alloc;

use plix_mod_sdk::prelude::*;
use plix_mod_sdk_macros::{plix_mod, on_event};

#[plix_mod]
struct MyFirstMod;

#[plix_mod]
impl MyFirstMod {
    fn init(&self) {
        info!("My first mod initialized!");
        subscribe(EventType::PlayerChat).unwrap();
    }

    fn shutdown(&self) {
        info!("Goodbye!");
    }

    #[on_event("on_player_chat")]
    fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
        if payload.text.contains("hello") {
            info!("Someone said hello!");
        }
    }
}
```

### 3. Build the mod

```bash
plix-mod build --release
```

This compiles to `target/wasm32-unknown-unknown/release/my_first_mod.wasm`

### 4. Pack the mod

```bash
plix-mod pack
```

Creates `my-first-mod-0.1.0.plixmod` - a distributable bundle.

### 5. Validate the bundle

```bash
plix-mod validate my-first-mod-0.1.0.plixmod
```

### 6. Install locally

```bash
plix-mod install my-first-mod-0.1.0.plixmod
```

### 7. Load in server

Add to your server's `mods.toml`:
```toml
[[mods]]
id = "my-first-mod"
version = "0.1.0"
```

## Available Templates

- **chat-filter** - Filter chat messages (demonstrates event cancellation)
- **world-query** - Read blocks and raycast (demonstrates world API)
- **timers-net** - Timers and networking (demonstrates timers and messaging)

```bash
plix-mod new my-mod --template world-query
```

## Next Steps

- [SDK Reference](sdk.md) - Complete API documentation
- [Capabilities Guide](capabilities.md) - Understanding permissions
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Quick Reference

### Event Types

| Event | Description |
|-------|-------------|
| `ServerStart` | Server is starting |
| `ServerStop` | Server is stopping |
| `PlayerJoin` | Player connected |
| `PlayerLeave` | Player disconnected |
| `PlayerChat` | Chat message sent |
| `BlockPlaced` | Block was placed |
| `BlockBroken` | Block was broken |
| `EntitySpawned` | Entity created |
| `EntityDespawned` | Entity removed |

### Capabilities

| Capability | Description |
|------------|-------------|
| `world_read` | Read blocks |
| `world_write` | Modify blocks |
| `entity_read` | Query entities |
| `entity_write` | Damage/push entities |
| `net_send` | Send messages |
| `event_cancel_chat` | Cancel chat events |
| `event_cancel_blocks` | Cancel block events |

### CLI Commands

```bash
plix-mod new <name> [--template <template>]  # Create new mod
plix-mod build [--release]                    # Compile to WASM
plix-mod pack [-o <file>]                     # Create .plixmod bundle
plix-mod validate <bundle>                    # Validate bundle
plix-mod install <bundle> [--force]           # Install to cache
```
