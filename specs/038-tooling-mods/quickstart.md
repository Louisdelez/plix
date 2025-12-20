# Quickstart: Creating Your First Plix Mod

This guide walks you through creating, building, packaging, and running your first Plix mod in under 5 minutes.

## Prerequisites

1. **Rust toolchain** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **WASM target**
   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. **Plix mod CLI** (installed with Plix)
   ```bash
   # Verify installation
   plix mod --version
   ```

## Step 1: Create a New Mod (30 seconds)

Create a mod from the `chat-filter` template:

```bash
plix mod new my-chat-filter --template chat-filter
cd my-chat-filter
```

This creates:
```
my-chat-filter/
├── Cargo.toml          # Rust project config
├── mod.toml            # Mod manifest
├── src/lib.rs          # Mod source code
└── build.sh            # Build helper script
```

## Step 2: Explore the Code (1 minute)

Open `src/lib.rs`:

```rust
use plix_mod_sdk::prelude::*;

#[plix_mod]
struct MyChatFilter;

#[plix_mod]
impl MyChatFilter {
    fn init(&self) {
        info!("Chat filter mod initialized!");
        subscribe(EventType::PlayerChat).unwrap();
    }

    fn shutdown(&self) {
        info!("Chat filter mod shutting down");
    }

    #[on_event("on_player_chat")]
    fn handle_chat(&self, ctx: &EventContext, payload: PlayerChatPayload) {
        info!("Player {} said: {}", payload.player_id, payload.text);

        // Example: Cancel messages containing "badword"
        if payload.text.to_lowercase().contains("badword") {
            info!("Blocked message from player {}", payload.player_id);
            ctx.cancel().unwrap();
        }
    }
}
```

Open `mod.toml`:

```toml
id = "my-chat-filter"
name = "My Chat Filter"
version = "1.0.0"
api_version = 1

[capabilities]
event_cancel_chat = true
```

## Step 3: Build the Mod (1 minute)

Compile to WASM:

```bash
plix mod build
```

Output:
```
   Compiling my-chat-filter v1.0.0
    Finished release [optimized] target(s) in 2.34s
    Built: target/wasm32-unknown-unknown/release/my_chat_filter.wasm
```

## Step 4: Package the Mod (30 seconds)

Create a `.plixmod` bundle:

```bash
plix mod pack
```

Output:
```
Packing my-chat-filter v1.0.0...
  Adding mod.toml
  Adding mod.wasm (45.2 KB)

Packed: my-chat-filter-1.0.0.plixmod (46.8 KB)
SHA-256: a1b2c3d4e5f6...
```

## Step 5: Validate the Bundle (15 seconds)

Verify the bundle is correct:

```bash
plix mod validate my-chat-filter-1.0.0.plixmod
```

Output:
```
Validating my-chat-filter-1.0.0.plixmod...
✓ Manifest valid
✓ Required fields present
✓ WASM binary present
✓ Required exports found (mod_init, mod_on_event, mod_shutdown)
✓ Size OK (46.8 KB / 10 MB)
✓ API version compatible (1)
✓ Capabilities valid

Result: PASS
```

## Step 6: Install and Run (1 minute)

### Option A: Copy to server mods folder

```bash
cp my-chat-filter-1.0.0.plixmod /path/to/server/mods/
```

### Option B: Install to local cache

```bash
plix mod install my-chat-filter-1.0.0.plixmod --local
```

### Start the server

```bash
plix-server --mods-dir ./mods
```

Look for in the logs:
```
[INFO] Loading mod: my-chat-filter v1.0.0
[INFO] [my-chat-filter] Chat filter mod initialized!
```

## Step 7: Test Your Mod

1. Connect to the server with a client
2. Send a chat message - see it logged
3. Send a message containing "badword" - see it blocked

## Next Steps

- **More templates**: Try `world-query` or `timers-net`
- **SDK reference**: See `docs/modding/sdk.md` for all APIs
- **Distribution**: See `docs/modding/distribution.md` for publishing
- **Troubleshooting**: See `docs/modding/troubleshooting.md` for common issues

## Quick Reference

| Command | Description |
|---------|-------------|
| `plix mod new <name> -t <template>` | Create from template |
| `plix mod build` | Compile to WASM |
| `plix mod pack` | Create .plixmod bundle |
| `plix mod validate <bundle>` | Check bundle validity |
| `plix mod install <bundle>` | Install to cache |

| Template | Use Case |
|----------|----------|
| `chat-filter` | Event handling, cancellation |
| `world-query` | Block reading, raycasts |
| `timers-net` | Timers, network messages |

## Troubleshooting

### "Missing wasm32 target"
```bash
rustup target add wasm32-unknown-unknown
```

### "WASM exports not found"
Make sure you have `#[plix_mod]` on both the struct and impl block.

### "Size exceeded"
Bundles must be ≤ 10 MB. Optimize assets or split into multiple mods.

### "API version mismatch"
Update your SDK: `cargo update plix-mod-sdk`
