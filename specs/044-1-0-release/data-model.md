# Data Model: 1.0 Release

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## Entities

### E1: VersionInfo

Central version information structure for the release.

```rust
/// Semantic version components
pub struct VersionInfo {
    /// Major version (breaking changes)
    pub major: u16,
    /// Minor version (backward-compatible features)
    pub minor: u16,
    /// Patch version (backward-compatible fixes)
    pub patch: u16,
    /// Pre-release identifier (e.g., "alpha", "beta", "rc.1")
    pub pre_release: Option<String>,
    /// Build metadata (e.g., commit hash)
    pub build_metadata: Option<String>,
}
```

**Relationships**:
- Embedded in BuildInfo (extends existing)
- Referenced by ProtocolVersion, ModApiVersion, ContentSchemaVersion

**Validation Rules**:
- major, minor, patch: non-negative integers
- pre_release: alphanumeric with dots/hyphens
- build_metadata: alphanumeric with dots/hyphens

---

### E2: ProtocolVersion

Network protocol version for client-server compatibility.

```rust
/// Protocol version for network compatibility
pub struct ProtocolVersion {
    /// Major version - must match for compatibility
    pub major: u8,
    /// Minor version - higher server supports lower client
    pub minor: u8,
}

/// Current protocol version constant
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion {
    major: 1,
    minor: 0,
};
```

**Relationships**:
- Sent in ClientMessage::Connect handshake
- Validated by server on connection

**Validation Rules**:
- Client major must equal server major
- Client minor must be <= server minor

**State Transitions**:
- N/A (immutable constant per build)

---

### E3: ModApiVersion

Mod API version for mod compatibility.

```rust
/// Mod API version for mod compatibility
pub struct ModApiVersion {
    /// Major version - breaking API changes
    pub major: u8,
    /// Minor version - additive API changes
    pub minor: u8,
}

/// Current mod API version constant
pub const MOD_API_VERSION: ModApiVersion = ModApiVersion {
    major: 1,
    minor: 0,
};
```

**Relationships**:
- Declared in mod manifest (required engine version)
- Checked by mod loader at startup

**Validation Rules**:
- Mod major must equal engine major
- Mod minor must be <= engine minor

---

### E4: ContentSchemaVersion

Content definition schema version.

```rust
/// Content schema version for content file compatibility
pub struct ContentSchemaVersion {
    /// Major version - breaking schema changes
    pub major: u8,
    /// Minor version - additive schema changes
    pub minor: u8,
}

/// Current content schema version
pub const CONTENT_SCHEMA_VERSION: ContentSchemaVersion = ContentSchemaVersion {
    major: 1,
    minor: 0,
};
```

**Relationships**:
- Validated at content registry load time
- Used by migration system for content upgrades

---

### E5: MigrationContext

Context for a migration operation.

```rust
/// Migration operation context
pub struct MigrationContext {
    /// Source version
    pub from_version: u32,
    /// Target version
    pub to_version: u32,
    /// Backup path created before migration
    pub backup_path: PathBuf,
    /// Timestamp of migration start
    pub started_at: DateTime<Utc>,
    /// Whether this is a dry run
    pub dry_run: bool,
}
```

**Relationships**:
- Created by migration engine
- Contains reference to Backup

---

### E6: Backup

Backup metadata for migration safety.

```rust
/// Backup metadata
pub struct Backup {
    /// Path to backup file
    pub path: PathBuf,
    /// Original file path
    pub original_path: PathBuf,
    /// Timestamp of backup creation
    pub created_at: DateTime<Utc>,
    /// Size in bytes
    pub size_bytes: u64,
    /// SHA-256 hash of backup contents
    pub checksum: String,
}
```

**Relationships**:
- Created before each migration
- Managed by backup rotation (keep 3 most recent)

**Validation Rules**:
- path must exist after creation
- checksum must match file contents

---

### E7: MigrationResult

Result of a migration operation.

```rust
/// Result of a migration operation
pub enum MigrationResult {
    /// Migration completed successfully
    Success {
        from_version: u32,
        to_version: u32,
        changes: Vec<MigrationChange>,
        backup: Backup,
    },
    /// Migration failed, data unchanged
    Failed {
        from_version: u32,
        to_version: u32,
        error: MigrationError,
        backup: Backup,
    },
    /// No migration needed (already at target version)
    NoOp {
        current_version: u32,
    },
}
```

**State Transitions**:
```
[Start] → Backing Up → Migrating → Success
                    ↘ Failed (rollback)
```

---

### E8: ConfigVersion

Version tracking for configuration files.

```rust
/// Configuration file with version tracking
pub struct VersionedConfig<T> {
    /// Configuration version for migration tracking
    pub config_version: u32,
    /// The actual configuration data
    #[serde(flatten)]
    pub config: T,
}
```

**Relationships**:
- Wraps existing config types (GameConfig, ServerConfig, LauncherConfig)
- Read/written by config migration engine

**Validation Rules**:
- config_version: positive integer, monotonically increasing

---

### E9: StabilityStatus

API stability marker for mod documentation.

```rust
/// Stability status for public APIs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilityStatus {
    /// Stable - guaranteed for major version lifetime
    Stable,
    /// Experimental - may change in minor versions
    Experimental,
    /// Deprecated - will be removed in next major version
    Deprecated { since: &'static str, replacement: Option<&'static str> },
}
```

**Relationships**:
- Applied to mod API functions/types via attributes
- Extracted for documentation generation

---

### E10: ReleaseArtifact

Metadata for release distribution files.

```rust
/// Release artifact metadata
pub struct ReleaseArtifact {
    /// Artifact name (e.g., "plix-client-linux-x86_64.tar.gz")
    pub name: String,
    /// Target platform
    pub platform: Platform,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// File size in bytes
    pub size_bytes: u64,
    /// SHA-256 checksum
    pub sha256: String,
    /// Download URL
    pub url: String,
}

pub enum Platform {
    LinuxX64,
    WindowsX64,
    MacOSX64,
    MacOSArm64,
}

pub enum ArtifactType {
    Client,
    Server,
    Checksums,
}
```

**Relationships**:
- Listed in release manifest
- Downloaded by launcher

---

## Entity Relationships

```
VersionInfo (1.0.0)
    ├── ProtocolVersion (1.0)
    ├── ModApiVersion (1.0)
    └── ContentSchemaVersion (1.0)

MigrationContext
    ├── from_version (u32)
    ├── to_version (u32)
    └── Backup (1:1)

VersionedConfig<T>
    └── config_version → MigrationContext (triggers migration)

ReleaseArtifact
    └── sha256 (checksum for verification)
```

## Version Compatibility Matrix

| Source Version | Target Version | Compatible | Notes |
|---------------|----------------|------------|-------|
| Client 1.x | Server 1.x | Yes | Same major version |
| Client 1.x | Server 2.x | No | Major version mismatch |
| Mod 1.0 | Engine 1.5 | Yes | Minor upgrade OK |
| Mod 1.5 | Engine 1.0 | No | Mod requires newer engine |
| Content 1.x | Engine 1.x | Yes | Same major version |

## Data Locations

| Data Type | Location | Format |
|-----------|----------|--------|
| Client config | ~/.config/plix/config.toml | TOML |
| Server config | ./server.toml | TOML |
| Player saves | ~/.local/share/plix/worlds/ | Binary (bincode) |
| Content defs | ./assets/content/ | TOML |
| Backups | Adjacent to original (*.bak.*) | Same as original |
