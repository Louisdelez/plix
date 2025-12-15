//! Integration tests for config persistence

use plix_client::config::{
    load_config_from, save_config_to, Action, GameConfig, Key, SENSITIVITY_DEFAULT, SENSITIVITY_MAX,
};
use tempfile::tempdir;

/// T025: Load missing file returns defaults
#[test]
fn test_load_missing_file_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("nonexistent.toml");

    // load_config_from should error on missing file
    let result = load_config_from(&path);
    assert!(result.is_err());
}

/// T026: Save and reload roundtrip
#[test]
fn test_save_reload_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    // Create custom config
    let mut config = GameConfig::default();
    config.sensitivity = 0.005;
    config.fov_degrees = 90.0;
    config.fullscreen = true;
    config.audio_muted = true;

    // Save
    save_config_to(&config, &path).expect("Failed to save config");

    // Reload
    let loaded = load_config_from(&path).expect("Failed to load config");

    // Verify values match
    assert!((loaded.sensitivity - 0.005).abs() < f32::EPSILON);
    assert!((loaded.fov_degrees - 90.0).abs() < f32::EPSILON);
    assert!(loaded.fullscreen);
    assert!(loaded.audio_muted);
}

/// T027: Clamp out-of-range values on load
#[test]
fn test_clamp_out_of_range_values() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    // Write config with out-of-range values
    let content = r#"
sensitivity = 1.0
fov_degrees = 200.0
fullscreen = false
audio_muted = false

[keybinds.bindings]
Forward = "W"
Backward = "S"
Left = "A"
Right = "D"
Jump = "Space"
Attack = "LeftClick"
PlaceBlock = "RightClick"
RemoveBlock = "LeftClick"
Pause = "Escape"
"#;
    std::fs::write(&path, content).expect("Failed to write test config");

    // Load - should clamp values
    let loaded = load_config_from(&path).expect("Failed to load config");

    // Verify clamped
    assert!((loaded.sensitivity - SENSITIVITY_MAX).abs() < f32::EPSILON);
    assert!((loaded.fov_degrees - 110.0).abs() < f32::EPSILON);
}

/// Test corrupted config file falls back (via load_config which returns default)
#[test]
fn test_corrupted_config_returns_error() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    // Write invalid TOML
    std::fs::write(&path, "invalid toml {{{{").expect("Failed to write test config");

    // load_config_from should error
    let result = load_config_from(&path);
    assert!(result.is_err());
}

/// Test keybinds serialization roundtrip
#[test]
fn test_keybinds_roundtrip() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    // Create config with modified keybinds
    let mut config = GameConfig::default();
    config.keybinds.set(Action::Forward, Key::I);
    config.keybinds.set(Action::Backward, Key::K);

    // Save and reload
    save_config_to(&config, &path).expect("Failed to save config");
    let loaded = load_config_from(&path).expect("Failed to load config");

    // Verify keybinds
    assert_eq!(loaded.keybinds.get(Action::Forward), Some(Key::I));
    assert_eq!(loaded.keybinds.get(Action::Backward), Some(Key::K));
    // Others should still be defaults
    assert_eq!(loaded.keybinds.get(Action::Jump), Some(Key::Space));
}

/// Test missing keybinds are filled with defaults
#[test]
fn test_missing_keybinds_use_defaults() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("config.toml");

    // Write config with only some keybinds
    let content = r#"
sensitivity = 0.003
fov_degrees = 70.0
fullscreen = false
audio_muted = false

[keybinds.bindings]
Forward = "W"
"#;
    std::fs::write(&path, content).expect("Failed to write test config");

    // Load - missing keybinds should be filled
    let loaded = load_config_from(&path).expect("Failed to load config");

    // Forward was specified
    assert_eq!(loaded.keybinds.get(Action::Forward), Some(Key::W));
    // Others should be filled with defaults
    assert_eq!(loaded.keybinds.get(Action::Backward), Some(Key::S));
    assert_eq!(loaded.keybinds.get(Action::Jump), Some(Key::Space));
}

/// Test default config values
#[test]
fn test_default_config_values() {
    let config = GameConfig::default();

    assert!((config.sensitivity - SENSITIVITY_DEFAULT).abs() < f32::EPSILON);
    assert!((config.fov_degrees - 70.0).abs() < f32::EPSILON);
    assert!(!config.fullscreen);
    assert!(!config.audio_muted);

    // Check default keybinds
    assert_eq!(config.keybinds.get(Action::Forward), Some(Key::W));
    assert_eq!(config.keybinds.get(Action::Backward), Some(Key::S));
    assert_eq!(config.keybinds.get(Action::Left), Some(Key::A));
    assert_eq!(config.keybinds.get(Action::Right), Some(Key::D));
    assert_eq!(config.keybinds.get(Action::Jump), Some(Key::Space));
    assert_eq!(config.keybinds.get(Action::Attack), Some(Key::LeftClick));
    assert_eq!(
        config.keybinds.get(Action::PlaceBlock),
        Some(Key::RightClick)
    );
    assert_eq!(
        config.keybinds.get(Action::RemoveBlock),
        Some(Key::LeftClick)
    );
    assert_eq!(config.keybinds.get(Action::Pause), Some(Key::Escape));
}

/// Test atomic write creates parent directories
#[test]
fn test_save_creates_parent_dirs() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("subdir").join("nested").join("config.toml");

    let config = GameConfig::default();
    save_config_to(&config, &path).expect("Failed to save config");

    assert!(path.exists());
}
