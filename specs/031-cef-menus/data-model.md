# Data Model: CEF Menus

**Feature**: 031-cef-menus
**Date**: 2025-12-18

## Entities

### BridgeMessage

Represents a message exchanged between JavaScript UI and Rust game engine.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| id | String? | Optional, max 64 chars | Correlation ID for request/response. Null for push events. |
| type | String | Required, enum | Message type identifier |
| payload | Object | Required | Type-specific data |
| ok | Boolean? | Response only | Success indicator (responses only) |
| error | ErrorInfo? | Response only | Error details if ok=false |

**Message Types** (type field):
- Request types: `Handshake`, `GetConfig`, `SetConfig`, `FetchServers`, `ToggleFavorite`, `Connect`, `Quit`
- Response types: Same as request (response uses same type, distinguished by presence of `ok` field)
- Push types: `ConnectionStatus`, `FavoritesUpdated`

### ErrorInfo

Error details returned in failed responses.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| code | String | Required, format E[CAT][NUM] | Error code (e.g., ECON001) |
| message | String | Required, max 256 chars | User-friendly error message |

### ServerEntry

Represents a server in the browser list.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| address | String | Required, max 64 chars | Server address (ip:port or hostname:port) |
| name | String | Required, max 64 chars | Display name |
| region | String | Optional, max 32 chars | Geographic region |
| tags | String[] | Optional, each max 32 chars | Game mode tags |
| players | Number | Required, >= 0 | Current player count |
| max_players | Number | Required, > 0 | Maximum player capacity |
| protocol_version | Number | Required | Protocol version number |
| is_favorite | Boolean | Required | Whether server is in favorites |
| is_compatible | Boolean | Required | Whether protocol version matches client |

### GameConfigUI

Subset of GameConfig exposed to UI (matches native settings exactly).

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| sensitivity | Number | 0.0001 - 0.01 | Mouse sensitivity |
| fov_degrees | Number | 60 - 110 | Field of view in degrees |
| fullscreen | Boolean | - | Fullscreen mode enabled |
| audio_muted | Boolean | - | Master audio muted |
| keybinds | KeybindMap | - | Action to key mappings |

### KeybindMap

Key binding configuration.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| forward | String | Key name | Forward movement key |
| backward | String | Key name | Backward movement key |
| left | String | Key name | Strafe left key |
| right | String | Key name | Strafe right key |
| jump | String | Key name | Jump key |
| attack | String | Key name | Primary attack key |
| place_block | String | Key name | Place block key |
| remove_block | String | Key name | Remove block key |
| pause | String | Key name | Pause/menu key |
| toggle_debug | String | Key name | Debug overlay toggle |

### Favorites

Persisted favorites data structure.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| version | Number | Required, = 1 | File format version |
| favorites | String[] | Each max 64 chars | List of server addresses |

**File Location**: `~/.config/plix/favorites.toml`

### ConnectionStatus

Push event for connection state updates.

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| state | String | enum | Current state: "connecting", "connected", "failed", "disconnected" |
| address | String | Optional | Target server address |
| message | String | Optional, max 256 chars | Status message or error |

## State Transitions

### Connection Flow

```
[Disconnected] --Connect--> [Connecting] --Success--> [Connected]
                                |
                                +--Failure--> [Failed] --Retry/Cancel--> [Disconnected]
```

### UI Navigation

```
[MainMenu] --Servers--> [ServerBrowser]
    |                        |
    +--Settings--> [Settings]
    |                        |
    +--Quit--> [Exit]        +--Connect--> [Connecting]
                             |
                             +--Back--> [MainMenu]
```

## Validation Rules

### String Length Limits (FR-013)

| Field | Max Length |
|-------|------------|
| Server name | 64 chars |
| Search query | 32 chars |
| Display name | 32 chars |
| Region | 32 chars |
| Tag (each) | 32 chars |
| Error message | 256 chars |
| Correlation ID | 64 chars |

### Numeric Ranges

| Field | Min | Max |
|-------|-----|-----|
| sensitivity | 0.0001 | 0.01 |
| fov_degrees | 60 | 110 |
| players | 0 | max_players |
| max_players | 1 | 64 |

## Relationships

```
BridgeMessage 1--* ErrorInfo (only on failure)
ServerEntry *--1 Favorites (by address)
GameConfigUI 1--1 KeybindMap
```
