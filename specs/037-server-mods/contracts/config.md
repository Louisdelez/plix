# Configuration Contract: Server Mods + Client Sync

**Feature**: 037-server-mods
**Date**: 2025-12-19

## Server Configuration

### server_mods.toml Extension

The existing `server_mods.toml` from Feature 036 is extended with join policy and sync configuration sections.

```toml
# ============================================================================
# Existing sections from Feature 036
# ============================================================================

[[registries]]
name = "official"
url = "https://mods.plix.dev/index.json"
priority = 100

[[mods]]
id = "my-mod"
version = "^1.0"

[download]
connect_timeout_secs = 30
read_timeout_secs = 120
retries = 3
max_bundle_size = 52428800  # 50 MB

[cache]
path = "/var/cache/plix/mods"

[trust]
require_signature = false
allowed_keys = []

# ============================================================================
# NEW: Feature 037 - Join Policy Configuration
# ============================================================================

[join_policy]
# Allow connections when all mods are server-only
# Default: true
allow_server_only = true

# Allow payload synchronization during handshake
# If false, clients without cached payloads will be refused
# Default: true
allow_payload_sync = true

# Require clients to support payload sync
# If true, clients without sync capability are refused
# Default: false
require_payload_sync = false

# ============================================================================
# NEW: Feature 037 - Sync Configuration
# ============================================================================

[sync]
# Maximum payload size in megabytes
# Mods with larger client payloads will fail validation at server startup
# Default: 25
max_payload_mb = 25

# Chunk size for payload transfer in kilobytes
# Larger chunks = fewer messages, but more memory per in-flight chunk
# Default: 256
chunk_size_kb = 256

# Maximum concurrent in-flight chunks per transfer
# Higher = faster transfer on good connections, more memory usage
# Default: 8
max_inflight_chunks = 8

# Transfer timeout in seconds
# Transfers not completed within this time are aborted
# Default: 300 (5 minutes)
transfer_timeout_secs = 300
```

### Configuration Validation

At server startup, the following validations are performed:

| Field | Constraint | Error |
|-------|-----------|-------|
| `max_payload_mb` | 1 ≤ value ≤ 100 | "max_payload_mb must be between 1 and 100" |
| `chunk_size_kb` | 16 ≤ value ≤ 1024 | "chunk_size_kb must be between 16 and 1024" |
| `max_inflight_chunks` | 1 ≤ value ≤ 32 | "max_inflight_chunks must be between 1 and 32" |
| `transfer_timeout_secs` | 30 ≤ value ≤ 1800 | "transfer_timeout_secs must be between 30 and 1800" |

### Default Configuration

If sections are omitted, these defaults apply:

```toml
[join_policy]
allow_server_only = true
allow_payload_sync = true
require_payload_sync = false

[sync]
max_payload_mb = 25
chunk_size_kb = 256
max_inflight_chunks = 8
transfer_timeout_secs = 300
```

## Mod Manifest Extension

### mod.toml Extension

The mod manifest (`mod.toml`) is extended with runtime and network fields.

```toml
[mod]
id = "my-mod"
name = "My Mod"
version = "1.0.0"
api_version = 1
description = "An example mod"
author = "Developer Name"

# ============================================================================
# NEW: Feature 037 - Runtime Configuration
# ============================================================================

# Where this mod executes
# Values: "server" (default), "client", "both"
# - "server": Mod runs only on server, client doesn't need anything
# - "client": Mod runs only on client (future feature)
# - "both": Mod runs on both server and client (future feature)
runtime = "server"

# Whether this mod has data to sync to clients
# If true, client_payload_files must list files to include
# Default: false
client_payload = true

# Files to include in client payload
# Paths relative to mod bundle root
# Supports glob patterns
# Only used when client_payload = true
client_payload_files = [
    "client/config.json",
    "client/items/*.json",
    "client/strings.toml",
]

# ============================================================================
# NEW: Feature 037 - Network Configuration
# ============================================================================

[mod.network]
# Channels client is allowed to send messages on
# Format: subchannel names (without mod:{id}: prefix)
# Client can send to: mod:{mod_id}:{subchannel}
# Default: [] (no client-to-server messages allowed)
allowed_client_channels = ["input", "config", "request"]
```

### Manifest Validation

| Field | Constraint | Error |
|-------|-----------|-------|
| `runtime` | One of: "server", "client", "both" | "Invalid runtime value" |
| `client_payload` | Boolean | "client_payload must be boolean" |
| `client_payload_files` | Required if `client_payload=true` | "client_payload_files required when client_payload is true" |
| `client_payload_files` | Valid paths within bundle | "Invalid path in client_payload_files: {path}" |
| `allowed_client_channels` | Max 32 channels | "Too many allowed_client_channels (max 32)" |
| `allowed_client_channels` | Each ≤ 64 chars | "Channel name too long: {name}" |

### Client Payload Constraints

When `client_payload = true`:

1. **Total size limit**: Combined files must not exceed `max_payload_mb`
2. **No executable content**: Files must not be `.wasm`, `.dll`, `.so`, `.exe`
3. **Deterministic ordering**: Files are archived in sorted order by path
4. **Hash computation**: SHA-256 of the complete ZIP archive

### Allowed File Extensions for Client Payload

| Category | Extensions |
|----------|------------|
| Data | `.json`, `.toml`, `.yaml`, `.yml`, `.xml` |
| Text | `.txt`, `.md`, `.csv` |
| Images | `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.webp` |
| Audio | `.ogg`, `.mp3`, `.wav`, `.flac` |
| Fonts | `.ttf`, `.otf`, `.woff`, `.woff2` |

Executables are explicitly blocked:
- `.wasm`, `.dll`, `.so`, `.dylib`, `.exe`, `.sh`, `.bat`, `.ps1`

## Client Configuration

### Payload Cache Location

The client stores cached payloads at:

| Platform | Path |
|----------|------|
| Linux | `~/.local/share/plix/mods/payloads/` |
| macOS | `~/Library/Application Support/plix/mods/payloads/` |
| Windows | `%APPDATA%\plix\mods\payloads\` |

### Cache File Format

For each cached payload:

```text
{hash}.bin   # The payload archive (ZIP)
{hash}.meta  # Metadata JSON
```

Metadata format:
```json
{
  "mod_id": "my-mod",
  "mod_version": "1.0.0",
  "cached_at": "2025-12-19T10:00:00Z",
  "size": 1048576
}
```

### Cache Cleanup

The client may implement automatic cache cleanup:
- Remove entries older than 30 days (suggested)
- Limit total cache size to 500MB (suggested)
- Remove invalid entries (missing .bin or .meta)

These are client implementation details, not protocol requirements.
