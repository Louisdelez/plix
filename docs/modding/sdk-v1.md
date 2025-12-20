# Plix Mod SDK v1.0 Reference

This document provides the complete API reference for the Plix Mod SDK v1.0.

## Overview

The Plix Mod SDK enables creating mods that extend and modify game behavior. Mods run in a sandboxed WebAssembly environment with controlled access to game systems.

### SDK Version

```rust
// Current SDK version
const MOD_API_VERSION: (u8, u8) = (1, 0);
```

Mods built for API v1.x are compatible with any Plix v1.x engine.

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- `wasm32-unknown-unknown` target
- Plix SDK crate

### Installation

```bash
# Add the Plix mod SDK to your project
cargo add plix-mod-sdk

# Ensure WASM target is installed
rustup target add wasm32-unknown-unknown
```

### Minimal Mod

```rust
use plix_mod_sdk::prelude::*;

// Mod entry point
#[plix_mod]
pub struct HelloMod;

impl PlixMod for HelloMod {
    fn on_load(&mut self, ctx: &mut ModContext) {
        ctx.log_info("Hello from my mod!");
    }
}
```

## Mod Manifest

Every mod requires a `plix-mod.toml` manifest:

```toml
[mod]
id = "my-mod"
name = "My Awesome Mod"
version = "1.0.0"
api_version = "1.0"
description = "Does awesome things"
authors = ["Your Name <you@example.com>"]

[capabilities]
# Request specific capabilities
player_events = true
chat_events = false
block_events = false
timer = true
storage = false
```

## Capabilities

Mods must declare capabilities they need. The engine validates these at load time.

### Available Capabilities

| Capability | Description | Stability |
|------------|-------------|-----------|
| `player_events` | Receive player join/leave/death events | Stable |
| `chat_events` | Receive and modify chat messages | Stable |
| `block_events` | Receive block place/break events | Stable |
| `timer` | Create timed callbacks | Stable |
| `storage` | Persistent key-value storage | Stable |
| `http` | Make HTTP requests (restricted) | Experimental |
| `world_edit` | Modify blocks programmatically | Experimental |

### Capability Request

```rust
impl PlixMod for MyMod {
    fn capabilities() -> Capabilities {
        Capabilities::new()
            .with_player_events()
            .with_timer()
    }
}
```

## Events

### Player Events

```rust
impl PlixMod for MyMod {
    fn on_player_join(&mut self, ctx: &mut ModContext, event: PlayerJoinEvent) {
        ctx.log_info(&format!("Welcome, {}!", event.player_name));
    }

    fn on_player_leave(&mut self, ctx: &mut ModContext, event: PlayerLeaveEvent) {
        ctx.log_info(&format!("Goodbye, {}!", event.player_name));
    }

    fn on_player_death(&mut self, ctx: &mut ModContext, event: PlayerDeathEvent) {
        if let Some(killer) = event.killer_id {
            // Handle PvP death
        }
    }
}
```

### Chat Events

```rust
impl PlixMod for MyMod {
    fn on_chat(&mut self, ctx: &mut ModContext, event: ChatEvent) -> ChatResult {
        if event.message.starts_with("/mycommand") {
            ctx.send_message(event.player_id, "Command received!");
            return ChatResult::Consumed; // Don't broadcast
        }
        ChatResult::Pass // Normal processing
    }
}
```

### Block Events

```rust
impl PlixMod for MyMod {
    fn on_block_place(&mut self, ctx: &mut ModContext, event: BlockPlaceEvent) -> BlockResult {
        if event.position.y > 100 {
            ctx.send_message(event.player_id, "Cannot build above height 100!");
            return BlockResult::Cancel;
        }
        BlockResult::Allow
    }

    fn on_block_break(&mut self, ctx: &mut ModContext, event: BlockBreakEvent) -> BlockResult {
        // Allow all block breaks
        BlockResult::Allow
    }
}
```

### Timer Events

```rust
impl PlixMod for MyMod {
    fn on_load(&mut self, ctx: &mut ModContext) {
        // Call every 60 ticks (1 second at 60 TPS)
        ctx.set_timer("heartbeat", 60, true); // repeating
    }

    fn on_timer(&mut self, ctx: &mut ModContext, timer_id: &str) {
        if timer_id == "heartbeat" {
            ctx.log_debug("Heartbeat tick");
        }
    }
}
```

## Context API

The `ModContext` provides access to game systems.

### Logging

```rust
ctx.log_trace("Very detailed message");
ctx.log_debug("Debug information");
ctx.log_info("Normal information");
ctx.log_warn("Warning message");
ctx.log_error("Error message");
```

### Player Queries

```rust
// Get all online player IDs
let players: Vec<PlayerId> = ctx.get_players();

// Get player info
if let Some(info) = ctx.get_player_info(player_id) {
    println!("Name: {}", info.name);
    println!("Position: {:?}", info.position);
    println!("Health: {}", info.health);
}

// Get player count
let count = ctx.get_player_count();
```

### Messaging

```rust
// Send to specific player
ctx.send_message(player_id, "Hello!");

// Send to all players
ctx.broadcast("Server message");

// Send with color (using format codes)
ctx.send_message(player_id, "&cThis is red text");
```

### Storage (Persistent)

```rust
// Store a value
ctx.storage_set("key", "value")?;

// Retrieve a value
if let Some(value) = ctx.storage_get("key")? {
    ctx.log_info(&format!("Value: {}", value));
}

// Delete a value
ctx.storage_delete("key")?;
```

## Data Types

### PlayerId

```rust
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u16);
```

### Vec3

```rust
#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
```

### BlockPos

```rust
#[derive(Clone, Copy)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
```

### BlockType

```rust
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Dirt,
    Wood,
    // ... more types
}
```

## Error Handling

```rust
impl PlixMod for MyMod {
    fn on_load(&mut self, ctx: &mut ModContext) {
        match ctx.storage_get("config") {
            Ok(Some(config)) => self.load_config(&config),
            Ok(None) => self.use_defaults(),
            Err(e) => ctx.log_error(&format!("Storage error: {}", e)),
        }
    }
}
```

## Resource Limits

Mods operate within resource constraints:

| Resource | Limit | Notes |
|----------|-------|-------|
| Memory | 16 MB | Per-mod WASM heap |
| CPU | 10ms/tick | Per-tick execution time |
| Storage | 1 MB | Per-mod persistent storage |
| Timers | 16 | Maximum concurrent timers |
| HTTP | Rate limited | If capability enabled |

Exceeding limits may cause the mod to be unloaded.

## Stability Markers

### Stable API

Functions marked as stable are guaranteed for the v1.x lifecycle:

- `ModContext::log_*`
- `ModContext::get_players`
- `ModContext::get_player_info`
- `ModContext::send_message`
- `ModContext::broadcast`
- `ModContext::set_timer`
- `ModContext::storage_*`
- All player event handlers
- All chat event handlers
- All block event handlers
- All timer event handlers

### Experimental API

May change in minor versions (use at your own risk):

- `ModContext::http_*`
- `ModContext::set_block`
- `ModContext::get_block`
- World edit capabilities

### Deprecated API

Will be removed in v2.0:

None currently.

## Building Mods

```bash
# Build for WASM
cargo build --target wasm32-unknown-unknown --release

# The output is in:
# target/wasm32-unknown-unknown/release/your_mod.wasm
```

### Optimization

```bash
# Install wasm-opt
cargo install wasm-opt

# Optimize the WASM binary
wasm-opt -O3 -o your_mod.wasm target/wasm32-unknown-unknown/release/your_mod.wasm
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use plix_mod_sdk::testing::*;

    #[test]
    fn test_on_player_join() {
        let mut mod_instance = MyMod::new();
        let mut ctx = MockModContext::new();

        let event = PlayerJoinEvent {
            player_id: PlayerId(1),
            player_name: "TestPlayer".to_string(),
        };

        mod_instance.on_player_join(&mut ctx, event);

        assert!(ctx.logs().contains(&"Welcome, TestPlayer!"));
    }
}
```

## Examples

See the `examples/` directory in the SDK repository for complete example mods:

- `hello-world` - Minimal mod template
- `chat-commands` - Custom chat commands
- `pvp-stats` - Player kill/death tracking
- `build-protection` - Block protection zones

## Related Documents

- [Modding Overview](overview.md)
- [Getting Started Tutorial](getting-started.md)
- [Stability Policy](stability.md)
- [Publishing Mods](publishing.md)
- [Compatibility Guide](compatibility.md)
