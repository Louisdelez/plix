# Bridge Message Contracts: CEF In-Game UI

**Feature**: 032-cef-ingame-ui
**Bridge Version**: 1.0 (extends Feature 031)
**Date**: 2025-12-18

## Message Envelope

All messages follow the Feature 031 envelope format:

```json
{
  "id": "string | null",  // Request ID (null for push events)
  "type": "string",       // Message type identifier
  "payload": {}           // Type-specific payload
}
```

## UI → Game Messages (Requests)

### ChatSend

Send a chat message to the server for broadcast.

**Type**: `"ChatSend"`

**Payload**:
```json
{
  "text": "string"  // Message content (1-200 chars)
}
```

**Validation**:
- `text` required, non-empty after trim
- `text` max 200 characters
- Rate limited: 1 message per 500ms (client-side)

**Response**: Empty success or error

**Error Codes**:
- `ECHAT001`: Message too long
- `ECHAT002`: Rate limited (try again later)
- `ECHAT003`: Empty message

**Example**:
```json
// Request
{"id":"chat-1","type":"ChatSend","payload":{"text":"Hello everyone!"}}

// Response (success)
{"id":"chat-1","type":"ChatSend","ok":true,"payload":{}}

// Response (error)
{"id":"chat-2","type":"ChatSend","ok":false,"error":{"code":"ECHAT002","message":"Please wait before sending another message"}}
```

---

### ChatOpen

Notify the game that chat input was opened.

**Type**: `"ChatOpen"`

**Payload**: `{}`

**Side Effects**:
- Game transitions to `ChatTyping` input focus
- Gameplay input blocked until `ChatClose`

**Response**: Empty success

**Example**:
```json
{"id":"co-1","type":"ChatOpen","payload":{}}
```

---

### ChatClose

Notify the game that chat input was closed.

**Type**: `"ChatClose"`

**Payload**: `{}`

**Side Effects**:
- Game transitions back to `Game` input focus
- Gameplay input restored

**Response**: Empty success

**Example**:
```json
{"id":"cc-1","type":"ChatClose","payload":{}}
```

---

### ChatClear

Clear the local chat history.

**Type**: `"ChatClear"`

**Payload**: `{}`

**Side Effects**:
- Local message history emptied
- Does not affect server or other clients

**Response**: Empty success

**Example**:
```json
{"id":"clr-1","type":"ChatClear","payload":{}}
```

---

## Game → UI Messages (Push Events)

### HudState

Push HUD data to UI (throttled to 10-20 Hz).

**Type**: `"HudState"`

**Payload**:
```json
{
  "hp": 100,           // Current health (0-100)
  "max_hp": 100,       // Maximum health
  "rtt_ms": 45,        // Round-trip time in ms
  "fps": 144           // Optional FPS (null if disabled)
}
```

**Push Frequency**:
- Every ~66ms (15 Hz baseline)
- Immediately on HP change

**Example**:
```json
{"id":null,"type":"HudState","payload":{"hp":75,"max_hp":100,"rtt_ms":32,"fps":60}}
```

---

### ChatMessage

Push received chat message to UI for display.

**Type**: `"ChatMessage"`

**Payload**:
```json
{
  "author": "string",     // Sender display name
  "text": "string",       // Message content
  "kind": "player|system",// Message type
  "timestamp": 1702912345 // Unix timestamp (seconds)
}
```

**Kind Values**:
- `"player"`: Normal player message
- `"system"`: Server announcement, join/leave

**Example**:
```json
{"id":null,"type":"ChatMessage","payload":{"author":"Alice","text":"Good game!","kind":"player","timestamp":1702912345}}
{"id":null,"type":"ChatMessage","payload":{"author":"Server","text":"Bob has joined the match","kind":"system","timestamp":1702912346}}
```

---

### ChatToast

Push toast notification when chat is closed.

**Type**: `"ChatToast"`

**Payload**:
```json
{
  "author": "string",  // Sender display name
  "text": "string"     // Message preview (truncated if needed)
}
```

**UI Behavior**:
- Show non-intrusive toast notification
- Auto-dismiss after 3 seconds
- Click-through (does not capture focus)
- Queue up to 3 toasts

**Example**:
```json
{"id":null,"type":"ChatToast","payload":{"author":"Alice","text":"Anyone want to team up?"}}
```

---

### ScoreboardState

Push scoreboard data when visible.

**Type**: `"ScoreboardState"`

**Payload**:
```json
{
  "server_name": "string",  // Server/match name
  "rows": [
    {
      "name": "string",     // Player name
      "ping_ms": 32,        // Player ping
      "score": 5,           // Optional score
      "kills": 5,           // Optional kills
      "deaths": 2,          // Optional deaths
      "team": "red"         // Optional team ("red", "blue", null)
    }
  ]
}
```

**Constraints**:
- Max 64 rows
- Only pushed when scoreboard is visible
- Update frequency: on change or 2-5 Hz while visible

**Example**:
```json
{
  "id": null,
  "type": "ScoreboardState",
  "payload": {
    "server_name": "Arena Battle",
    "rows": [
      {"name": "Alice", "ping_ms": 25, "score": 10, "kills": 10, "deaths": 3, "team": "red"},
      {"name": "Bob", "ping_ms": 45, "score": 8, "kills": 8, "deaths": 5, "team": "blue"},
      {"name": "Charlie", "ping_ms": 60, "score": 5, "kills": 5, "deaths": 7, "team": "red"}
    ]
  }
}
```

---

### UiConfig

Push UI configuration on startup.

**Type**: `"UiConfig"`

**Payload**:
```json
{
  "cefHudEnabled": true,        // Whether web HUD is active
  "cefCrosshairEnabled": true,  // Whether web crosshair is active
  "keybinds": {
    "chatOpen": "Enter",        // Key to open chat
    "scoreboard": "Tab"         // Key to show scoreboard
  }
}
```

**Push Timing**: Once on `ui.ready` handshake

**Example**:
```json
{"id":null,"type":"UiConfig","payload":{"cefHudEnabled":true,"cefCrosshairEnabled":true,"keybinds":{"chatOpen":"Enter","scoreboard":"Tab"}}}
```

---

## Error Codes Summary

| Code | Category | Description |
|------|----------|-------------|
| ECHAT001 | Chat | Message exceeds 200 characters |
| ECHAT002 | Chat | Rate limited (500ms cooldown) |
| ECHAT003 | Chat | Empty message after trim |
| EBRG001 | Bridge | Version mismatch (from F031) |
| EBRG002 | Bridge | Invalid message format (from F031) |
| EBRG003 | Bridge | Unknown message type (from F031) |

---

## Message Type Registry

Complete list of message types for Feature 032 (extends F031):

**Request/Response**:
- `Handshake` (F031)
- `GetConfig` (F031)
- `SetConfig` (F031)
- `FetchServers` (F031)
- `ToggleFavorite` (F031)
- `Connect` (F031)
- `Quit` (F031)
- `ChatSend` (F032) ← NEW
- `ChatOpen` (F032) ← NEW
- `ChatClose` (F032) ← NEW
- `ChatClear` (F032) ← NEW

**Push Events**:
- `ConnectionStatus` (F031)
- `FavoritesUpdated` (F031)
- `HudState` (F032) ← NEW
- `ChatMessage` (F032) ← NEW
- `ChatToast` (F032) ← NEW
- `ScoreboardState` (F032) ← NEW
- `UiConfig` (F032) ← NEW
