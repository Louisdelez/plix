# Data Model: Cross-Platform Packaging & Headless Server

**Feature**: 041-cross-platform | **Date**: 2025-12-19

## Overview

This feature introduces data structures for build metadata, server configuration, and bundle manifests. No database persistence required - all data is file-based (JSON, TOML) or embedded at compile time.

---

## 1. Build Info

Compile-time embedded metadata for version tracking and reproducibility.

### BuildInfo (Embedded at Compile Time)

| Field | Type | Source | Description |
|-------|------|--------|-------------|
| `version` | String | Cargo.toml | Semantic version (e.g., "0.1.0") |
| `commit_sha` | String | Git HEAD | Full commit hash |
| `commit_sha_short` | String | Git HEAD | Short commit hash (7 chars) |
| `build_timestamp` | String | Build time | ISO 8601 UTC timestamp |
| `build_date` | String | Build time | Date only (YYYY-MM-DD) |
| `target_triple` | String | Rust target | e.g., "x86_64-unknown-linux-gnu" |
| `rust_version` | String | Compiler | Rust version used |
| `branch` | String | Git branch | Branch name at build time |
| `is_dirty` | bool | Git status | True if uncommitted changes |

### JSON Representation (build_info.json)

```json
{
  "version": "0.1.0",
  "commit_sha": "abc1234567890def",
  "commit_sha_short": "abc1234",
  "build_timestamp": "2025-12-19T14:30:00Z",
  "build_date": "2025-12-19",
  "target_triple": "x86_64-unknown-linux-gnu",
  "rust_version": "1.83.0",
  "branch": "041-cross-platform",
  "is_dirty": false
}
```

### Rust Type Definition

```rust
// In plix-common/src/build_info.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    pub version: String,
    pub commit_sha: String,
    pub commit_sha_short: String,
    pub build_timestamp: String,
    pub build_date: String,
    pub target_triple: String,
    pub rust_version: String,
    pub branch: String,
    pub is_dirty: bool,
}

impl BuildInfo {
    /// Display version for CLI output
    pub fn display_version(&self) -> String {
        format!("{} ({}) built {}",
            self.version,
            self.commit_sha_short,
            self.build_date
        )
    }
}
```

---

## 2. Server Exit Codes

Standardized exit codes for server binary.

### ExitCode Enumeration

| Code | Name | Description |
|------|------|-------------|
| 0 | `Success` | Clean shutdown |
| 1 | `GeneralError` | Unspecified runtime error |
| 2 | `Misuse` | Invalid CLI arguments |
| 64 | `BindFailed` | Port already in use or permission denied |
| 65 | `AssetLoadFailed` | Missing arena or asset files |
| 66 | `PersistenceError` | World save/load failure |
| 67 | `NetworkError` | Socket creation or network failure |
| 68 | `ShutdownTimeout` | Forced exit after timeout |

### Rust Type Definition

```rust
// In plix-server/src/exit_codes.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    Success = 0,
    GeneralError = 1,
    Misuse = 2,
    BindFailed = 64,
    AssetLoadFailed = 65,
    PersistenceError = 66,
    NetworkError = 67,
    ShutdownTimeout = 68,
}

impl ExitCode {
    pub fn exit(self) -> ! {
        std::process::exit(self as i32)
    }
}
```

---

## 3. Server Configuration

TOML-based configuration for headless server.

### ServerConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bind_address` | String | "0.0.0.0" | IP address to bind |
| `port` | u16 | 7777 | UDP port for game traffic |
| `max_players` | u8 | 32 | Maximum concurrent players |
| `tick_rate` | u8 | 30 | Server tick rate (Hz) |
| `arena` | String | "ffa_small" | Arena definition file |
| `server_name` | String | "Plix Server" | Display name in browser |
| `motd` | String | "" | Message of the day |
| `log_level` | String | "info" | Logging verbosity |
| `autosave_interval_secs` | u64 | 300 | World save interval |
| `shutdown_timeout_secs` | u64 | 5 | Graceful shutdown timeout |

### TOML Representation (server.toml)

```toml
# Plix Server Configuration

[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "ffa_small"

[server]
name = "My Plix Server"
motd = "Welcome!"

[logging]
level = "info"

[persistence]
autosave_interval_secs = 300
shutdown_timeout_secs = 5
```

### Rust Type Definition

```rust
// In plix-server/src/config.rs
#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub game: GameConfig,
    #[serde(default)]
    pub server: ServerMetaConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub persistence: PersistenceConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_bind")]
    pub bind_address: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_max_players")]
    pub max_players: u8,
}

impl ServerConfig {
    pub fn validate(&self) -> Result<(), Vec<ConfigError>> {
        let mut errors = Vec::new();

        if self.network.port == 0 {
            errors.push(ConfigError::InvalidPort("Port cannot be 0"));
        }
        if self.game.tick_rate < 20 || self.game.tick_rate > 60 {
            errors.push(ConfigError::InvalidTickRate(self.game.tick_rate));
        }
        if self.network.max_players == 0 || self.network.max_players > 128 {
            errors.push(ConfigError::InvalidMaxPlayers(self.network.max_players));
        }

        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

---

## 4. Bundle Manifest

Metadata describing a packaged bundle for validation.

### BundleManifest

| Field | Type | Description |
|-------|------|-------------|
| `bundle_type` | String | "client" or "server" |
| `platform` | String | Target platform identifier |
| `build_info` | BuildInfo | Embedded build metadata |
| `files` | Vec<FileEntry> | List of expected files |
| `total_size_bytes` | u64 | Total bundle size |

### FileEntry

| Field | Type | Description |
|-------|------|-------------|
| `path` | String | Relative path in bundle |
| `size_bytes` | u64 | File size |
| `sha256` | String | File checksum |
| `required` | bool | Whether file is mandatory |

### JSON Representation (manifest.json)

```json
{
  "bundle_type": "client",
  "platform": "linux-x86_64",
  "build_info": { /* BuildInfo object */ },
  "files": [
    {
      "path": "plix-client",
      "size_bytes": 45678901,
      "sha256": "abc123...",
      "required": true
    },
    {
      "path": "assets/ui/index.html",
      "size_bytes": 1234,
      "sha256": "def456...",
      "required": true
    }
  ],
  "total_size_bytes": 123456789
}
```

---

## 5. State Transitions

### Server Lifecycle States

```
┌─────────────┐
│   Created   │
└──────┬──────┘
       │ validate_config()
       ▼
┌─────────────┐
│ Initializing│
└──────┬──────┘
       │ bind() + load_assets()
       ▼
┌─────────────┐     SIGINT/SIGTERM
│   Running   │ ─────────────────────┐
└──────┬──────┘                      │
       │ shutdown_signal             │
       ▼                             ▼
┌─────────────┐              ┌──────────────┐
│  Stopping   │ ─────────────│ Force Exited │
└──────┬──────┘  timeout     └──────────────┘
       │ cleanup                   (code 68)
       ▼
┌─────────────┐
│   Stopped   │
└─────────────┘
    (code 0)
```

### Exit Code Mapping

| Transition | From | To | Exit Code |
|------------|------|-----|-----------|
| Config validation fails | Created | - | 2 (Misuse) |
| Bind fails | Initializing | - | 64 (BindFailed) |
| Asset load fails | Initializing | - | 65 (AssetLoadFailed) |
| Clean shutdown | Running | Stopped | 0 (Success) |
| Shutdown timeout | Stopping | Force Exited | 68 (ShutdownTimeout) |
| Runtime error | Running | - | 1 (GeneralError) |

---

## 6. Relationships

```
┌─────────────────┐
│   BuildInfo     │
└────────┬────────┘
         │ embedded in
         ▼
┌─────────────────┐     validates     ┌─────────────────┐
│ BundleManifest  │ ◄────────────────►│   FileEntry[]   │
└────────┬────────┘                   └─────────────────┘
         │ describes
         ▼
┌─────────────────┐
│  Bundle (zip/   │
│  tar.gz file)   │
└─────────────────┘

┌─────────────────┐     parsed from     ┌─────────────────┐
│  ServerConfig   │ ◄──────────────────│   server.toml   │
└────────┬────────┘                     └─────────────────┘
         │ configures
         ▼
┌─────────────────┐     uses     ┌─────────────────┐
│ HeadlessServer  │ ─────────────│   ExitCode      │
└─────────────────┘              └─────────────────┘
```

---

## File Locations

| Entity | Location | Format |
|--------|----------|--------|
| BuildInfo | Embedded in binary | Rust const |
| build_info.json | Bundle root | JSON |
| server.toml | `~/.config/plix/` or bundle | TOML |
| server_mods.toml | Same as server.toml | TOML |
| manifest.json | Bundle root | JSON |
