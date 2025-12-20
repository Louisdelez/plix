# Quickstart: CEF In-Game UI Development

**Feature**: 032-cef-ingame-ui
**Date**: 2025-12-18

## Prerequisites

- Rust 1.75+ (stable)
- Cargo with workspace support
- Node.js (for UI development, optional)
- Git

## Setup

### 1. Clone and Build

```bash
cd /home/louis/Documents/plix
git checkout 032-cef-ingame-ui

# Build all crates
cargo build

# Run tests
cargo test
```

### 2. Feature Flag

The CEF UI is behind a feature flag. To enable:

```bash
cargo build --features cef-ui
```

Without the flag, native fallback UI is always used.

### 3. Development Configuration

Create or update `~/.config/plix/config.toml`:

```toml
[ui]
# Enable CEF HUD (requires cef-ui feature)
cef_hud = true
# Debug bridge messages
debug_bridge = true

[keybinds]
chat_open = "Return"  # Enter key
scoreboard = "Tab"
```

## Running

### Start Server (Terminal 1)

```bash
cargo run -p plix-server -- --arena default --mode ffa
```

### Start Client (Terminal 2)

```bash
# With CEF UI
cargo run -p plix-client --features cef-ui

# With native fallback only
cargo run -p plix-client
```

## Development Workflow

### Modifying Rust Code

Key files for this feature:

```
crates/plix-client/src/
├── ui_cef/
│   ├── ingame/        # In-game overlay (HUD, chat, scoreboard)
│   │   ├── mod.rs
│   │   ├── hud.rs
│   │   ├── chat.rs
│   │   └── scoreboard.rs
│   └── bridge/
│       ├── messages.rs  # Add new message types here
│       └── handlers.rs  # Add new handlers here
├── ui/
│   ├── chat_native.rs   # Native chat fallback
│   └── scoreboard_native.rs
└── input.rs             # InputFocus state machine

crates/plix-common/src/
├── protocol/messages.rs # ChatSend, ChatReceived protocol messages
└── chat.rs              # Chat types
```

### Modifying UI (HTML/CSS/JS)

UI assets are in `assets/ui/ingame/`:

```
assets/ui/ingame/
├── overlay.html   # Main overlay page
├── overlay.css    # Styles
└── overlay.js     # Bridge integration
```

Changes to UI files require reloading:
- Press F5 in-game (if CEF dev mode enabled)
- Or restart the client

### Testing

```bash
# Run all tests
cargo test

# Run specific test modules
cargo test chat_tests
cargo test input_focus
cargo test bridge_ingame

# Run with output
cargo test -- --nocapture
```

## Debug Tools

### Bridge Message Logging

Enable in config:
```toml
[ui]
debug_bridge = true
```

This logs all bridge messages to stdout:
```
[DEBUG] Bridge TX: {"type":"HudState","payload":{...}}
[DEBUG] Bridge RX: {"type":"ChatSend","payload":{...}}
```

### In-Game Debug Overlay

Press F3 to toggle debug overlay showing:
- Current input focus state
- Last bridge message type
- HUD update frequency
- Chat history count

## Common Tasks

### Adding a New Bridge Message

1. Add to `MessageType` enum in `ui_cef/bridge/messages.rs`:
```rust
pub enum MessageType {
    // ... existing ...
    MyNewMessage,
}
```

2. Add parse/as_str implementations:
```rust
impl MessageType {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            // ... existing ...
            "MyNewMessage" => Some(Self::MyNewMessage),
            _ => None,
        }
    }
}
```

3. Add handler in `ui_cef/bridge/handlers.rs`:
```rust
pub fn handle_my_new_message(id: &str, payload: &Value) -> BridgeResponse {
    // ... implementation ...
}
```

4. Add to JS in `overlay.js`:
```javascript
window.plix.handlers.MyNewMessage = (payload) => {
    // ... implementation ...
};
```

### Testing Chat Locally

1. Start server
2. Start two clients (or use `--name` flag for different names)
3. Press Enter to open chat
4. Type message, press Enter to send
5. Message appears in both clients

### Testing Scoreboard

1. Join a match
2. Hold Tab to show scoreboard
3. Release Tab to hide
4. Verify data updates as players join/leave

## Troubleshooting

### CEF Not Loading

1. Check feature flag: `cargo build --features cef-ui`
2. Check logs for CEF init errors
3. Verify `config.toml` has `cef_hud = true`

### Chat Messages Not Sending

1. Check server console for received messages
2. Verify rate limit (500ms between messages)
3. Check message length (<= 200 chars)

### Scoreboard Empty

1. Verify in multiplayer match (single player has only local player)
2. Check server is sending player snapshots
3. Verify scoreboard visibility state in debug overlay

### Input Stuck in Chat

1. Press Escape to force close
2. Check input focus state in debug overlay
3. Look for errors in bridge message logs

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                      Game Loop                          │
│  ┌─────────┐  ┌─────────┐  ┌─────────────┐            │
│  │  Input  │─>│  Game   │─>│   Render    │            │
│  └────┬────┘  └────┬────┘  └──────┬──────┘            │
│       │            │               │                   │
│       v            v               v                   │
│  ┌─────────────────────────────────────────────┐      │
│  │              Input Focus Controller          │      │
│  │  [Game] <──> [ChatTyping] <──> [CefUI]      │      │
│  └────────────────────┬────────────────────────┘      │
│                       │                                │
│                       v                                │
│  ┌─────────────────────────────────────────────┐      │
│  │           CEF In-Game Overlay               │      │
│  │  ┌───────┐  ┌────────┐  ┌──────────────┐   │      │
│  │  │  HUD  │  │  Chat  │  │  Scoreboard  │   │      │
│  │  └───┬───┘  └────┬───┘  └──────┬───────┘   │      │
│  │      │           │             │            │      │
│  │      └───────────┼─────────────┘            │      │
│  │                  v                          │      │
│  │         Bridge Dispatcher                   │      │
│  └──────────────────┬──────────────────────────┘      │
│                     │                                  │
│                     v                                  │
│            ┌────────────────┐                         │
│            │   CEF Shell    │                         │
│            │  (HTML/JS/CSS) │                         │
│            └────────────────┘                         │
└─────────────────────────────────────────────────────────┘
```
