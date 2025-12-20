# Data Model: Server Mods + Client Sync

**Feature**: 037-server-mods
**Date**: 2025-12-19

## Entities

### ModSetDescriptor

Server's complete mod configuration sent to clients during handshake.

```rust
/// Server's mod configuration sent during handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModSetDescriptor {
    /// Protocol version for this descriptor format
    pub protocol_version: u8,
    /// Engine version (e.g., "0.37.0")
    pub engine_version: String,
    /// Mod API version
    pub api_version: u8,
    /// List of mods loaded on this server
    pub mods: Vec<ModEntry>,
}
```

**Validation Rules**:
- `protocol_version` must be ≥ 1
- `engine_version` must be valid semver
- `api_version` must match client's supported range
- `mods` may be empty (no mods loaded)

### ModEntry

Single mod's metadata within ModSetDescriptor.

```rust
/// Mod entry in the server's mod set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModEntry {
    /// Mod identifier (e.g., "my-mod")
    pub id: ModId,
    /// Exact version (e.g., "1.2.0")
    pub version: Version,
    /// SHA-256 hash of the mod bundle (from lockfile)
    pub bundle_hash: String,
    /// Runtime mode: where this mod executes
    pub runtime: RuntimeMode,
    /// Whether client must have this mod's payload
    pub required: bool,
    /// SHA-256 hash of client payload (if client_payload=true)
    pub payload_hash: Option<String>,
    /// Size of client payload in bytes
    pub payload_size: Option<u64>,
}

/// Where a mod executes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    /// Server-only execution (default)
    Server,
    /// Client-only execution (future)
    Client,
    /// Both server and client
    Both,
}

impl Default for RuntimeMode {
    fn default() -> Self {
        Self::Server
    }
}
```

**Validation Rules**:
- `id` must be valid ModId (lowercase alphanumeric + hyphens)
- `version` must be valid semver
- `bundle_hash` must be 64 hex characters (SHA-256)
- `payload_hash` required if mod has `client_payload=true`
- `payload_size` must be ≤ max_payload_mb configuration

### ClientCapabilities

Client's response during handshake.

```rust
/// Client's capabilities and cache state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    /// Whether client supports payload sync
    pub supports_sync: bool,
    /// SHA-256 hashes of cached payloads
    pub cached_payload_hashes: Vec<String>,
    /// Client engine version (for compatibility check)
    pub engine_version: Option<String>,
}
```

**Validation Rules**:
- `cached_payload_hashes` entries must be 64 hex characters
- Empty `cached_payload_hashes` is valid (no cache)

### JoinDecision

Server's determination after evaluating client capabilities.

```rust
/// Server's join decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinDecision {
    /// Join allowed immediately
    Ok,
    /// Join refused with reason
    Refused {
        /// Error code for programmatic handling
        code: JoinRefusalCode,
        /// Human-readable message
        message: String,
    },
    /// Sync required before join
    SyncRequired {
        /// Payloads that need to be transferred
        payloads: Vec<PayloadDescriptor>,
    },
}

/// Reason codes for join refusal
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum JoinRefusalCode {
    /// Client missing required mod
    ModMismatch,
    /// Client missing required payload and can't sync
    PayloadMissing,
    /// Client doesn't support sync but server requires it
    SyncUnsupported,
    /// Protocol version mismatch
    ProtocolMismatch,
    /// Engine version incompatible
    EngineIncompatible,
}

/// Descriptor for a payload that needs sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadDescriptor {
    /// Mod this payload belongs to
    pub mod_id: ModId,
    /// SHA-256 hash of payload
    pub hash: String,
    /// Size in bytes
    pub size: u64,
}
```

### ClientPayload

Data-only archive for client-side consumption.

```rust
/// Represents a client payload (not a struct, but format documentation)
///
/// Format: ZIP archive containing:
/// - Files listed in mod.toml `client_payload_files`
/// - No executable code (WASM, native binaries)
///
/// Identified by SHA-256 hash of the complete archive.
///
/// Cache path: ~/.local/share/plix/mods/payloads/{hash}.bin
/// Metadata path: ~/.local/share/plix/mods/payloads/{hash}.meta

/// Metadata stored alongside cached payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadMetadata {
    /// Mod this payload belongs to
    pub mod_id: ModId,
    /// Mod version at time of download
    pub mod_version: Version,
    /// When this was cached
    pub cached_at: DateTime<Utc>,
    /// Original size in bytes
    pub size: u64,
}
```

### PayloadChunk

Fragment of client payload for streaming transfer.

```rust
/// Messages for payload transfer protocol

/// Begin payload transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadBegin {
    /// SHA-256 hash of complete payload
    pub hash: String,
    /// Total size in bytes
    pub total_size: u64,
    /// Size of each chunk
    pub chunk_size: u32,
    /// Total number of chunks
    pub num_chunks: u32,
}

/// Single chunk of payload data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadChunk {
    /// SHA-256 hash of complete payload (reference)
    pub hash: String,
    /// Chunk index (0-based)
    pub index: u32,
    /// Chunk data
    pub data: Vec<u8>,
}

/// End of payload transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadEnd {
    /// SHA-256 hash of complete payload
    pub hash: String,
}

/// Client acknowledgment of successful payload receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadAck {
    /// SHA-256 hash of received payload
    pub hash: String,
}

/// Client request to resend missing chunks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayloadResendRequest {
    /// SHA-256 hash of payload
    pub hash: String,
    /// Indices of missing chunks
    pub missing_indices: Vec<u32>,
}
```

### JoinPolicy

Server configuration for mod compatibility and sync.

```rust
/// Join policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinPolicy {
    /// Allow connections when all mods are server-only
    #[serde(default = "default_true")]
    pub allow_server_only: bool,
    /// Allow payload synchronization
    #[serde(default = "default_true")]
    pub allow_payload_sync: bool,
    /// Require clients to support payload sync
    #[serde(default)]
    pub require_payload_sync: bool,
}

fn default_true() -> bool { true }

impl Default for JoinPolicy {
    fn default() -> Self {
        Self {
            allow_server_only: true,
            allow_payload_sync: true,
            require_payload_sync: false,
        }
    }
}
```

### SyncConfig

Configuration for payload synchronization.

```rust
/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Maximum payload size in megabytes
    #[serde(default = "default_max_payload_mb")]
    pub max_payload_mb: u32,
    /// Chunk size in kilobytes
    #[serde(default = "default_chunk_size_kb")]
    pub chunk_size_kb: u32,
    /// Maximum in-flight chunks
    #[serde(default = "default_max_inflight")]
    pub max_inflight_chunks: u8,
    /// Transfer timeout in seconds
    #[serde(default = "default_timeout")]
    pub transfer_timeout_secs: u32,
}

fn default_max_payload_mb() -> u32 { 25 }
fn default_chunk_size_kb() -> u32 { 256 }
fn default_max_inflight() -> u8 { 8 }
fn default_timeout() -> u32 { 300 }

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_payload_mb: 25,
            chunk_size_kb: 256,
            max_inflight_chunks: 8,
            transfer_timeout_secs: 300,
        }
    }
}
```

### ModManifest Extension

Extension to existing mod.toml manifest.

```rust
/// Extended mod manifest fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModManifestExt {
    /// Runtime mode (default: "server")
    #[serde(default)]
    pub runtime: RuntimeMode,
    /// Whether this mod has client payload
    #[serde(default)]
    pub client_payload: bool,
    /// Files to include in client payload
    #[serde(default)]
    pub client_payload_files: Vec<String>,
    /// Network configuration
    #[serde(default)]
    pub network: ModNetworkConfig,
}

/// Mod network configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModNetworkConfig {
    /// Channels client is allowed to send on
    /// Format: ["input", "config"] → client can send to mod:{id}:input, mod:{id}:config
    #[serde(default)]
    pub allowed_client_channels: Vec<String>,
}
```

## State Transitions

### Handshake State Machine (Client)

```text
┌──────────────┐
│   Initial    │
└──────┬───────┘
       │ Connect sent
       ▼
┌──────────────┐
│ AwaitModSet  │
└──────┬───────┘
       │ S2C_ModSetDescriptor received
       ▼
┌──────────────┐
│ Processing   │──── Build ClientCapabilities
└──────┬───────┘     Check cache
       │ C2S_ModSetResponse sent
       ▼
┌──────────────┐
│ AwaitDecision│
└──────┬───────┘
       │
       ├─── JoinDecision::Ok ──────────────▶ Connected
       │
       ├─── JoinDecision::Refused ─────────▶ Disconnected
       │
       └─── JoinDecision::SyncRequired ───▶ Syncing
                                               │
┌──────────────┐                              │
│   Syncing    │◀─────────────────────────────┘
└──────┬───────┘
       │ All payloads received + verified
       ▼
┌──────────────┐
│  Connected   │
└──────────────┘
```

### Payload Transfer State Machine (Client)

```text
┌──────────────┐
│    Idle      │
└──────┬───────┘
       │ PayloadBegin received
       ▼
┌──────────────┐
│  Receiving   │──── Allocate temp file
└──────┬───────┘     Track received chunks
       │
       │ PayloadChunk received
       │ (loop until all chunks)
       │
       │ PayloadEnd received
       ▼
┌──────────────┐
│  Verifying   │──── Compute SHA-256
└──────┬───────┘     Compare with expected
       │
       ├─── Match ────▶ Cache + PayloadAck ───▶ Complete
       │
       └─── Mismatch ─▶ Delete temp ──────────▶ Failed
```

## Relationships

```text
ModSetDescriptor 1───* ModEntry
       │
       └─────────────────────────────┐
                                     │
JoinDecision::SyncRequired 1───* PayloadDescriptor
                                     │
                                     │
PayloadBegin/Chunk/End ────────────*─┤ (all reference same hash)
                                     │
PayloadCache 1───* PayloadMetadata ──┘
```

## Indexes & Lookup

### Server Side

- `modset_by_lockfile`: ModSetDescriptor cached at startup from lockfile
- `payload_by_hash`: Map<SHA256, PayloadData> for serving client payloads

### Client Side

- `payload_cache`: Directory `~/.local/share/plix/mods/payloads/`
  - Lookup by SHA-256 hash: `{hash}.bin` exists?
  - Metadata: `{hash}.meta` (ModId, Version, timestamp)
