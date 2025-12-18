//! LocalState persistence tests

use plix_launcher::LocalState;
use std::collections::HashMap;
use tempfile::tempdir;

#[test]
fn test_state_default() {
    let state = LocalState::default();
    assert_eq!(state.state_version, 1);
    assert!(state.installed_version.is_none());
    assert!(state.last_update.is_none());
    assert!(state.file_checksums.is_empty());
    assert!(state.install_path.is_none());
}

#[test]
fn test_state_persistence() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.toml");

    // Create and save state
    let mut state = LocalState::default();
    state.installed_version = Some("1.5.0".to_string());
    state.install_path = Some("/home/user/.local/share/plix/current".to_string());
    state
        .file_checksums
        .insert("plix-client".to_string(), "a1b2c3d4e5f67890".to_string());
    state.file_checksums.insert(
        "assets/arena.toml".to_string(),
        "f6e5d4c3b2a19876".to_string(),
    );

    state.save(&path).unwrap();

    // Load and verify
    let loaded = LocalState::load(&path).unwrap();
    assert_eq!(loaded.installed_version, Some("1.5.0".to_string()));
    assert_eq!(
        loaded.install_path,
        Some("/home/user/.local/share/plix/current".to_string())
    );
    assert_eq!(loaded.file_checksums.len(), 2);
    assert_eq!(
        loaded.file_checksums.get("plix-client"),
        Some(&"a1b2c3d4e5f67890".to_string())
    );
}

#[test]
fn test_state_load_missing_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("does_not_exist.toml");

    let state = LocalState::load(&path).unwrap();
    // Should return default state
    assert!(state.installed_version.is_none());
    assert!(state.file_checksums.is_empty());
}

#[test]
fn test_state_update_after_install() {
    let mut state = LocalState::default();

    let mut checksums = HashMap::new();
    checksums.insert("bin/game".to_string(), "hash1".to_string());
    checksums.insert("data/config.toml".to_string(), "hash2".to_string());

    state.update_after_install("3.0.0".to_string(), "/opt/game".to_string(), checksums);

    assert_eq!(state.installed_version, Some("3.0.0".to_string()));
    assert_eq!(state.install_path, Some("/opt/game".to_string()));
    assert!(state.last_update.is_some());
    assert_eq!(state.file_checksums.len(), 2);
}

#[test]
fn test_state_has_installed_version() {
    let mut state = LocalState::default();
    assert!(!state.has_installed_version());

    state.installed_version = Some("1.0.0".to_string());
    assert!(state.has_installed_version());

    state.installed_version = None;
    assert!(!state.has_installed_version());
}

#[test]
fn test_state_atomic_write() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.toml");

    let mut state = LocalState::default();
    state.installed_version = Some("1.0.0".to_string());

    // First save
    state.save(&path).unwrap();
    let content1 = std::fs::read_to_string(&path).unwrap();

    // Update and save again
    state.installed_version = Some("2.0.0".to_string());
    state.save(&path).unwrap();
    let content2 = std::fs::read_to_string(&path).unwrap();

    assert!(content1.contains("1.0.0"));
    assert!(content2.contains("2.0.0"));
    assert!(!content2.contains("1.0.0"));

    // No temp file should remain
    let temp_path = path.with_extension("toml.tmp");
    assert!(!temp_path.exists());
}

#[test]
fn test_state_toml_format() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("state.toml");

    let mut state = LocalState::default();
    state.installed_version = Some("1.2.3".to_string());
    state.last_update = Some(1734567890);
    state
        .file_checksums
        .insert("game".to_string(), "abc123".to_string());

    state.save(&path).unwrap();

    let content = std::fs::read_to_string(&path).unwrap();

    // Verify TOML structure
    assert!(content.contains("state_version = 1"));
    assert!(content.contains("installed_version = \"1.2.3\""));
    assert!(content.contains("last_update = 1734567890"));
    assert!(content.contains("[file_checksums]"));
    assert!(content.contains("game = \"abc123\""));
}
