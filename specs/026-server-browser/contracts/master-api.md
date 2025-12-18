# Master Server HTTP API Contract

**Version**: 1.0.0
**Base URL**: `http://{master_host}:{master_port}`
**Content-Type**: `application/json`

## Endpoints

### GET /servers

Retrieve list of active game servers.

**Request**:
- Method: `GET`
- Path: `/servers`
- Headers: None required

**Response** (200 OK):
```json
{
  "servers": [
    {
      "server_id": "a1b2c3d4e5f67890",
      "name": "My Awesome Server",
      "host": "192.168.1.100",
      "port": 7777,
      "region": "eu-west",
      "tags": ["ctf", "competitive"],
      "player_count": 12,
      "max_players": 32,
      "game_modes": ["ctf", "tdm"],
      "protocol_version": "0.1.0",
      "last_seen": 1702828800
    }
  ],
  "total": 1,
  "timestamp": 1702828805
}
```

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| servers | Array<ServerEntry> | List of active servers (TTL not expired) |
| total | Integer | Total server count |
| timestamp | Integer | Response timestamp (Unix epoch seconds) |

**Error Responses**:
- `500 Internal Server Error`: Server error
  ```json
  { "error": "Internal server error" }
  ```

---

### POST /heartbeat

Register or update a game server.

**Request**:
- Method: `POST`
- Path: `/heartbeat`
- Headers:
  - `Content-Type: application/json`
- Body:
```json
{
  "name": "My Awesome Server",
  "host": "192.168.1.100",
  "port": 7777,
  "region": "eu-west",
  "tags": ["ctf", "competitive"],
  "player_count": 12,
  "max_players": 32,
  "game_modes": ["ctf", "tdm"],
  "protocol_version": "0.1.0"
}
```

**Request Fields**:
| Field | Type | Required | Constraints |
|-------|------|----------|-------------|
| name | String | Yes | 1-64 chars, alphanumeric + space/hyphen/underscore |
| host | String | Yes | 1-255 chars, valid IP or hostname |
| port | Integer | Yes | 1-65535 |
| region | String | Yes | 1-32 chars, alphanumeric + hyphen |
| tags | Array<String> | No | Max 10 items, each 1-32 chars |
| player_count | Integer | Yes | 0-255, <= max_players |
| max_players | Integer | Yes | 1-255 |
| game_modes | Array<String> | No | Max 10 items, each 1-32 chars |
| protocol_version | String | Yes | Non-empty |

**Response** (200 OK):
```json
{
  "success": true,
  "server_id": "a1b2c3d4e5f67890"
}
```

**Response** (400 Bad Request - Validation Error):
```json
{
  "success": false,
  "server_id": "",
  "error": "name: exceeds 64 characters"
}
```

**Response** (429 Too Many Requests - Rate Limited):
```json
{
  "success": false,
  "server_id": "",
  "error": "Rate limit exceeded. Max 10 requests per minute."
}
```

**Response Fields**:
| Field | Type | Description |
|-------|------|-------------|
| success | Boolean | Whether heartbeat was accepted |
| server_id | String | Assigned server ID (hash of host:port) |
| error | String? | Error message if success is false |

---

### GET /health

Health check endpoint.

**Request**:
- Method: `GET`
- Path: `/health`

**Response** (200 OK):
```json
{
  "status": "ok",
  "servers_active": 42,
  "uptime_secs": 3600
}
```

---

## Rate Limiting

- **Limit**: 10 requests per minute per IP address
- **Applies to**: `POST /heartbeat` only
- **Response when exceeded**: HTTP 429 with error message

## Server Entry TTL

- **TTL**: 60 seconds from last heartbeat
- **Recommended heartbeat interval**: 20 seconds
- **Behavior**: Expired entries automatically filtered from `GET /servers` response

## Error Format

All error responses follow this format:
```json
{
  "error": "Human-readable error message"
}
```

## Validation Rules

### Name Validation
- Length: 1-64 characters
- Allowed characters: `a-z`, `A-Z`, `0-9`, space, `-`, `_`
- Trimmed of leading/trailing whitespace

### Region Validation
- Length: 1-32 characters
- Allowed characters: `a-z`, `A-Z`, `0-9`, `-`
- Case-insensitive comparison

### Tag Validation
- Max tags: 10
- Tag length: 1-32 characters per tag
- Allowed characters: `a-z`, `A-Z`, `0-9`, `-`, `_`

### Host Validation
- Valid IPv4 address, IPv6 address, or hostname
- Length: 1-255 characters

## Example Usage

### Game Server Heartbeat (curl)
```bash
curl -X POST http://master.plix.game:8080/heartbeat \
  -H "Content-Type: application/json" \
  -d '{
    "name": "EU Competitive CTF",
    "host": "game1.example.com",
    "port": 7777,
    "region": "eu-west",
    "tags": ["ctf", "competitive", "ranked"],
    "player_count": 16,
    "max_players": 32,
    "game_modes": ["ctf"],
    "protocol_version": "0.1.0"
  }'
```

### Client List Servers (curl)
```bash
curl http://master.plix.game:8080/servers
```
