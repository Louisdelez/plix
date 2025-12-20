//! Integration tests for server migration functionality.

use std::fs;
use std::path::Path;
use tempfile::TempDir;

use plix_common::migration::{
    config::{detect_config_version, V0ToV1ConfigMigration},
    list_backups, MigrationEngine, MigrationVersion, MAX_BACKUPS,
};

/// Create a v0 config fixture (no config_version field)
fn create_v0_config(path: &Path) {
    let content = r#"
[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "test_arena"

[server]
name = "Test Server"
motd = "Welcome!"

[logging]
level = "info"
"#;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

/// Create a v1 config fixture (has config_version field)
fn create_v1_config(path: &Path) {
    let content = r#"config_version = "1.0.0"

[network]
bind_address = "0.0.0.0"
port = 7777
max_players = 32

[game]
tick_rate = 30
arena = "test_arena"
"#;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn test_detect_v0_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("server.toml");
    create_v0_config(&config_path);

    let content = fs::read(&config_path).unwrap();
    let version = detect_config_version(&content);

    assert_eq!(version, MigrationVersion::ZERO);
}

#[test]
fn test_detect_v1_config() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("server.toml");
    create_v1_config(&config_path);

    let content = fs::read(&config_path).unwrap();
    let version = detect_config_version(&content);

    assert_eq!(version, MigrationVersion::V1_0_0);
}

#[test]
fn test_dry_run_no_changes() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config").join("server.toml");
    let backup_dir = temp.path().join("backups");

    create_v0_config(&config_path);
    let original_content = fs::read_to_string(&config_path).unwrap();

    // Set up migration engine
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));

    // Run dry-run
    let report = engine
        .run_migrations_dry_run(&config_path, MigrationVersion::ZERO)
        .unwrap();

    // Verify dry-run results
    assert!(report.dry_run);
    assert_eq!(report.migrations_applied, 1);
    assert!(report.backup.is_none()); // No backup in dry-run

    // Verify file is unchanged
    let new_content = fs::read_to_string(&config_path).unwrap();
    assert_eq!(original_content, new_content);

    // Verify no backup was created
    assert!(!backup_dir.exists());
}

#[test]
fn test_migrate_v0_to_v1() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config").join("server.toml");
    let backup_dir = temp.path().join("backups");

    create_v0_config(&config_path);

    // Set up migration engine
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));

    // Run migration
    let report = engine
        .run_migrations(&config_path, MigrationVersion::ZERO)
        .unwrap();

    // Verify migration results
    assert!(!report.dry_run);
    assert_eq!(report.from_version, MigrationVersion::ZERO);
    assert_eq!(report.to_version, MigrationVersion::V1_0_0);
    assert_eq!(report.migrations_applied, 1);
    assert!(report.backup.is_some());

    // Verify file was migrated
    let content = fs::read(&config_path).unwrap();
    let new_version = detect_config_version(&content);
    assert_eq!(new_version, MigrationVersion::V1_0_0);

    // Verify content preserved
    let text = fs::read_to_string(&config_path).unwrap();
    assert!(text.contains("config_version = \"1.0.0\""));
    assert!(text.contains("port = 7777"));
    assert!(text.contains("max_players = 32"));
    assert!(text.contains("arena = \"test_arena\""));
}

#[test]
fn test_migration_creates_backup_with_checksum() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config").join("server.toml");
    let backup_dir = temp.path().join("backups");

    create_v0_config(&config_path);

    // Set up migration engine
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));

    // Run migration
    let report = engine
        .run_migrations(&config_path, MigrationVersion::ZERO)
        .unwrap();

    // Verify backup was created
    let backup = report.backup.unwrap();
    assert!(backup.path.exists());
    assert!(!backup.checksum.is_empty());
    assert_eq!(backup.checksum.len(), 64); // SHA-256 hex = 64 chars
}

#[test]
fn test_backup_rotation_keeps_three() {
    let temp = TempDir::new().unwrap();
    let backup_dir = temp.path().join("backups");
    fs::create_dir_all(&backup_dir).unwrap();

    // Create 5 fake backups
    for i in 1..=5 {
        let backup_name = format!("server.toml.bak.2025-01-0{}T12-00-00", i);
        fs::write(backup_dir.join(&backup_name), format!("backup {}", i)).unwrap();
    }

    // Verify we have 5
    let backups = list_backups(&backup_dir, "server.toml").unwrap();
    assert_eq!(backups.len(), 5);

    // Run rotation
    let deleted =
        plix_common::migration::rotate_backups(&backup_dir, "server.toml", MAX_BACKUPS).unwrap();

    // Verify 2 were deleted
    assert_eq!(deleted.len(), 2);

    // Verify 3 remain
    let remaining = list_backups(&backup_dir, "server.toml").unwrap();
    assert_eq!(remaining.len(), 3);

    // Verify the newest 3 are kept (03, 04, 05)
    let names: Vec<_> = remaining
        .iter()
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_string())
        .collect();
    assert!(names[0].contains("03"));
    assert!(names[1].contains("04"));
    assert!(names[2].contains("05"));
}

#[test]
fn test_v1_config_no_migration_needed() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config").join("server.toml");
    let backup_dir = temp.path().join("backups");

    create_v1_config(&config_path);

    // Set up migration engine
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));

    // Try to run migration on already-migrated config
    let result = engine.run_migrations(&config_path, MigrationVersion::V1_0_0);

    // Should fail with AlreadyAtVersion
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(format!("{}", err).contains("already at target version"));
}

#[test]
fn test_migration_preserves_all_data() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config").join("server.toml");
    let backup_dir = temp.path().join("backups");

    // Create config with lots of data
    let content = r#"
[network]
bind_address = "192.168.1.100"
port = 8888
max_players = 64

[game]
tick_rate = 60
arena = "competitive_arena"

[server]
name = "Pro Gaming Server"
motd = "Welcome to competitive play!"

[logging]
level = "debug"

[persistence]
autosave_interval_secs = 120
shutdown_timeout_secs = 30
"#;
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(&config_path, content).unwrap();

    // Set up and run migration
    let mut engine = MigrationEngine::new(&backup_dir);
    engine.register(Box::new(V0ToV1ConfigMigration));
    engine
        .run_migrations(&config_path, MigrationVersion::ZERO)
        .unwrap();

    // Verify all data preserved
    let migrated = fs::read_to_string(&config_path).unwrap();
    assert!(migrated.contains("config_version = \"1.0.0\""));
    assert!(migrated.contains("bind_address = \"192.168.1.100\""));
    assert!(migrated.contains("port = 8888"));
    assert!(migrated.contains("max_players = 64"));
    assert!(migrated.contains("tick_rate = 60"));
    assert!(migrated.contains("arena = \"competitive_arena\""));
    assert!(migrated.contains("name = \"Pro Gaming Server\""));
    assert!(migrated.contains("motd = \"Welcome to competitive play!\""));
    assert!(migrated.contains("level = \"debug\""));
    assert!(migrated.contains("autosave_interval_secs = 120"));
    assert!(migrated.contains("shutdown_timeout_secs = 30"));
}
