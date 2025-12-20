# Protocol Contract: Server Mods + Client Sync

**Feature**: 037-server-mods
**Protocol Version**: 1
**Date**: 2025-12-19

## Overview

This document specifies the network protocol for server mod handshake, join policy enforcement, and client payload synchronization.

## Message Types

All messages are serialized using bincode and transmitted over the existing TCP connection.

### Handshake Messages

#### S2C_ModSetDescriptor (Server → Client)

Sent immediately after successful `Connect` message processing.

```rust
ServerMessage::ModSetDescriptor {
    protocol_version: u8,       // Always 1 for this version
    engine_version: String,     // e.g., "0.37.0"
    api_version: u8,            // Mod API version
    mods: Vec<ModEntry>,        // List of loaded mods
}

struct ModEntry {
    id: String,                 // Mod identifier
    version: String,            // Semver version
    bundle_hash: String,        // SHA-256 (64 hex chars)
    runtime: String,            // "server" | "client" | "both"
    required: bool,             // Is client data required?
    payload_hash: Option<String>, // SHA-256 if client_payload=true
    payload_size: Option<u64>,  // Bytes if payload exists
}
```

#### C2S_ModSetResponse (Client → Server)

Client's response with capabilities and cache state.

```rust
ClientMessage::ModSetResponse {
    supports_sync: bool,              // Can receive payloads?
    cached_payload_hashes: Vec<String>, // SHA-256 hashes in cache
    engine_version: Option<String>,   // Client engine version
}
```

#### S2C_JoinDecision (Server → Client)

Server's decision based on policy evaluation.

```rust
ServerMessage::JoinDecision(JoinDecision)

enum JoinDecision {
    Ok,
    Refused {
        code: String,           // Error code (see table below)
        message: String,        // Human-readable message
    },
    SyncRequired {
        payloads: Vec<PayloadDescriptor>,
    },
}

struct PayloadDescriptor {
    mod_id: String,
    hash: String,               // SHA-256
    size: u64,                  // Bytes
}
```

### Payload Transfer Messages

#### S2C_PayloadBegin (Server → Client)

Initiates payload transfer.

```rust
ServerMessage::PayloadBegin {
    hash: String,               // SHA-256 of complete payload
    total_size: u64,            // Total bytes
    chunk_size: u32,            // Bytes per chunk (default 262144)
    num_chunks: u32,            // Total chunk count
}
```

#### S2C_PayloadChunk (Server → Client)

Single chunk of payload data.

```rust
ServerMessage::PayloadChunk {
    hash: String,               // Payload reference
    index: u32,                 // 0-based chunk index
    data: Vec<u8>,              // Chunk bytes (≤ chunk_size)
}
```

#### S2C_PayloadEnd (Server → Client)

Signals end of payload transfer.

```rust
ServerMessage::PayloadEnd {
    hash: String,               // Payload reference
}
```

#### C2S_PayloadAck (Client → Server)

Client acknowledgment of successful receipt.

```rust
ClientMessage::PayloadAck {
    hash: String,               // SHA-256 of verified payload
}
```

#### C2S_PayloadResendRequest (Client → Server)

Request to resend missing chunks (MVP recovery).

```rust
ClientMessage::PayloadResendRequest {
    hash: String,               // Payload reference
    missing_indices: Vec<u32>,  // Chunk indices to resend
}
```

### Mod Channel Messages

#### S2C_ModMessage (Server → Client)

Server-side mod sending data to client.

```rust
ServerMessage::ModMessage {
    channel: String,            // Format: "mod:{mod_id}:{subchannel}"
    data: Vec<u8>,              // Payload (≤ 8192 bytes)
}
```

#### C2S_ModMessage (Client → Server)

Client sending data to server-side mod.

```rust
ClientMessage::ModMessage {
    channel: String,            // Must match mod's allowed_client_channels
    data: Vec<u8>,              // Payload (≤ 8192 bytes)
}
```

## Error Codes

| Code | Description | User Message Template |
|------|-------------|----------------------|
| `MOD_MISMATCH` | Required mod missing | "Server requires mod '{id}' version {ver}" |
| `PAYLOAD_MISSING` | Required payload not available | "Missing client data for mod '{id}'" |
| `SYNC_UNSUPPORTED` | Client can't sync, server requires | "Your client doesn't support mod sync. Update to join." |
| `SYNC_TIMEOUT` | Transfer timeout | "Mod data transfer timed out. Try reconnecting." |
| `INTEGRITY_FAILED` | SHA-256 mismatch | "Mod data verification failed. Reconnecting..." |
| `PROTOCOL_MISMATCH` | Incompatible protocol version | "Protocol version mismatch. Update your client." |
| `ENGINE_INCOMPATIBLE` | Engine version incompatible | "Client version incompatible. Update to {min_version}." |

## Protocol Flow

### Successful Server-Only Join

```text
Client                              Server
  │                                   │
  │── Connect ───────────────────────▶│
  │◀── ConnectAccept ─────────────────│
  │◀── S2C_ModSetDescriptor ──────────│  (all mods runtime=server)
  │── C2S_ModSetResponse ────────────▶│  (supports_sync=true, hashes=[])
  │◀── S2C_JoinDecision::Ok ──────────│
  │                                   │
  │   [Normal gameplay begins]        │
```

### Join with Payload Sync

```text
Client                              Server
  │                                   │
  │── Connect ───────────────────────▶│
  │◀── ConnectAccept ─────────────────│
  │◀── S2C_ModSetDescriptor ──────────│  (payload_hash="abc123...")
  │── C2S_ModSetResponse ────────────▶│  (hashes=[] - cache miss)
  │◀── S2C_JoinDecision::SyncRequired─│  (payloads=[{hash,size}])
  │                                   │
  │◀── S2C_PayloadBegin ──────────────│
  │◀── S2C_PayloadChunk (0) ──────────│
  │◀── S2C_PayloadChunk (1) ──────────│
  │◀── ... ───────────────────────────│
  │◀── S2C_PayloadEnd ────────────────│
  │                                   │
  │   [Client verifies SHA-256]       │
  │                                   │
  │── C2S_PayloadAck ────────────────▶│
  │◀── S2C_JoinDecision::Ok ──────────│
  │                                   │
  │   [Normal gameplay begins]        │
```

### Join with Cache Hit

```text
Client                              Server
  │                                   │
  │── Connect ───────────────────────▶│
  │◀── ConnectAccept ─────────────────│
  │◀── S2C_ModSetDescriptor ──────────│  (payload_hash="abc123...")
  │── C2S_ModSetResponse ────────────▶│  (hashes=["abc123..."] - cache hit!)
  │◀── S2C_JoinDecision::Ok ──────────│  (no sync needed)
  │                                   │
  │   [Normal gameplay begins]        │
```

### Join Refused

```text
Client                              Server
  │                                   │
  │── Connect ───────────────────────▶│
  │◀── ConnectAccept ─────────────────│
  │◀── S2C_ModSetDescriptor ──────────│  (required=true, payload needed)
  │── C2S_ModSetResponse ────────────▶│  (supports_sync=false)
  │◀── S2C_JoinDecision::Refused ─────│  (code=SYNC_UNSUPPORTED)
  │                                   │
  │   [Connection closed]             │
```

## Rate Limits

| Limit | Value | Scope |
|-------|-------|-------|
| Mod messages per second | 20 | Per client |
| Mod message max size | 8192 bytes | Per message |
| Payload chunk size | 262144 bytes (256KB) | Per chunk |
| Max inflight chunks | 8 | Per transfer |
| Transfer timeout | 300 seconds | Per payload |
| Max payload size | 26214400 bytes (25MB) | Per mod |

## Versioning

- `protocol_version` in ModSetDescriptor enables future evolution
- Incompatible changes require protocol version increment
- Clients must reject unknown protocol versions
- Servers should support previous protocol version for one major release

## Security Considerations

1. **No code execution**: Client payloads contain data only (JSON, TOML, text)
2. **Hash verification**: All payloads verified via SHA-256 before use
3. **Size limits**: Prevents memory exhaustion attacks
4. **Channel isolation**: Clients cannot spoof other mods' channels
5. **Rate limiting**: Prevents DoS via message flooding
