# Quickstart: 1.0 Release Implementation

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## Overview

This guide covers implementing the 1.0 release infrastructure in priority order.

## Phase 1: Version Infrastructure (Days 1-2)

### 1.1 Add Version Module

Create `crates/plix-common/src/version.rs`:

```rust
//! Version information and compatibility checking

use serde::{Deserialize, Serialize};

/// Protocol version for network compatibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

/// Current protocol version
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion {
    major: 1,
    minor: 0,
};

impl ProtocolVersion {
    pub fn is_compatible_with(&self, other: &ProtocolVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}
```

### 1.2 Add Mod API Version

Add to `crates/plix-mod-core/src/lib.rs`:

```rust
/// Mod API version for compatibility checking
pub const MOD_API_VERSION: (u8, u8) = (1, 0);
```

### 1.3 Add Content Schema Version

Add to `crates/plix-server/src/content/mod.rs`:

```rust
/// Content schema version
pub const CONTENT_SCHEMA_VERSION: (u8, u8) = (1, 0);
```

### 1.4 Update Cargo.toml

Change workspace version:

```toml
[workspace.package]
version = "1.0.0"
```

## Phase 2: Migration Framework (Days 3-5)

### 2.1 Create Migration Module

Create `crates/plix-common/src/migration/mod.rs`:

```rust
//! Data migration framework with automatic backup

mod backup;
mod config;

pub use backup::{create_backup, rotate_backups, Backup};
pub use config::ConfigMigrator;

/// Maximum number of backups to retain
pub const MAX_BACKUPS: usize = 3;
```

### 2.2 Implement Backup

Create `crates/plix-common/src/migration/backup.rs`:

```rust
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub struct Backup {
    pub path: PathBuf,
    pub original_path: PathBuf,
    pub created_at: chrono::DateTime<Utc>,
    pub size_bytes: u64,
    pub checksum: String,
}

pub fn create_backup(source: &Path, backup_dir: &Path) -> Result<Backup, std::io::Error> {
    fs::create_dir_all(backup_dir)?;

    let timestamp = Utc::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = source.file_name().unwrap().to_str().unwrap();
    let backup_name = format!("{}.bak.{}", filename, timestamp);
    let backup_path = backup_dir.join(&backup_name);

    fs::copy(source, &backup_path)?;

    let content = fs::read(&backup_path)?;
    let checksum = format!("{:x}", Sha256::digest(&content));

    Ok(Backup {
        path: backup_path,
        original_path: source.to_path_buf(),
        created_at: Utc::now(),
        size_bytes: content.len() as u64,
        checksum,
    })
}

pub fn rotate_backups(backup_dir: &Path, original_filename: &str, keep: usize) -> Result<(), std::io::Error> {
    let pattern = format!("{}.bak.", original_filename);
    let mut backups: Vec<_> = fs::read_dir(backup_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_str().map_or(false, |n| n.starts_with(&pattern)))
        .collect();

    backups.sort_by_key(|e| e.file_name());

    while backups.len() > keep {
        if let Some(oldest) = backups.first() {
            fs::remove_file(oldest.path())?;
            backups.remove(0);
        }
    }

    Ok(())
}
```

### 2.3 Implement Config Migrator

Create `crates/plix-common/src/migration/config.rs`:

```rust
use serde_json::Value;
use std::path::Path;

pub trait Migration {
    fn from_version(&self) -> u32;
    fn to_version(&self) -> u32;
    fn migrate(&self, data: &mut Value) -> Result<(), String>;
}

pub struct ConfigMigrator {
    migrations: Vec<Box<dyn Migration>>,
}

impl ConfigMigrator {
    pub fn new() -> Self {
        Self { migrations: Vec::new() }
    }

    pub fn register(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
        self.migrations.sort_by_key(|m| m.from_version());
    }

    pub fn migrate(&self, data: &mut Value, from: u32, to: u32) -> Result<(), String> {
        for migration in &self.migrations {
            if migration.from_version() >= from && migration.to_version() <= to {
                migration.migrate(data)?;
            }
        }
        Ok(())
    }
}
```

## Phase 3: Governance Files (Days 6-7)

### 3.1 Create LICENSE

```
MIT License

Copyright (c) 2025 Plix Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### 3.2 Create README.md Template

```markdown
# Plix

A multiplayer voxel game platform with first-class mod support.

## Features

- Multiplayer voxel world with real-time combat
- Server-authoritative architecture
- Full mod support (data, script, WASM)
- Cross-platform (Windows, Linux, macOS)

## Quick Start

### Players

1. Download the latest release
2. Extract and run `plix-client`
3. Connect to a server from the browser

### Server Operators

1. Download the headless server
2. Configure `server.toml`
3. Run `plix-server`

## Documentation

- [User Guide](docs/user/installation.md)
- [Server Admin Guide](docs/server/installation.md)
- [Modding Guide](docs/modding/getting-started.md)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE)
```

### 3.3 Create CONTRIBUTING.md Template

```markdown
# Contributing to Plix

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch
4. Make your changes
5. Submit a pull request

## Code Standards

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix all warnings
- Add tests for new functionality
- Update documentation as needed

## Commit Format

Use conventional commits:

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation only
- `refactor:` Code change that neither fixes nor adds
- `test:` Adding tests
- `chore:` Maintenance tasks

## Pull Request Process

1. Update the CHANGELOG if applicable
2. Ensure all tests pass
3. Request review from maintainers
4. Address feedback
5. Merge after approval
```

## Phase 4: Documentation (Days 8-12)

Create documentation files under `docs/`:

- `docs/user/installation.md`
- `docs/user/getting-started.md`
- `docs/server/installation.md`
- `docs/server/configuration.md`
- `docs/modding/getting-started.md`
- `docs/release/CHANGELOG.md`

## Phase 5: Release Automation (Days 13-14)

Update `.github/workflows/release.yml` to:

1. Verify GPG-signed tag
2. Build all platforms
3. Generate SHA-256 checksums
4. Create GitHub Release

## Verification Checklist

Run before tagging v1.0.0:

```bash
# Code quality
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all

# Version verification
grep "version = \"1.0.0\"" Cargo.toml

# Governance files
test -f LICENSE
test -f README.md
test -f CONTRIBUTING.md
test -f CODE_OF_CONDUCT.md

# Documentation
test -d docs/user
test -d docs/server
test -d docs/modding
```

## Summary

| Phase | Focus | Duration |
|-------|-------|----------|
| 1 | Version infrastructure | 2 days |
| 2 | Migration framework | 3 days |
| 3 | Governance files | 2 days |
| 4 | Documentation | 5 days |
| 5 | Release automation | 2 days |
| **Total** | | **14 days** |
