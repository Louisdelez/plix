# Quickstart: Server Mods + Client Sync

**Feature**: 037-server-mods
**Date**: 2025-12-19

## Overview

This feature enables server-only mod execution with optional client data synchronization. Players can join modded servers without installing mods locally.

## For Server Administrators

### Basic Setup (Server-Only Mods)

No additional configuration needed beyond Feature 036. Server-only mods work out of the box.

```toml
# server_mods.toml
[[mods]]
id = "game-rules"
version = "^1.0"

[[mods]]
id = "custom-weapons"
version = "^2.0"
```

Players connect normally. Mods execute on the server only.

### With Client Data Sync

If mods require client-side data (item definitions, UI strings):

```toml
# server_mods.toml
[[mods]]
id = "custom-items"
version = "^1.0"

# Enable sync (default is already true)
[join_policy]
allow_payload_sync = true

# Adjust limits if needed
[sync]
max_payload_mb = 25
```

When players connect, they receive mod data automatically before joining.

### Strict Mode (Require Sync Support)

To block clients that can't receive mod data:

```toml
[join_policy]
require_payload_sync = true
```

Older clients without sync capability will be refused with a clear message.

## For Mod Developers

### Creating a Server-Only Mod

```toml
# mod.toml
[mod]
id = "my-server-mod"
name = "My Server Mod"
version = "1.0.0"
api_version = 1
runtime = "server"  # Default, can omit
```

The mod runs only on the server. No client installation needed.

### Adding Client Data

If your mod needs to send data to clients (item definitions, config):

```toml
# mod.toml
[mod]
id = "custom-items"
name = "Custom Items"
version = "1.0.0"
api_version = 1
runtime = "server"
client_payload = true
client_payload_files = [
    "client/items.json",
    "client/config.toml",
]
```

Bundle structure:
```text
my-mod/
├── mod.toml
├── mod.wasm
└── client/
    ├── items.json
    └── config.toml
```

The `client/` folder contents are packaged and synced to players automatically.

### Using Mod Channels

To send real-time messages to connected players:

```rust
// In your WASM mod
fn on_player_action(player_id: PlayerId) {
    // Send data to client on your mod's channel
    send_mod_message(player_id, "scoreboard", &score_data);
}
```

The client receives messages on `mod:my-mod:scoreboard`.

To allow clients to send messages back:

```toml
# mod.toml
[mod.network]
allowed_client_channels = ["input", "request"]
```

Clients can then send on `mod:my-mod:input` and `mod:my-mod:request`.

## Protocol Summary

### Connection Flow

```text
1. Client connects to server
2. Server sends mod list (ModSetDescriptor)
3. Client responds with cache state (which payloads it already has)
4. Server decides: OK / Refused / SyncRequired
5. If sync required: server streams payload chunks
6. Client verifies SHA-256 hash
7. Join completes
```

### Cache Behavior

- Payloads are cached by SHA-256 hash
- Same content = same hash = no re-download
- Cache persists across sessions
- Cache location: `~/.local/share/plix/mods/payloads/`

## Testing

### Manual Testing

1. **Server-only join**:
   - Start server with `runtime = "server"` mod
   - Connect vanilla client
   - Verify: join succeeds, mod effects visible

2. **Payload sync**:
   - Start server with `client_payload = true` mod
   - Connect client without cache
   - Verify: payload downloads, join succeeds

3. **Cache hit**:
   - Disconnect and reconnect
   - Verify: no re-download, instant join

4. **Refusal**:
   - Set `require_payload_sync = true`
   - Connect old client without sync support
   - Verify: clear refusal message

### Integration Tests

```bash
cargo test -p plix-server mod_sync
cargo test -p plix-client payload_cache
```

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "Sync unsupported" | Old client | Update client to 0.37+ |
| Slow join | Large payload | Reduce `client_payload_files` |
| "Integrity failed" | Corrupted transfer | Reconnect (auto-retry) |
| "Payload too large" | Exceeds limit | Increase `max_payload_mb` or reduce payload |

## Performance Tips

1. **Keep payloads small**: Under 5MB for quick joins
2. **Use JSON/TOML for data**: Compresses well
3. **Avoid large assets**: Use server-side references instead
4. **Test on slow connections**: 1 Mbps should complete in <30s for 25MB
