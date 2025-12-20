# Data Model: Mod Distribution

**Feature Branch**: `036-mod-distribution`
**Date**: 2025-12-18
**Status**: Complete

## Overview

This document defines the data structures, schemas, and relationships for the mod distribution system.

## Core Types

### ModId

Unique identifier for a mod.

```rust
/// Unique mod identifier (lowercase alphanumeric + hyphens)
/// Examples: "my-mod", "core-lib", "weapons-pack-v2"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModId(String);

impl ModId {
    /// Create a new ModId, validating format
    pub fn new(id: impl Into<String>) -> Result<Self, DistributionError> {
        let id = id.into();
        if Self::is_valid(&id) {
            Ok(Self(id))
        } else {
            Err(DistributionError::invalid_mod_id(&id))
        }
    }

    /// Validate mod ID format: [a-z0-9][a-z0-9-]*[a-z0-9]
    pub fn is_valid(id: &str) -> bool {
        !id.is_empty()
            && id.len() <= 64
            && id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !id.starts_with('-')
            && !id.ends_with('-')
    }
}
```

### ModVersion

A specific version with metadata.

```rust
use semver::Version;

/// A specific mod version in a registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModVersion {
    /// SemVer version
    pub version: Version,

    /// SHA-256 hash of the bundle (hex-encoded, 64 chars)
    pub sha256: String,

    /// Download URL (absolute or relative to registry base)
    pub download_url: String,

    /// Bundle size in bytes
    pub size: u64,

    /// Dependencies with version constraints
    pub dependencies: Vec<ModDependency>,

    /// Required API version (from plix-mod-core)
    pub api_version: u32,

    /// Optional engine version constraints
    #[serde(default)]
    pub engine: Option<EngineConstraint>,

    /// Optional Ed25519 signature (hex-encoded, 128 chars)
    #[serde(default)]
    pub signature: Option<String>,

    /// Signing key ID (hex-encoded public key prefix, 16 chars)
    #[serde(default)]
    pub signer: Option<String>,

    /// Publication timestamp (RFC 3339)
    #[serde(default)]
    pub published_at: Option<String>,

    /// Yanked flag (version should not be installed)
    #[serde(default)]
    pub yanked: bool,
}
```

### ModDependency

A dependency on another mod.

```rust
use semver::VersionReq;

/// Dependency on another mod with version constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    /// Mod ID of the dependency
    pub id: ModId,

    /// SemVer version requirement (e.g., "^1.0", ">=2.0, <3.0")
    pub version_req: VersionReq,

    /// Whether this dependency is optional
    #[serde(default)]
    pub optional: bool,
}
```

### EngineConstraint

Engine version compatibility.

```rust
/// Engine version constraints
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EngineConstraint {
    /// Minimum engine version (inclusive)
    #[serde(default)]
    pub min: Option<Version>,

    /// Maximum engine version (inclusive)
    #[serde(default)]
    pub max: Option<Version>,
}

impl EngineConstraint {
    pub fn is_compatible(&self, engine_version: &Version) -> bool {
        if let Some(ref min) = self.min {
            if engine_version < min {
                return false;
            }
        }
        if let Some(ref max) = self.max {
            if engine_version > max {
                return false;
            }
        }
        true
    }
}
```

---

## Registry Types

### RegistryIndex

The main registry catalog.

```rust
/// Registry index containing all available mods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryIndex {
    /// Schema version for forward compatibility
    pub registry_version: u32,

    /// Registry name for display
    pub name: String,

    /// Registry base URL (for relative download URLs)
    #[serde(default)]
    pub base_url: Option<String>,

    /// Last update timestamp (RFC 3339)
    #[serde(default)]
    pub updated_at: Option<String>,

    /// All mods in this registry
    pub mods: Vec<RegistryMod>,
}
```

### RegistryMod

A mod entry in the registry.

```rust
/// A mod in the registry with all its versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMod {
    /// Mod ID
    pub id: ModId,

    /// Display name
    pub name: String,

    /// Short description
    #[serde(default)]
    pub description: Option<String>,

    /// Author name
    #[serde(default)]
    pub author: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// All available versions (newest first recommended)
    pub versions: Vec<ModVersion>,
}
```

---

## Configuration Types

### DistributionConfig

Server mod configuration from `server_mods.toml`.

```rust
/// Server mod distribution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionConfig {
    /// Configured registries in priority order
    pub registries: Vec<RegistryConfig>,

    /// Required mods with version constraints
    pub mods: Vec<ModRequirement>,

    /// Trust policy for signatures
    #[serde(default)]
    pub trust: TrustPolicy,

    /// Download settings
    #[serde(default)]
    pub download: DownloadSettings,

    /// Cache settings
    #[serde(default)]
    pub cache: CacheSettings,
}
```

### RegistryConfig

A single registry source.

```rust
/// Registry source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Registry name (for logging/errors)
    pub name: String,

    /// Registry URL or local path
    /// - HTTP(S) URL: "https://mods.example.com/index.json"
    /// - Local path: "file:///path/to/registry" or "/path/to/registry"
    pub url: String,

    /// Priority (lower = higher priority, default 100)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Whether this registry is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_priority() -> u32 { 100 }
fn default_true() -> bool { true }
```

### ModRequirement

A required mod from config.

```rust
/// A mod requirement from server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModRequirement {
    /// Mod ID
    pub id: ModId,

    /// Version constraint (SemVer)
    pub version: VersionReq,

    /// Pin to exact version (ignores newer compatible versions)
    #[serde(default)]
    pub pinned: bool,

    /// Whether this mod is optional
    #[serde(default)]
    pub optional: bool,
}
```

### TrustPolicy

Signature verification settings.

```rust
/// Trust policy for mod signatures
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Require valid signatures on all mods
    #[serde(default)]
    pub require_signature: bool,

    /// Allowed signing key IDs (hex-encoded public keys)
    /// Empty = allow any valid signature
    #[serde(default)]
    pub allowed_keys: Vec<String>,
}
```

### DownloadSettings

Network configuration.

```rust
/// Download behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSettings {
    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Read timeout in seconds
    #[serde(default = "default_read_timeout")]
    pub read_timeout_secs: u64,

    /// Maximum retry attempts
    #[serde(default = "default_retries")]
    pub retries: u32,

    /// Maximum bundle size in bytes
    #[serde(default = "default_max_size")]
    pub max_bundle_size: u64,
}

fn default_connect_timeout() -> u64 { 30 }
fn default_read_timeout() -> u64 { 120 }
fn default_retries() -> u32 { 3 }
fn default_max_size() -> u64 { 50 * 1024 * 1024 } // 50 MB
```

### CacheSettings

Cache directory configuration.

```rust
/// Cache directory settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSettings {
    /// Cache directory path (default: ~/.local/share/plix/mods/)
    #[serde(default)]
    pub path: Option<PathBuf>,

    /// Maximum cache size in bytes (0 = unlimited)
    #[serde(default)]
    pub max_size: u64,
}
```

---

## Lockfile Types

### Lockfile

The `mods.lock` format.

```rust
/// Lockfile for reproducible mod installations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    /// Lockfile schema version
    pub version: u32,

    /// Generation timestamp (RFC 3339)
    pub generated_at: String,

    /// Engine version used for compatibility check
    pub engine_version: String,

    /// Locked mod entries
    pub mods: Vec<LockedMod>,
}
```

### LockedMod

A locked mod entry.

```rust
/// A mod locked to a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedMod {
    /// Mod ID
    pub id: ModId,

    /// Exact version
    pub version: Version,

    /// SHA-256 hash for integrity
    pub sha256: String,

    /// Source registry name
    pub source: String,

    /// Download URL (resolved)
    pub download_url: String,

    /// Resolved dependencies (mod IDs only, versions in their own entries)
    pub dependencies: Vec<ModId>,
}
```

---

## Bundle Types

### ModBundle

In-memory representation of a `.plixmod` bundle.

```rust
/// A mod bundle in memory
#[derive(Debug)]
pub struct ModBundle {
    /// Parsed manifest (from mod.toml)
    pub manifest: ModManifest,

    /// WASM module bytes (if present)
    pub wasm: Option<Vec<u8>>,

    /// Asset files: path -> contents
    pub assets: HashMap<String, Vec<u8>>,

    /// Raw bundle bytes for hash verification
    pub raw_bytes: Vec<u8>,

    /// Computed SHA-256 hash
    pub sha256: String,
}
```

### BundleMetadata

Metadata extracted from bundle without full extraction.

```rust
/// Quick metadata extraction without full bundle load
#[derive(Debug, Clone)]
pub struct BundleMetadata {
    /// Mod ID from manifest
    pub id: ModId,

    /// Version from manifest
    pub version: Version,

    /// Has WASM module
    pub has_wasm: bool,

    /// Total uncompressed size
    pub uncompressed_size: u64,

    /// File count
    pub file_count: usize,
}
```

---

## Resolution Types

### ResolvedGraph

Result of dependency resolution.

```rust
/// Complete resolution result
#[derive(Debug, Clone)]
pub struct ResolvedGraph {
    /// Resolved mods in installation order (dependencies first)
    pub mods: Vec<ResolvedMod>,

    /// Resolution statistics
    pub stats: ResolutionStats,
}

/// A single resolved mod
#[derive(Debug, Clone)]
pub struct ResolvedMod {
    /// Mod ID
    pub id: ModId,

    /// Resolved version
    pub version: Version,

    /// Source registry
    pub source: String,

    /// Download URL
    pub download_url: String,

    /// SHA-256 hash
    pub sha256: String,

    /// Bundle size in bytes
    pub size: u64,

    /// Direct dependencies (resolved)
    pub dependencies: Vec<ModId>,
}

/// Resolution statistics
#[derive(Debug, Clone, Default)]
pub struct ResolutionStats {
    /// Total mods resolved
    pub total_mods: usize,

    /// Total download size (bytes)
    pub total_size: u64,

    /// Mods already cached
    pub cached_count: usize,

    /// Resolution time (milliseconds)
    pub resolution_time_ms: u64,
}
```

---

## Error Types

### DistributionError

Structured error with codes.

```rust
/// Distribution error with structured code
#[derive(Debug, Clone)]
pub struct DistributionError {
    /// Error code (EMREG001-008)
    pub code: ErrorCode,

    /// Human-readable message
    pub message: String,

    /// Affected mod (if applicable)
    pub mod_id: Option<ModId>,

    /// Additional context
    pub context: HashMap<String, String>,
}

/// Error codes per spec
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// EMREG001: Registry unreachable
    RegistryUnreachable,

    /// EMREG002: Invalid registry index
    InvalidIndex,

    /// EMREG003: Download failed
    DownloadFailed,

    /// EMREG004: Hash mismatch
    HashMismatch,

    /// EMREG005: Signature invalid
    SignatureInvalid,

    /// EMREG006: Dependency conflict
    DependencyConflict,

    /// EMREG007: Version incompatible
    VersionIncompatible,

    /// EMREG008: Dependency cycle detected
    CycleDetected,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::RegistryUnreachable => "EMREG001",
            Self::InvalidIndex => "EMREG002",
            Self::DownloadFailed => "EMREG003",
            Self::HashMismatch => "EMREG004",
            Self::SignatureInvalid => "EMREG005",
            Self::DependencyConflict => "EMREG006",
            Self::VersionIncompatible => "EMREG007",
            Self::CycleDetected => "EMREG008",
        }
    }
}
```

---

## File System Layout

### Cache Directory

```
~/.local/share/plix/mods/
├── bundles/                        # Content-addressed bundles
│   ├── abc123...def.plixmod       # SHA-256 hash as filename
│   └── fed321...cba.plixmod
├── installed/                      # Extracted mods
│   ├── my-mod/
│   │   └── 1.0.0/
│   │       ├── mod.toml
│   │       ├── mod.wasm
│   │       └── assets/
│   └── other-mod/
│       ├── 2.0.0/
│       └── 2.1.0/
└── indexes/                        # Cached registry indexes
    ├── registry1_hash.json
    └── registry2_hash.json
```

### Server Directory

```
server/
├── server_mods.toml               # Mod configuration
├── mods.lock                       # Lockfile
└── mods/                           # (Optional) Local registry
    ├── index.json
    └── bundles/
        └── local-mod-1.0.0.plixmod
```

---

## Relationships

```
DistributionConfig
    │
    ├──► RegistryConfig[] ──► RegistryIndex ──► RegistryMod[] ──► ModVersion[]
    │                                                                   │
    ├──► ModRequirement[] ◄─────────────────────────────────────────────┘
    │           │
    │           ▼
    │       Resolver
    │           │
    │           ▼
    ├──► ResolvedGraph ──► ResolvedMod[]
    │           │
    │           ▼
    ├──► Lockfile ──► LockedMod[]
    │
    └──► TrustPolicy ──► Signature Verification
```

---

## Validation Rules

1. **ModId**: `^[a-z0-9][a-z0-9-]*[a-z0-9]$`, max 64 chars
2. **SHA-256**: Exactly 64 hex characters
3. **Signature**: Exactly 128 hex characters (if present)
4. **Version**: Valid SemVer (major.minor.patch[-prerelease][+build])
5. **VersionReq**: Valid SemVer requirement expression
6. **URL**: Valid HTTP(S) URL or `file://` path
7. **Timestamp**: RFC 3339 format
