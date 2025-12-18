# Bridge Messages Contract

**Feature**: 031-cef-menus
**Version**: 1.0
**Date**: 2025-12-18

## Protocol Overview

All communication between JavaScript UI and Rust game engine uses JSON messages through the `window.plix` bridge object.

### Message Format

**Request (JS → Rust)**:
```json
{
  "id": "req-123",
  "type": "MessageType",
  "payload": { ... }
}
```

**Response (Rust → JS)**:
```json
{
  "id": "req-123",
  "type": "MessageType",
  "ok": true,
  "payload": { ... }
}
```

**Error Response (Rust → JS)**:
```json
{
  "id": "req-123",
  "type": "MessageType",
  "ok": false,
  "error": {
    "code": "EXXX000",
    "message": "User-friendly error message"
  }
}
```

**Push Event (Rust → JS)**:
```json
{
  "id": null,
  "type": "PushType",
  "payload": { ... }
}
```

---

## Message Types

### Handshake

Validates bridge version compatibility. MUST be sent first on UI load.

**Request**:
```json
{
  "id": "handshake-1",
  "type": "Handshake",
  "payload": {
    "bridge_version": "1.0"
  }
}
```

**Response (Success)**:
```json
{
  "id": "handshake-1",
  "type": "Handshake",
  "ok": true,
  "payload": {
    "supported_version": "1.0",
    "display_name": "PlayerName"
  }
}
```

**Response (Version Mismatch)**:
```json
{
  "id": "handshake-1",
  "type": "Handshake",
  "ok": false,
  "error": {
    "code": "EBRG001",
    "message": "Bridge version 2.0 not supported. Expected 1.x"
  }
}
```

---

### GetConfig

Retrieves current game configuration for settings page.

**Request**:
```json
{
  "id": "cfg-1",
  "type": "GetConfig",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "cfg-1",
  "type": "GetConfig",
  "ok": true,
  "payload": {
    "sensitivity": 0.003,
    "fov_degrees": 70,
    "fullscreen": false,
    "audio_muted": false,
    "keybinds": {
      "forward": "W",
      "backward": "S",
      "left": "A",
      "right": "D",
      "jump": "Space",
      "attack": "LMB",
      "place_block": "RMB",
      "remove_block": "LMB",
      "pause": "Escape",
      "toggle_debug": "F3"
    }
  }
}
```

---

### SetConfig

Updates game configuration. Validation is performed server-side.

**Request**:
```json
{
  "id": "cfg-2",
  "type": "SetConfig",
  "payload": {
    "sensitivity": 0.005,
    "fov_degrees": 90,
    "fullscreen": true,
    "audio_muted": false,
    "keybinds": {
      "forward": "W",
      ...
    }
  }
}
```

**Response (Success)**:
```json
{
  "id": "cfg-2",
  "type": "SetConfig",
  "ok": true,
  "payload": {}
}
```

**Response (Validation Error)**:
```json
{
  "id": "cfg-2",
  "type": "SetConfig",
  "ok": false,
  "error": {
    "code": "ECFG001",
    "message": "Sensitivity must be between 0.0001 and 0.01"
  }
}
```

---

### FetchServers

Retrieves server list from master server.

**Request**:
```json
{
  "id": "srv-1",
  "type": "FetchServers",
  "payload": {}
}
```

**Response (Success)**:
```json
{
  "id": "srv-1",
  "type": "FetchServers",
  "ok": true,
  "payload": {
    "servers": [
      {
        "address": "192.168.1.10:7777",
        "name": "My Server",
        "region": "EU",
        "tags": ["FFA", "Vanilla"],
        "players": 8,
        "max_players": 16,
        "protocol_version": 1,
        "is_favorite": true,
        "is_compatible": true
      }
    ]
  }
}
```

**Response (Master Unreachable)**:
```json
{
  "id": "srv-1",
  "type": "FetchServers",
  "ok": false,
  "error": {
    "code": "ESRV001",
    "message": "Could not reach server list. Check your connection."
  }
}
```

---

### ToggleFavorite

Adds or removes a server from favorites.

**Request**:
```json
{
  "id": "fav-1",
  "type": "ToggleFavorite",
  "payload": {
    "address": "192.168.1.10:7777"
  }
}
```

**Response**:
```json
{
  "id": "fav-1",
  "type": "ToggleFavorite",
  "ok": true,
  "payload": {
    "is_favorite": true
  }
}
```

---

### Connect

Initiates connection to a game server.

**Request**:
```json
{
  "id": "con-1",
  "type": "Connect",
  "payload": {
    "address": "192.168.1.10:7777"
  }
}
```

**Response (Connection Started)**:
```json
{
  "id": "con-1",
  "type": "Connect",
  "ok": true,
  "payload": {}
}
```

**Note**: Connection progress is sent via `ConnectionStatus` push events.

---

### Quit

Requests clean game exit.

**Request**:
```json
{
  "id": "quit-1",
  "type": "Quit",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "quit-1",
  "type": "Quit",
  "ok": true,
  "payload": {}
}
```

---

## Push Events

### ConnectionStatus

Sent by Rust to notify UI of connection state changes.

```json
{
  "id": null,
  "type": "ConnectionStatus",
  "payload": {
    "state": "connecting",
    "address": "192.168.1.10:7777",
    "message": "Connecting to server..."
  }
}
```

**States**: `"connecting"`, `"connected"`, `"failed"`, `"disconnected"`

---

### FavoritesUpdated

Sent when favorites change (e.g., from native UI).

```json
{
  "id": null,
  "type": "FavoritesUpdated",
  "payload": {
    "favorites": ["192.168.1.10:7777", "game.example.com:7777"]
  }
}
```

---

## Error Codes

| Code | Category | Description |
|------|----------|-------------|
| EBRG001 | Bridge | Version mismatch |
| EBRG002 | Bridge | Invalid message format |
| EBRG003 | Bridge | Unknown message type |
| ECFG001 | Config | Validation failed |
| ECFG002 | Config | Save failed |
| ESRV001 | Server | Master unreachable |
| ESRV002 | Server | Empty server list |
| ECON001 | Connection | Connection timeout |
| ECON002 | Connection | Connection refused |
| ECON003 | Connection | Version incompatible |
| ECON004 | Connection | Server full |
