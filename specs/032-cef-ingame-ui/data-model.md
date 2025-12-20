# Data Model: CEF In-Game UI (HUD, Chat, Scoreboard)

**Feature**: 032-cef-ingame-ui
**Date**: 2025-12-18

## Entities

### HudState

Real-time HUD data pushed to UI at throttled rate.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| hp | u8 | Current health points | 0-100 |
| max_hp | u8 | Maximum health points | 100 (fixed) |
| rtt_ms | u32 | Round-trip time in milliseconds | 0-65535 |
| fps | Option<u32> | Frames per second (optional) | 0-999 |
| crosshair_visible | bool | Whether crosshair should render | - |

**Validation Rules**:
- `hp` must not exceed `max_hp`
- `rtt_ms` capped at display max (9999ms shown as "9999+")

**State Transitions**: None (stateless snapshot)

### ChatMessage

A single chat message in the history.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| id | u64 | Unique message ID (local) | Auto-increment |
| author | String | Display name of sender | Max 32 chars |
| text | String | Message content | Max 200 chars |
| kind | ChatMessageKind | Message type | Enum |
| timestamp | u64 | Unix timestamp (seconds) | - |

**Validation Rules**:
- `text` must be 1-200 characters after trimming
- `author` must be 1-32 characters
- `text` must not contain control characters (except newline for display)

**State Transitions**: Created → (persists in history) → Evicted (when history exceeds 100)

### ChatMessageKind

Enumeration of message types.

| Variant | Description |
|---------|-------------|
| Player | Normal player chat message |
| System | Server announcement, join/leave notification |

### ChatHistory

Local chat history state.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| messages | VecDeque<ChatMessage> | Ordered message list | Max 100 entries |
| next_id | u64 | Next local message ID | Auto-increment |

**Validation Rules**:
- When `messages.len() >= 100`, oldest is removed before adding new

### ScoreboardRow

A single player entry in the scoreboard.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| name | String | Player display name | Max 32 chars |
| ping_ms | u32 | Player ping | 0-9999 |
| score | Option<u16> | Total score (mode-dependent) | 0-65535 |
| kills | Option<u16> | Kill count | 0-65535 |
| deaths | Option<u16> | Death count | 0-65535 |
| team | Option<String> | Team identifier | "red", "blue", null |

**Validation Rules**:
- `name` must be non-empty
- All numeric values capped at display max

### ScoreboardState

Complete scoreboard snapshot.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| server_name | String | Server/match name | Max 64 chars |
| rows | Vec<ScoreboardRow> | Player entries | Max 64 entries |

**Validation Rules**:
- `rows` truncated to 64 if more players exist
- Sorted by score (descending) or team grouping

### UiConfig

Configuration pushed to UI on startup.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| cef_hud_enabled | bool | Whether web HUD is active | - |
| cef_crosshair_enabled | bool | Whether web crosshair is active | - |
| keybinds | KeybindMap | Key mapping for UI reference | - |

### KeybindMap

Subset of keybinds relevant to UI.

| Field | Type | Description |
|-------|------|-------------|
| chat_open | String | Key to open chat (default: "Enter") |
| scoreboard | String | Key to show scoreboard (default: "Tab") |

### InputFocusState

Extended input focus state machine.

| Variant | Description | Gameplay Input | UI Input |
|---------|-------------|----------------|----------|
| Game | Normal gameplay | ✅ Active | ❌ Blocked |
| CefUI | Menu UI active | ❌ Blocked | ✅ Active |
| ChatTyping | Chat input active | ❌ Blocked | ✅ Chat only |

**State Transitions**:

```
Game ──Enter──> ChatTyping ──Enter/Escape──> Game
Game ──ESC──> CefUI (menu) ──ESC──> Game
ChatTyping ──click outside──> Game
```

### ChatClient

Client-side chat state manager.

| Field | Type | Description |
|-------|------|-------------|
| history | ChatHistory | Local message history |
| is_open | bool | Whether chat input is open |
| pending_text | String | Current input text |
| last_send_time | Option<Instant> | Rate limit tracking |
| rate_limit_ms | u64 | Minimum interval between sends (500) |

**Operations**:
- `send(text)` → validates, rate-limits, sends to server
- `receive(message)` → adds to history, triggers toast if closed
- `clear()` → empties local history
- `handle_command(text)` → processes /help, /clear, forwards others

### ScoreboardClient

Client-side scoreboard state manager.

| Field | Type | Description |
|-------|------|-------------|
| visible | bool | Whether scoreboard is showing |
| cached_state | Option<ScoreboardState> | Latest scoreboard data |
| last_update | Option<Instant> | When data was last refreshed |

**Operations**:
- `show()` → set visible, trigger data fetch
- `hide()` → set not visible
- `update_from_snapshot(snapshot)` → refresh cached_state (only if visible)

### HudStatePublisher

Throttled HUD state publisher.

| Field | Type | Description |
|-------|------|-------------|
| last_publish | Instant | Time of last publish |
| last_state | HudState | Previously published state |
| min_interval_ms | u64 | Minimum publish interval (66ms = 15 Hz) |

**Operations**:
- `maybe_publish(current)` → returns Some(state) if should publish, None otherwise
- Publishes immediately on HP change (damage feedback)
- Publishes on interval for RTT/FPS updates

## Relationships

```
┌─────────────────┐     publishes     ┌──────────────┐
│ HudStatePublisher│──────────────────>│   HudState   │
└─────────────────┘                    └──────────────┘
                                              │
                                              v (bridge push)
                                       ┌──────────────┐
                                       │   CEF UI     │
                                       └──────────────┘
                                              ^
                                              │ (bridge push)
┌─────────────────┐     manages        ┌──────────────┐
│   ChatClient    │───────────────────>│ ChatHistory  │
└─────────────────┘                    └──────────────┘
        │                                     │
        │ sends                               │ contains
        v                                     v
┌─────────────────┐                    ┌──────────────┐
│  Server (UDP)   │                    │ ChatMessage  │
└─────────────────┘                    └──────────────┘

┌─────────────────┐     caches         ┌──────────────┐
│ScoreboardClient │───────────────────>│ScoreboardState│
└─────────────────┘                    └──────────────┘
        ^                                     │
        │ derives from                        │ contains
        │                                     v
┌─────────────────┐                    ┌──────────────┐
│ WorldSnapshot   │                    │ScoreboardRow │
└─────────────────┘                    └──────────────┘
```

## Protocol Extensions

### ClientMessage Extensions

```rust
pub enum ClientMessage {
    // ... existing variants ...

    /// Send a chat message to the server
    ChatSend {
        /// Message text (max 200 chars, validated server-side)
        text: String,
    },
}
```

### GameEvent Extensions

```rust
pub enum GameEvent {
    // ... existing variants ...

    /// Chat message received (broadcast to all)
    ChatReceived {
        /// Sender's player ID
        sender_id: PlayerId,
        /// Sender's display name
        sender_name: String,
        /// Message content
        text: String,
        /// Message kind (Player or System)
        kind: ChatMessageKind,
        /// Server timestamp
        timestamp: u64,
    },
}
```

## Persistence

**None** - All data is in-memory:
- Chat history resets on disconnect
- Scoreboard derived from live snapshots
- HUD state is ephemeral

Configuration (cef_hud_enabled, keybinds) persists via existing config system (Feature 005).
