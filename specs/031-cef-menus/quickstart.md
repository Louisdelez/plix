# Quickstart: CEF Menus

**Feature**: 031-cef-menus
**Date**: 2025-12-18

## Overview

This feature adds HTML/CSS/JS menus (main menu, settings, server browser) to the game client, rendered via the CEF shell (Feature 030). All communication between the UI and game engine uses a typed JSON message bridge.

## Prerequisites

- Feature 030 (CEF UI Shell) implemented and functional
- Feature 026 (Server Browser v1) for master server fetch
- Feature 025 (Account Identity) for display name
- Rust 1.75+ stable toolchain
- CEF binaries bundled with client

## Quick Test

1. Build and run the client:
   ```bash
   cargo build -p plix-client --features cef-ui
   ./target/debug/plix-client --cef-ui
   ```

2. Verify main menu appears with Play, Servers, Settings, Quit buttons

3. Navigate to Settings → modify sensitivity → Save → restart → verify persisted

4. Navigate to Servers → Refresh → select server → Connect

## Project Structure

```
crates/plix-client/src/ui_cef/
├── bridge/           # JS↔Rust message bridge
│   ├── mod.rs        # Bridge dispatcher
│   ├── messages.rs   # Message types (serde)
│   ├── handlers.rs   # Request handlers
│   └── serialize.rs  # JSON serialization
└── menus/            # Menu-specific handlers
    ├── mod.rs
    ├── config.rs     # GetConfig/SetConfig
    ├── servers.rs    # FetchServers/Connect
    └── favorites.rs  # Favorites persistence

assets/ui/
├── index.html        # App shell
├── app.js            # Router + state + bridge
├── styles.css        # Global styles
├── pages/            # Page modules
│   ├── main.js       # Main menu
│   ├── settings.js   # Settings page
│   └── servers.js    # Server browser
└── components/       # Reusable components
    ├── button.js     # Debounced button
    ├── modal.js      # Modal dialogs
    ├── list.js       # Server list
    └── keybind.js    # Keybind editor
```

## Key Concepts

### Message Bridge

All UI actions go through typed messages:

```javascript
// JS side
window.plix.send({ id: "req-1", type: "GetConfig", payload: {} });

// Register callback
window.plix.onMessage((msg) => {
  if (msg.id === "req-1") {
    // Handle response
  }
});
```

```rust
// Rust side
match message.msg_type.as_str() {
    "GetConfig" => handle_get_config(message.id),
    "SetConfig" => handle_set_config(message.id, message.payload),
    _ => send_error(message.id, "EBRG003", "Unknown message type"),
}
```

### Favorites Persistence

Favorites stored in `~/.config/plix/favorites.toml`:

```toml
version = 1
favorites = [
  "192.168.1.10:7777",
  "game.example.com:7777"
]
```

### Fallback Behavior

If CEF is disabled or fails:
1. `CefShell::should_fallback()` returns true
2. Game automatically uses native UI (Feature 005)
3. No code changes needed - handled by initialization logic

## Development Workflow

### Adding a New Message Type

1. Add type to `messages.rs`:
   ```rust
   pub enum MessageType {
       // existing...
       NewAction,
   }
   ```

2. Add handler in `handlers.rs`:
   ```rust
   pub fn handle_new_action(id: &str, payload: Value) -> BridgeResponse {
       // implementation
   }
   ```

3. Wire up in dispatcher:
   ```rust
   MessageType::NewAction => handle_new_action(&msg.id, msg.payload),
   ```

4. Use from JS:
   ```javascript
   plix.send({ id: genId(), type: "NewAction", payload: { ... } });
   ```

### Testing

- **Bridge tests**: `cargo test -p plix-client ui_cef::bridge`
- **Favorites tests**: `cargo test -p plix-client ui_cef::menus::favorites`
- **Manual UI tests**: Run client, exercise all screens

## Common Tasks

### Change settings exposed in UI

Edit `menus/config.rs` and update:
1. `GameConfigUI` struct fields
2. `to_ui_config()` conversion
3. `from_ui_config()` validation

### Add new server filter

1. Update `pages/servers.js` filter logic
2. Add UI control for filter
3. No Rust changes needed (filtering is client-side)

### Add new error code

1. Add to `messages.rs` error codes
2. Add to `contracts/bridge-messages.md`
3. Use in handler: `send_error(id, "ENEW001", "Description")`

## Debugging

### Enable CEF DevTools

```bash
./target/debug/plix-client --cef-ui --cef-devtools
```

Then connect Chrome DevTools to the debug port.

### View bridge messages

Add logging in `bridge/mod.rs`:
```rust
tracing::debug!(msg_type = %msg.msg_type, "Bridge message received");
```

### Check favorites file

```bash
cat ~/.config/plix/favorites.toml
```
