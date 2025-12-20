# Contract: Migration API

**Feature**: 044-1-0-release | **Date**: 2025-12-20

## Overview

Internal API for data migration with automatic backup.

## Core Trait

```rust
// Location: plix-common/src/migration/mod.rs

/// A single version-to-version migration step
pub trait Migration: Send + Sync {
    /// Source version this migration upgrades from
    fn from_version(&self) -> u32;

    /// Target version this migration upgrades to
    fn to_version(&self) -> u32;

    /// Apply the migration to the data
    /// Returns list of changes made for logging
    fn migrate(&self, data: &mut serde_json::Value) -> Result<Vec<MigrationChange>, MigrationError>;

    /// Human-readable description of what this migration does
    fn description(&self) -> &str;
}

/// A change made during migration (for logging)
#[derive(Debug, Clone)]
pub struct MigrationChange {
    pub path: String,
    pub change_type: ChangeType,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Renamed { from: String },
}
```

## Migration Engine

```rust
// Location: plix-common/src/migration/mod.rs

/// Migration engine for a specific data type
pub struct MigrationEngine {
    migrations: Vec<Box<dyn Migration>>,
    backup_dir: PathBuf,
    max_backups: usize, // Default: 3
}

impl MigrationEngine {
    /// Create a new migration engine
    pub fn new(backup_dir: PathBuf) -> Self;

    /// Register a migration step
    pub fn register(&mut self, migration: Box<dyn Migration>);

    /// Run all necessary migrations from current to target version
    pub fn migrate(
        &self,
        data_path: &Path,
        current_version: u32,
        target_version: u32,
        dry_run: bool,
    ) -> Result<MigrationResult, MigrationError>;

    /// List available backups for a file
    pub fn list_backups(&self, original_path: &Path) -> Vec<Backup>;

    /// Restore from a specific backup
    pub fn restore_backup(&self, backup: &Backup) -> Result<(), MigrationError>;
}
```

## Backup API

```rust
// Location: plix-common/src/migration/backup.rs

/// Create a backup of a file before migration
pub fn create_backup(
    source_path: &Path,
    backup_dir: &Path,
) -> Result<Backup, BackupError>;

/// Rotate backups, keeping only the N most recent
pub fn rotate_backups(
    backup_dir: &Path,
    original_filename: &str,
    keep_count: usize, // Default: 3
) -> Result<Vec<PathBuf>, BackupError>; // Returns deleted paths

/// Generate backup filename with timestamp
pub fn backup_filename(original: &str) -> String {
    // Format: "{original}.bak.{ISO8601}"
    // Example: "config.toml.bak.2025-12-20T14-30-00"
}
```

## Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MigrationError {
    #[error("No migration path from version {from} to {to}")]
    NoMigrationPath { from: u32, to: u32 },

    #[error("Migration from {from} to {to} failed: {reason}")]
    MigrationFailed { from: u32, to: u32, reason: String },

    #[error("Backup creation failed: {0}")]
    BackupFailed(#[from] BackupError),

    #[error("Data corruption detected: {0}")]
    CorruptData(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Failed to create backup directory: {0}")]
    CreateDir(std::io::Error),

    #[error("Failed to copy file: {0}")]
    Copy(std::io::Error),

    #[error("Checksum mismatch after backup")]
    ChecksumMismatch,
}
```

## Config Migration Example

```rust
// Location: plix-common/src/migration/config.rs

/// Migration: config v1 → v2 (add new field with default)
pub struct ConfigMigrationV1ToV2;

impl Migration for ConfigMigrationV1ToV2 {
    fn from_version(&self) -> u32 { 1 }
    fn to_version(&self) -> u32 { 2 }

    fn migrate(&self, data: &mut serde_json::Value) -> Result<Vec<MigrationChange>, MigrationError> {
        let mut changes = Vec::new();

        // Add new field with default if missing
        if data.get("new_setting").is_none() {
            data["new_setting"] = serde_json::json!("default_value");
            changes.push(MigrationChange {
                path: "new_setting".to_string(),
                change_type: ChangeType::Added,
                old_value: None,
                new_value: Some("default_value".to_string()),
            });
        }

        // Update version
        data["config_version"] = serde_json::json!(2);

        Ok(changes)
    }

    fn description(&self) -> &str {
        "Add new_setting field with default value"
    }
}
```

## Usage: Server Startup

```rust
// Location: plix-server/src/main.rs

fn run_migrations() -> Result<(), MigrationError> {
    let engine = MigrationEngine::new(PathBuf::from("./backups"));

    // Register all migrations
    engine.register(Box::new(ConfigMigrationV1ToV2));
    // ... more migrations

    // Load config, get current version
    let config_path = Path::new("./server.toml");
    let config: VersionedConfig<ServerConfig> = load_config(config_path)?;

    // Run migrations if needed
    let result = engine.migrate(
        config_path,
        config.config_version,
        CURRENT_CONFIG_VERSION,
        false, // not dry run
    )?;

    match result {
        MigrationResult::Success { changes, backup, .. } => {
            tracing::info!("Migration complete, {} changes", changes.len());
            tracing::info!("Backup created: {}", backup.path.display());
        }
        MigrationResult::NoOp { .. } => {
            tracing::debug!("No migration needed");
        }
        MigrationResult::Failed { error, backup, .. } => {
            tracing::error!("Migration failed: {}", error);
            tracing::info!("Restore from: {}", backup.path.display());
            return Err(error);
        }
    }

    Ok(())
}
```

## CLI Flags

```rust
// Server CLI options for migration
#[derive(clap::Args)]
pub struct MigrationArgs {
    /// Run migration without applying changes
    #[arg(long)]
    pub migrate_dry_run: bool,

    /// Restore from a backup file
    #[arg(long)]
    pub restore_backup: Option<PathBuf>,

    /// List available backups
    #[arg(long)]
    pub list_backups: bool,
}
```
