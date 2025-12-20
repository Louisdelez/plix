# Contract: Version API

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## Overview

Internal API for version information access across all components.

## Constants

### Protocol Version

```rust
// Location: plix-common/src/version.rs

/// Current protocol version for network compatibility
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion {
    major: 1,
    minor: 0,
};

impl ProtocolVersion {
    /// Check if this version is compatible with another
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}
```

### Mod API Version

```rust
// Location: plix-mod-core/src/lib.rs

/// Current mod API version
pub const MOD_API_VERSION: ModApiVersion = ModApiVersion {
    major: 1,
    minor: 0,
};

impl ModApiVersion {
    /// Check if a mod requiring this version can run on engine with given version
    pub fn can_run_on(&self, engine: &ModApiVersion) -> bool {
        self.major == engine.major && self.minor <= engine.minor
    }
}
```

### Content Schema Version

```rust
// Location: plix-server/src/content/schema.rs

/// Current content schema version
pub const CONTENT_SCHEMA_VERSION: ContentSchemaVersion = ContentSchemaVersion {
    major: 1,
    minor: 0,
};
```

## Functions

### Get Full Version String

```rust
// Location: plix-common/src/version.rs

/// Get full version string for display
/// Format: "1.0.0 (abc1234) built 2025-12-20"
pub fn get_version_string(build_info: &BuildInfo) -> String {
    build_info.display_version()
}
```

### Check Version Compatibility

```rust
// Location: plix-common/src/version.rs

/// Check if client version is compatible with server
pub fn check_client_server_compat(
    client_protocol: &ProtocolVersion,
    server_protocol: &ProtocolVersion,
) -> Result<(), VersionError> {
    if client_protocol.major != server_protocol.major {
        return Err(VersionError::MajorMismatch {
            client: client_protocol.major,
            server: server_protocol.major,
        });
    }
    if client_protocol.minor > server_protocol.minor {
        return Err(VersionError::ClientTooNew {
            client: client_protocol.minor,
            server: server_protocol.minor,
        });
    }
    Ok(())
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    #[error("Protocol major version mismatch: client {client}, server {server}")]
    MajorMismatch { client: u8, server: u8 },

    #[error("Client protocol version {client} is newer than server {server}")]
    ClientTooNew { client: u8, server: u8 },

    #[error("Mod requires engine {required}, but engine is {actual}")]
    ModIncompatible { required: ModApiVersion, actual: ModApiVersion },
}
```

## Usage Examples

### Client Startup

```rust
// Display version in client logs
let build_info = BuildInfo::from_shadow(...);
tracing::info!("Plix Client {}", build_info.display_version());
```

### Server Connection Validation

```rust
// Validate client protocol version
fn handle_connect(msg: ClientMessage::Connect { protocol_version, .. }) {
    check_client_server_compat(
        &ProtocolVersion { major: protocol_version, minor: 0 },
        &PROTOCOL_VERSION,
    )?;
}
```

### Mod Loading

```rust
// Check mod compatibility
fn load_mod(manifest: &ModManifest) -> Result<()> {
    if !manifest.required_engine.can_run_on(&MOD_API_VERSION) {
        return Err(ModLoadError::IncompatibleVersion {
            required: manifest.required_engine,
            actual: MOD_API_VERSION,
        });
    }
    // Proceed with loading
}
```
