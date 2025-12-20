//! Migration Framework for Plix
//!
//! Provides infrastructure for migrating configuration files, save data,
//! and other versioned data between Plix versions.
//!
//! # Architecture
//!
//! - Migrations are versioned and run sequentially (N → N+1)
//! - Each migration implements the `Migration` trait
//! - `MigrationEngine` orchestrates running migrations in order
//! - Automatic backups are created before each migration
//!
//! # Example
//!
//! ```ignore
//! let engine = MigrationEngine::new(backup_dir);
//! engine.register(Box::new(V0ToV1ConfigMigration));
//! engine.run_migrations(&config_path, current_version)?;
//! ```

pub mod backup;
pub mod config;
pub mod save;

use std::path::Path;
use thiserror::Error;

pub use backup::{create_backup, list_backups, restore_backup, rotate_backups, Backup, BackupError, MAX_BACKUPS};

/// Semantic version for migration tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MigrationVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl MigrationVersion {
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self { major, minor, patch }
    }

    /// Version 0.0.0 - represents unversioned/legacy data
    pub const ZERO: Self = Self::new(0, 0, 0);

    /// Version 1.0.0 - first stable release
    pub const V1_0_0: Self = Self::new(1, 0, 0);
}

impl std::fmt::Display for MigrationVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Errors that can occur during migration.
#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("Backup failed: {0}")]
    Backup(#[from] BackupError),

    #[error("Migration from v{from} to v{to} failed: {reason}")]
    MigrationFailed {
        from: MigrationVersion,
        to: MigrationVersion,
        reason: String,
    },

    #[error("No migration path from v{from} to v{to}")]
    NoMigrationPath {
        from: MigrationVersion,
        to: MigrationVersion,
    },

    #[error("File not found: {0}")]
    FileNotFound(std::path::PathBuf),

    #[error("Failed to read file: {0}")]
    ReadError(#[source] std::io::Error),

    #[error("Failed to write file: {0}")]
    WriteError(#[source] std::io::Error),

    #[error("Failed to parse file: {0}")]
    ParseError(String),

    #[error("Data is already at target version v{0}")]
    AlreadyAtVersion(MigrationVersion),

    #[error("Data version v{data} is newer than engine version v{engine}")]
    NewerVersion {
        data: MigrationVersion,
        engine: MigrationVersion,
    },
}

/// Result type for migration operations.
pub type MigrationResult<T> = Result<T, MigrationError>;

/// Trait for implementing version migrations.
///
/// Each migration transforms data from one version to the next.
/// Migrations should be idempotent where possible.
pub trait Migration: Send + Sync {
    /// The version this migration upgrades from.
    fn from_version(&self) -> MigrationVersion;

    /// The version this migration upgrades to.
    fn to_version(&self) -> MigrationVersion;

    /// Human-readable description of what this migration does.
    fn description(&self) -> &str;

    /// Execute the migration on the given file.
    ///
    /// The file content is read before calling this method.
    /// Returns the migrated content to be written back.
    fn migrate(&self, content: &[u8]) -> MigrationResult<Vec<u8>>;

    /// Check if migration is needed (optional optimization).
    ///
    /// Returns true if the content needs migration.
    /// Default implementation always returns true.
    fn needs_migration(&self, _content: &[u8]) -> bool {
        true
    }
}

/// Report of a completed migration run.
#[derive(Debug, Clone)]
pub struct MigrationReport {
    /// Starting version before migrations
    pub from_version: MigrationVersion,
    /// Final version after migrations
    pub to_version: MigrationVersion,
    /// Number of migrations applied
    pub migrations_applied: usize,
    /// Backup created before migration (if any)
    pub backup: Option<Backup>,
    /// Whether this was a dry run
    pub dry_run: bool,
}

/// Engine for orchestrating migrations.
///
/// Manages a registry of migrations and runs them in sequence.
pub struct MigrationEngine {
    migrations: Vec<Box<dyn Migration>>,
    backup_dir: std::path::PathBuf,
}

impl MigrationEngine {
    /// Create a new migration engine.
    ///
    /// # Arguments
    ///
    /// * `backup_dir` - Directory where backups will be stored
    pub fn new(backup_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            migrations: Vec::new(),
            backup_dir: backup_dir.into(),
        }
    }

    /// Register a migration with the engine.
    ///
    /// Migrations should be registered in order (oldest to newest).
    pub fn register(&mut self, migration: Box<dyn Migration>) {
        self.migrations.push(migration);
    }

    /// Get the target version (highest to_version of registered migrations).
    pub fn target_version(&self) -> MigrationVersion {
        self.migrations
            .iter()
            .map(|m| m.to_version())
            .max()
            .unwrap_or(MigrationVersion::ZERO)
    }

    /// Find migrations needed to go from `from` version to `to` version.
    fn find_migration_path(
        &self,
        from: MigrationVersion,
        to: MigrationVersion,
    ) -> Vec<&dyn Migration> {
        let mut path = Vec::new();
        let mut current = from;

        while current < to {
            // Find a migration that starts at current version
            let next = self
                .migrations
                .iter()
                .find(|m| m.from_version() == current && m.to_version() <= to);

            match next {
                Some(migration) => {
                    current = migration.to_version();
                    path.push(migration.as_ref());
                }
                None => break, // No more migrations available
            }
        }

        path
    }

    /// Run migrations on a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file to migrate
    /// * `current_version` - Current version of the file's data
    ///
    /// # Returns
    ///
    /// A report of the migrations applied.
    pub fn run_migrations(
        &self,
        file_path: &Path,
        current_version: MigrationVersion,
    ) -> MigrationResult<MigrationReport> {
        self.run_migrations_internal(file_path, current_version, false)
    }

    /// Run migrations in dry-run mode (no changes written).
    ///
    /// Returns what would happen without making changes.
    pub fn run_migrations_dry_run(
        &self,
        file_path: &Path,
        current_version: MigrationVersion,
    ) -> MigrationResult<MigrationReport> {
        self.run_migrations_internal(file_path, current_version, true)
    }

    fn run_migrations_internal(
        &self,
        file_path: &Path,
        current_version: MigrationVersion,
        dry_run: bool,
    ) -> MigrationResult<MigrationReport> {
        let target = self.target_version();

        // Check if already at target version
        if current_version >= target {
            if current_version == target {
                return Err(MigrationError::AlreadyAtVersion(current_version));
            } else {
                return Err(MigrationError::NewerVersion {
                    data: current_version,
                    engine: target,
                });
            }
        }

        // Check file exists
        if !file_path.exists() {
            return Err(MigrationError::FileNotFound(file_path.to_path_buf()));
        }

        // Find migration path
        let path = self.find_migration_path(current_version, target);
        if path.is_empty() {
            return Err(MigrationError::NoMigrationPath {
                from: current_version,
                to: target,
            });
        }

        // Log migration plan
        tracing::info!(
            file = %file_path.display(),
            from = %current_version,
            to = %target,
            steps = path.len(),
            dry_run = dry_run,
            "Starting migration"
        );

        for migration in &path {
            tracing::debug!(
                from = %migration.from_version(),
                to = %migration.to_version(),
                description = migration.description(),
                "Migration step"
            );
        }

        // Create backup (unless dry run)
        let backup = if !dry_run {
            let backup = create_backup(file_path, &self.backup_dir)?;

            // Rotate old backups
            if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
                let _ = rotate_backups(&self.backup_dir, filename, MAX_BACKUPS);
            }

            Some(backup)
        } else {
            None
        };

        // Read current content
        let mut content = std::fs::read(file_path).map_err(MigrationError::ReadError)?;

        // Apply migrations sequentially
        let mut final_version = current_version;
        let mut applied = 0;

        for migration in &path {
            if !migration.needs_migration(&content) {
                tracing::debug!(
                    from = %migration.from_version(),
                    to = %migration.to_version(),
                    "Skipping migration (not needed)"
                );
                continue;
            }

            tracing::info!(
                from = %migration.from_version(),
                to = %migration.to_version(),
                description = migration.description(),
                "Applying migration"
            );

            content = migration.migrate(&content).map_err(|e| {
                tracing::error!(
                    from = %migration.from_version(),
                    to = %migration.to_version(),
                    error = %e,
                    "Migration failed"
                );
                e
            })?;

            final_version = migration.to_version();
            applied += 1;
        }

        // Write migrated content (unless dry run)
        if !dry_run && applied > 0 {
            std::fs::write(file_path, &content).map_err(MigrationError::WriteError)?;

            tracing::info!(
                file = %file_path.display(),
                from = %current_version,
                to = %final_version,
                migrations = applied,
                "Migration completed successfully"
            );
        }

        Ok(MigrationReport {
            from_version: current_version,
            to_version: final_version,
            migrations_applied: applied,
            backup,
            dry_run,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Simple test migration that adds a version field
    struct TestMigrationV0ToV1;

    impl Migration for TestMigrationV0ToV1 {
        fn from_version(&self) -> MigrationVersion {
            MigrationVersion::ZERO
        }

        fn to_version(&self) -> MigrationVersion {
            MigrationVersion::V1_0_0
        }

        fn description(&self) -> &str {
            "Add version field to config"
        }

        fn migrate(&self, content: &[u8]) -> MigrationResult<Vec<u8>> {
            let mut text = String::from_utf8_lossy(content).to_string();
            if !text.contains("config_version") {
                text.push_str("\nconfig_version = \"1.0.0\"\n");
            }
            Ok(text.into_bytes())
        }
    }

    #[test]
    fn test_migration_version_ordering() {
        let v0 = MigrationVersion::ZERO;
        let v1 = MigrationVersion::V1_0_0;
        let v1_1 = MigrationVersion::new(1, 1, 0);
        let v2 = MigrationVersion::new(2, 0, 0);

        assert!(v0 < v1);
        assert!(v1 < v1_1);
        assert!(v1_1 < v2);
    }

    #[test]
    fn test_migration_version_display() {
        assert_eq!(MigrationVersion::ZERO.to_string(), "0.0.0");
        assert_eq!(MigrationVersion::V1_0_0.to_string(), "1.0.0");
        assert_eq!(MigrationVersion::new(2, 3, 4).to_string(), "2.3.4");
    }

    #[test]
    fn test_migration_engine_register() {
        let temp = TempDir::new().unwrap();
        let mut engine = MigrationEngine::new(temp.path().join("backups"));

        engine.register(Box::new(TestMigrationV0ToV1));

        assert_eq!(engine.target_version(), MigrationVersion::V1_0_0);
    }

    #[test]
    fn test_migration_engine_find_path() {
        let temp = TempDir::new().unwrap();
        let mut engine = MigrationEngine::new(temp.path().join("backups"));

        engine.register(Box::new(TestMigrationV0ToV1));

        let path = engine.find_migration_path(MigrationVersion::ZERO, MigrationVersion::V1_0_0);
        assert_eq!(path.len(), 1);
        assert_eq!(path[0].from_version(), MigrationVersion::ZERO);
        assert_eq!(path[0].to_version(), MigrationVersion::V1_0_0);
    }

    #[test]
    fn test_migration_engine_run() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        let backup_dir = temp.path().join("backups");

        // Create initial config
        std::fs::write(&config_path, "name = \"test\"\n").unwrap();

        let mut engine = MigrationEngine::new(&backup_dir);
        engine.register(Box::new(TestMigrationV0ToV1));

        let report = engine
            .run_migrations(&config_path, MigrationVersion::ZERO)
            .unwrap();

        assert_eq!(report.from_version, MigrationVersion::ZERO);
        assert_eq!(report.to_version, MigrationVersion::V1_0_0);
        assert_eq!(report.migrations_applied, 1);
        assert!(report.backup.is_some());
        assert!(!report.dry_run);

        // Verify file was updated
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("config_version = \"1.0.0\""));

        // Verify backup exists
        let backups = list_backups(&backup_dir, "config.toml").unwrap();
        assert_eq!(backups.len(), 1);
    }

    #[test]
    fn test_migration_idempotence() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        let backup_dir = temp.path().join("backups");

        // Create initial config
        std::fs::write(&config_path, "name = \"test\"\n").unwrap();

        let mut engine = MigrationEngine::new(&backup_dir);
        engine.register(Box::new(TestMigrationV0ToV1));

        // Run migration first time
        let report1 = engine
            .run_migrations(&config_path, MigrationVersion::ZERO)
            .unwrap();
        let content1 = std::fs::read_to_string(&config_path).unwrap();

        // Running again should fail (already at target version)
        let result = engine.run_migrations(&config_path, MigrationVersion::V1_0_0);
        assert!(matches!(result, Err(MigrationError::AlreadyAtVersion(_))));

        // Content should be unchanged
        let content2 = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content1, content2);

        // Only one migration should have been applied
        assert_eq!(report1.migrations_applied, 1);
    }

    #[test]
    fn test_migration_dry_run() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");
        let backup_dir = temp.path().join("backups");

        // Create initial config
        let original = "name = \"test\"\n";
        std::fs::write(&config_path, original).unwrap();

        let mut engine = MigrationEngine::new(&backup_dir);
        engine.register(Box::new(TestMigrationV0ToV1));

        // Dry run
        let report = engine
            .run_migrations_dry_run(&config_path, MigrationVersion::ZERO)
            .unwrap();

        assert!(report.dry_run);
        assert_eq!(report.migrations_applied, 1);
        assert!(report.backup.is_none());

        // File should be unchanged
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, original);

        // No backups should exist
        assert!(!backup_dir.exists());
    }

    #[test]
    fn test_migration_file_not_found() {
        let temp = TempDir::new().unwrap();
        let mut engine = MigrationEngine::new(temp.path().join("backups"));
        engine.register(Box::new(TestMigrationV0ToV1));

        let result = engine.run_migrations(
            &temp.path().join("nonexistent.toml"),
            MigrationVersion::ZERO,
        );

        assert!(matches!(result, Err(MigrationError::FileNotFound(_))));
    }

    #[test]
    fn test_migration_no_path() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.toml");

        std::fs::write(&config_path, "test").unwrap();

        // Engine with no migrations
        let engine = MigrationEngine::new(temp.path().join("backups"));

        let result = engine.run_migrations(&config_path, MigrationVersion::ZERO);

        assert!(matches!(result, Err(MigrationError::NoMigrationPath { .. })));
    }
}
