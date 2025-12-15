# Contract: Configuration File

**Feature**: 005-minimal-ui-native
**Date**: 2025-12-15

## Overview

Specifies the configuration file format, location, loading behavior, and persistence rules for game settings.

---

## File Location

| Platform | Path |
|----------|------|
| Linux | `~/.config/plix/config.toml` |
| Windows | `%APPDATA%\plix\config.toml` |
| macOS | `~/Library/Application Support/plix/config.toml` |

**Directory Creation**: If the directory doesn't exist, create it with standard permissions (0755 on Unix).

**File Creation**: If the file doesn't exist, create it with default values.

---

## File Format

TOML format with the following schema:

```toml
# Mouse sensitivity (0.0001 to 0.01)
sensitivity = 0.003

# Field of view in degrees (60 to 110)
fov_degrees = 70.0

# Fullscreen mode
fullscreen = false

# Master audio muted
audio_muted = false

# Key bindings
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
```

---

## Default Values

| Setting | Default | Min | Max |
|---------|---------|-----|-----|
| sensitivity | 0.003 | 0.0001 | 0.01 |
| fov_degrees | 70.0 | 60.0 | 110.0 |
| fullscreen | false | - | - |
| audio_muted | false | - | - |

### Default Keybinds

| Action | Default Key |
|--------|-------------|
| Forward | W |
| Backward | S |
| Left | A |
| Right | D |
| Jump | Space |
| Attack | LeftClick |
| PlaceBlock | RightClick |
| RemoveBlock | LeftClick |
| Pause | Escape |

---

## Loading Behavior

### Load Sequence

```
1. Determine config path for current platform
2. Check if file exists
   - If not: Return default GameConfig
3. Read file contents
   - If read error: Log warning, return defaults
4. Parse TOML
   - If parse error: Log warning, return defaults
5. Deserialize to GameConfig
   - If deserialize error: Log warning, return defaults
6. Validate and clamp values
7. Fill missing keybinds with defaults
8. Return validated GameConfig
```

### Validation on Load

```rust
impl GameConfig {
    pub fn validate(&mut self) {
        // Clamp sensitivity
        self.sensitivity = self.sensitivity.clamp(0.0001, 0.01);

        // Clamp FOV
        self.fov_degrees = self.fov_degrees.clamp(60.0, 110.0);

        // Ensure all actions have bindings
        self.keybinds.ensure_all_actions_bound();
    }
}
```

### Error Recovery

| Error | Behavior |
|-------|----------|
| File not found | Use defaults, create file on first save |
| Permission denied | Log warning, use defaults, disable saving |
| Invalid TOML syntax | Log error with line number, use defaults |
| Invalid field type | Log warning, use default for that field |
| Missing field | Use default for that field |
| Value out of range | Clamp to valid range |
| Unknown field | Ignore (forward compatibility) |

---

## Saving Behavior

### Save Triggers

Settings are saved immediately when:
- Sensitivity slider changes value
- FOV slider changes value
- Fullscreen toggle changes
- Audio mute toggle changes
- Keybind is changed (after conflict resolution)

### Save Sequence

```
1. Serialize GameConfig to TOML string
2. Ensure directory exists (create if needed)
3. Write to temp file (config.toml.tmp)
4. Rename temp file to config.toml (atomic on Unix)
5. On error: Log warning, continue (don't crash)
```

### Atomic Write

Use temp file + rename pattern to prevent corruption:

```rust
pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
    let content = toml::to_string_pretty(self)?;
    let temp_path = path.with_extension("toml.tmp");

    std::fs::write(&temp_path, content)?;
    std::fs::rename(&temp_path, path)?;

    Ok(())
}
```

---

## API Contract

### Functions

```rust
/// Load configuration from default platform path
pub fn load_config() -> GameConfig {
    let path = config_path();
    match load_config_from(&path) {
        Ok(config) => config,
        Err(e) => {
            warn!(error = %e, "Failed to load config, using defaults");
            GameConfig::default()
        }
    }
}

/// Load configuration from specific path
pub fn load_config_from(path: &Path) -> Result<GameConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let mut config: GameConfig = toml::from_str(&content)?;
    config.validate();
    Ok(config)
}

/// Save configuration to default platform path
pub fn save_config(config: &GameConfig) -> Result<(), ConfigError> {
    let path = config_path();
    save_config_to(config, &path)
}

/// Save configuration to specific path
pub fn save_config_to(config: &GameConfig, path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = toml::to_string_pretty(config)?;
    let temp_path = path.with_extension("toml.tmp");
    std::fs::write(&temp_path, &content)?;
    std::fs::rename(&temp_path, path)?;
    Ok(())
}

/// Get platform-specific config path
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("config.toml")
}
```

### Error Type

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}
```

---

## Test Cases

### TC-001: Load missing file

```rust
#[test]
fn load_missing_file_returns_defaults() {
    let config = load_config_from(Path::new("/nonexistent/config.toml"));
    assert!(config.is_err());
    // load_config() should return defaults without error
}
```

### TC-002: Load valid file

```rust
#[test]
fn load_valid_config() {
    let content = r#"
        sensitivity = 0.005
        fov_degrees = 90.0
        fullscreen = true
        audio_muted = true
    "#;
    // write to temp file, load, verify values
}
```

### TC-003: Clamp out-of-range values

```rust
#[test]
fn clamp_out_of_range() {
    let content = r#"
        sensitivity = 1.0  # way too high
        fov_degrees = 200.0  # way too high
    "#;
    // load, verify sensitivity clamped to 0.01, FOV to 110.0
}
```

### TC-004: Handle corrupted file

```rust
#[test]
fn handle_corrupted_file() {
    let content = "not valid toml {{{{";
    // load_config_from should error, load_config should return defaults
}
```

### TC-005: Save and reload roundtrip

```rust
#[test]
fn save_reload_roundtrip() {
    let config = GameConfig {
        sensitivity: 0.005,
        fov_degrees: 90.0,
        fullscreen: true,
        audio_muted: true,
        keybinds: Keybinds::default(),
    };
    // save to temp file, reload, verify equal
}
```

### TC-006: Missing keybinds filled with defaults

```rust
#[test]
fn missing_keybinds_use_defaults() {
    let content = r#"
        sensitivity = 0.003
        fov_degrees = 70.0
        [keybinds.bindings]
        Forward = "W"
        # Missing all other bindings
    "#;
    // load, verify all actions have bindings (defaults for missing)
}
```

---

## Compatibility

### Forward Compatibility

Unknown fields in the config file are silently ignored, allowing older clients to use configs created by newer versions.

### Backward Compatibility

New fields added in future versions use defaults when loading old config files. The `validate()` method ensures all required fields are present.

### Version Header (Future)

Consider adding a version field for major schema changes:

```toml
config_version = 1
# ... rest of config
```

Currently not required as initial version.
