# Bridge Message Contracts: CEF Media Embeds

**Feature**: 033-cef-embeds
**Protocol Version**: 1.0
**Date**: 2025-12-18

## Message Envelope

All messages follow the existing Feature 031 bridge protocol:

```json
{
  "id": "string | null",
  "type": "string",
  "payload": {}
}
```

- `id`: Correlation ID for request/response (null for push events)
- `type`: Message type identifier
- `payload`: Type-specific data

## UI → Game Messages (Request/Response)

### EmbedOpenPanel

Open/show the embed panel.

**Request**:
```json
{
  "id": "req-123",
  "type": "EmbedOpenPanel",
  "payload": {}
}
```

**Response (Success)**:
```json
{
  "id": "req-123",
  "type": "EmbedOpenPanel",
  "ok": true,
  "payload": {}
}
```

**Response (Error)** - Embeds disabled:
```json
{
  "id": "req-123",
  "type": "EmbedOpenPanel",
  "ok": false,
  "error": {
    "code": "EEMB002",
    "message": "Embeds feature is disabled"
  }
}
```

---

### EmbedClosePanel

Close/hide the embed panel.

**Request**:
```json
{
  "id": "req-124",
  "type": "EmbedClosePanel",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "req-124",
  "type": "EmbedClosePanel",
  "ok": true,
  "payload": {}
}
```

---

### EmbedFocus

Notify game that embed panel received focus (click on panel).

**Request**:
```json
{
  "id": "req-125",
  "type": "EmbedFocus",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "req-125",
  "type": "EmbedFocus",
  "ok": true,
  "payload": {}
}
```

---

### EmbedUnfocus

Notify game that embed panel lost focus (Escape or click outside).

**Request**:
```json
{
  "id": "req-126",
  "type": "EmbedUnfocus",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "req-126",
  "type": "EmbedUnfocus",
  "ok": true,
  "payload": {}
}
```

---

### EmbedLoad

Request to load media content.

**Request**:
```json
{
  "id": "req-127",
  "type": "EmbedLoad",
  "payload": {
    "provider": "youtube",
    "url_or_id": "https://youtube.com/watch?v=dQw4w9WgXcQ"
  }
}
```

**Payload Fields**:
- `provider`: `"youtube"` | `"twitch"` | `"spotify"`
- `url_or_id`: URL or direct ID (video ID, channel name)

**Response (Success)**:
```json
{
  "id": "req-127",
  "type": "EmbedLoad",
  "ok": true,
  "payload": {
    "embed_url": "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ?autoplay=0&controls=1"
  }
}
```

**Response (Error)** - Invalid URL:
```json
{
  "id": "req-127",
  "type": "EmbedLoad",
  "ok": false,
  "error": {
    "code": "EEMB001",
    "message": "Invalid URL or video ID"
  }
}
```

**Response (Error)** - Provider disabled:
```json
{
  "id": "req-127",
  "type": "EmbedLoad",
  "ok": false,
  "error": {
    "code": "EEMB002",
    "message": "YouTube provider is disabled"
  }
}
```

**Response (Error)** - Rate limited:
```json
{
  "id": "req-127",
  "type": "EmbedLoad",
  "ok": false,
  "error": {
    "code": "EEMB004",
    "message": "Please wait before loading another video"
  }
}
```

---

### EmbedStop

Stop/clear the current embed.

**Request**:
```json
{
  "id": "req-128",
  "type": "EmbedStop",
  "payload": {}
}
```

**Response**:
```json
{
  "id": "req-128",
  "type": "EmbedStop",
  "ok": true,
  "payload": {}
}
```

---

## Game → UI Messages (Push Events)

### EmbedState

Push current embed panel state to UI.

```json
{
  "id": null,
  "type": "EmbedState",
  "payload": {
    "visible": true,
    "focused": false,
    "provider": "youtube",
    "embed_url": "https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ",
    "state": "playing"
  }
}
```

**Payload Fields**:
- `visible`: Panel visibility state
- `focused`: Whether panel has input focus
- `provider`: Current provider (`"youtube"` | `"twitch"` | `"spotify"` | `null`)
- `embed_url`: Current canonical embed URL (`null` if empty)
- `state`: Slot state (`"empty"` | `"loading"` | `"playing"` | `"error"`)

---

### EmbedError

Push error notification to UI.

```json
{
  "id": null,
  "type": "EmbedError",
  "payload": {
    "code": "EEMB003",
    "message": "Navigation to external domain blocked"
  }
}
```

---

### UiConfig (Extended)

Extends existing UiConfig push with embed settings.

```json
{
  "id": null,
  "type": "UiConfig",
  "payload": {
    "cefHudEnabled": true,
    "cefChatEnabled": true,
    "cefScoreboardEnabled": true,
    "embedsEnabled": true,
    "providersEnabled": {
      "youtube": true,
      "twitch": true,
      "spotify": false
    }
  }
}
```

---

## Error Codes Summary

| Code | Name | Description |
|------|------|-------------|
| EEMB001 | InvalidUrl | URL parsing failed or no valid ID extracted |
| EEMB002 | ProviderDisabled | The requested provider is disabled in config |
| EEMB003 | BlockedDomain | Navigation attempted to non-whitelisted domain |
| EEMB004 | RateLimited | Action attempted within 2s cooldown period |

---

## Message Type Enum Extension

Add to `MessageType` in `bridge/messages.rs`:

```rust
pub enum MessageType {
    // ... existing types ...

    // Feature 033: Embeds
    EmbedOpenPanel,
    EmbedClosePanel,
    EmbedFocus,
    EmbedUnfocus,
    EmbedLoad,
    EmbedStop,
}
```

Add to `PushType` in `bridge/messages.rs`:

```rust
pub enum PushType {
    // ... existing types ...

    // Feature 033: Embeds
    EmbedState,
    EmbedError,
}
```
