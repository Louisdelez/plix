//! Game configuration persistence

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::ui_cef::CefConfig;

/// All rebindable player actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Attack,
    PlaceBlock,
    RemoveBlock,
    Pause,
    ToggleDebugOverlay,
}

impl Action {
    /// Get all actions in display order
    pub fn all() -> &'static [Action] {
        &[
            Action::Forward,
            Action::Backward,
            Action::Left,
            Action::Right,
            Action::Jump,
            Action::Attack,
            Action::PlaceBlock,
            Action::RemoveBlock,
            Action::Pause,
            Action::ToggleDebugOverlay,
        ]
    }

    /// Get display name for the action
    pub fn display_name(&self) -> &'static str {
        match self {
            Action::Forward => "Forward",
            Action::Backward => "Backward",
            Action::Left => "Left",
            Action::Right => "Right",
            Action::Jump => "Jump",
            Action::Attack => "Attack",
            Action::PlaceBlock => "Place Block",
            Action::RemoveBlock => "Remove Block",
            Action::Pause => "Pause",
            Action::ToggleDebugOverlay => "Debug Overlay",
        }
    }
}

/// Key enumeration for input (extended for rebinding)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    // Letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Special keys
    Space,
    Escape,
    Enter,
    Tab,
    Backspace,

    // Modifiers
    Ctrl,
    Shift,
    Alt,

    // Arrow keys
    Up,
    Down,
    Left,
    Right,

    // Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Mouse buttons
    LeftClick,
    RightClick,
    MiddleClick,
}

impl Key {
    /// Convert from winit KeyCode
    pub fn from_keycode(code: winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode;
        match code {
            KeyCode::KeyA => Some(Key::A),
            KeyCode::KeyB => Some(Key::B),
            KeyCode::KeyC => Some(Key::C),
            KeyCode::KeyD => Some(Key::D),
            KeyCode::KeyE => Some(Key::E),
            KeyCode::KeyF => Some(Key::F),
            KeyCode::KeyG => Some(Key::G),
            KeyCode::KeyH => Some(Key::H),
            KeyCode::KeyI => Some(Key::I),
            KeyCode::KeyJ => Some(Key::J),
            KeyCode::KeyK => Some(Key::K),
            KeyCode::KeyL => Some(Key::L),
            KeyCode::KeyM => Some(Key::M),
            KeyCode::KeyN => Some(Key::N),
            KeyCode::KeyO => Some(Key::O),
            KeyCode::KeyP => Some(Key::P),
            KeyCode::KeyQ => Some(Key::Q),
            KeyCode::KeyR => Some(Key::R),
            KeyCode::KeyS => Some(Key::S),
            KeyCode::KeyT => Some(Key::T),
            KeyCode::KeyU => Some(Key::U),
            KeyCode::KeyV => Some(Key::V),
            KeyCode::KeyW => Some(Key::W),
            KeyCode::KeyX => Some(Key::X),
            KeyCode::KeyY => Some(Key::Y),
            KeyCode::KeyZ => Some(Key::Z),
            KeyCode::Digit0 => Some(Key::Key0),
            KeyCode::Digit1 => Some(Key::Key1),
            KeyCode::Digit2 => Some(Key::Key2),
            KeyCode::Digit3 => Some(Key::Key3),
            KeyCode::Digit4 => Some(Key::Key4),
            KeyCode::Digit5 => Some(Key::Key5),
            KeyCode::Digit6 => Some(Key::Key6),
            KeyCode::Digit7 => Some(Key::Key7),
            KeyCode::Digit8 => Some(Key::Key8),
            KeyCode::Digit9 => Some(Key::Key9),
            KeyCode::Space => Some(Key::Space),
            KeyCode::Escape => Some(Key::Escape),
            KeyCode::Enter => Some(Key::Enter),
            KeyCode::Tab => Some(Key::Tab),
            KeyCode::Backspace => Some(Key::Backspace),
            KeyCode::ControlLeft | KeyCode::ControlRight => Some(Key::Ctrl),
            KeyCode::ShiftLeft | KeyCode::ShiftRight => Some(Key::Shift),
            KeyCode::AltLeft | KeyCode::AltRight => Some(Key::Alt),
            KeyCode::ArrowUp => Some(Key::Up),
            KeyCode::ArrowDown => Some(Key::Down),
            KeyCode::ArrowLeft => Some(Key::Left),
            KeyCode::ArrowRight => Some(Key::Right),
            KeyCode::F1 => Some(Key::F1),
            KeyCode::F2 => Some(Key::F2),
            KeyCode::F3 => Some(Key::F3),
            KeyCode::F4 => Some(Key::F4),
            KeyCode::F5 => Some(Key::F5),
            KeyCode::F6 => Some(Key::F6),
            KeyCode::F7 => Some(Key::F7),
            KeyCode::F8 => Some(Key::F8),
            KeyCode::F9 => Some(Key::F9),
            KeyCode::F10 => Some(Key::F10),
            KeyCode::F11 => Some(Key::F11),
            KeyCode::F12 => Some(Key::F12),
            _ => None,
        }
    }

    /// Convert from mouse button
    pub fn from_mouse_button(button: winit::event::MouseButton) -> Option<Self> {
        use winit::event::MouseButton;
        match button {
            MouseButton::Left => Some(Key::LeftClick),
            MouseButton::Right => Some(Key::RightClick),
            MouseButton::Middle => Some(Key::MiddleClick),
            _ => None,
        }
    }

    /// Get display name for the key
    pub fn display_name(&self) -> &'static str {
        match self {
            Key::A => "A",
            Key::B => "B",
            Key::C => "C",
            Key::D => "D",
            Key::E => "E",
            Key::F => "F",
            Key::G => "G",
            Key::H => "H",
            Key::I => "I",
            Key::J => "J",
            Key::K => "K",
            Key::L => "L",
            Key::M => "M",
            Key::N => "N",
            Key::O => "O",
            Key::P => "P",
            Key::Q => "Q",
            Key::R => "R",
            Key::S => "S",
            Key::T => "T",
            Key::U => "U",
            Key::V => "V",
            Key::W => "W",
            Key::X => "X",
            Key::Y => "Y",
            Key::Z => "Z",
            Key::Key0 => "0",
            Key::Key1 => "1",
            Key::Key2 => "2",
            Key::Key3 => "3",
            Key::Key4 => "4",
            Key::Key5 => "5",
            Key::Key6 => "6",
            Key::Key7 => "7",
            Key::Key8 => "8",
            Key::Key9 => "9",
            Key::Space => "Space",
            Key::Escape => "Escape",
            Key::Enter => "Enter",
            Key::Tab => "Tab",
            Key::Backspace => "Backspace",
            Key::Ctrl => "Ctrl",
            Key::Shift => "Shift",
            Key::Alt => "Alt",
            Key::Up => "Up",
            Key::Down => "Down",
            Key::Left => "Left Arrow",
            Key::Right => "Right Arrow",
            Key::F1 => "F1",
            Key::F2 => "F2",
            Key::F3 => "F3",
            Key::F4 => "F4",
            Key::F5 => "F5",
            Key::F6 => "F6",
            Key::F7 => "F7",
            Key::F8 => "F8",
            Key::F9 => "F9",
            Key::F10 => "F10",
            Key::F11 => "F11",
            Key::F12 => "F12",
            Key::LeftClick => "LMB",
            Key::RightClick => "RMB",
            Key::MiddleClick => "MMB",
        }
    }
}

/// Key bindings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keybinds {
    /// Map of action to bound key
    pub bindings: HashMap<Action, Key>,
}

impl Default for Keybinds {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        bindings.insert(Action::Forward, Key::W);
        bindings.insert(Action::Backward, Key::S);
        bindings.insert(Action::Left, Key::A);
        bindings.insert(Action::Right, Key::D);
        bindings.insert(Action::Jump, Key::Space);
        bindings.insert(Action::Attack, Key::LeftClick);
        bindings.insert(Action::PlaceBlock, Key::RightClick);
        bindings.insert(Action::RemoveBlock, Key::LeftClick);
        bindings.insert(Action::Pause, Key::Escape);
        bindings.insert(Action::ToggleDebugOverlay, Key::F3);
        Self { bindings }
    }
}

impl Keybinds {
    /// Get the key bound to an action
    pub fn get(&self, action: Action) -> Option<Key> {
        self.bindings.get(&action).copied()
    }

    /// Set a key binding for an action
    pub fn set(&mut self, action: Action, key: Key) {
        self.bindings.insert(action, key);
    }

    /// Find which action (if any) is bound to a key
    pub fn action_for_key(&self, key: Key) -> Option<Action> {
        self.bindings
            .iter()
            .find(|(_, &k)| k == key)
            .map(|(&action, _)| action)
    }

    /// Swap bindings between two actions (for conflict resolution)
    pub fn swap(&mut self, action1: Action, action2: Action) {
        let key1 = self.bindings.get(&action1).copied();
        let key2 = self.bindings.get(&action2).copied();
        if let (Some(k1), Some(k2)) = (key1, key2) {
            self.bindings.insert(action1, k2);
            self.bindings.insert(action2, k1);
        }
    }

    /// Ensure all actions have bindings (fill missing with defaults)
    pub fn ensure_all_actions_bound(&mut self) {
        let defaults = Keybinds::default();
        for action in Action::all() {
            if !self.bindings.contains_key(action) {
                if let Some(key) = defaults.get(*action) {
                    self.bindings.insert(*action, key);
                }
            }
        }
    }
}

/// Sensitivity constraints
pub const SENSITIVITY_MIN: f32 = 0.0001;
pub const SENSITIVITY_MAX: f32 = 0.01;
pub const SENSITIVITY_DEFAULT: f32 = 0.003;
pub const SENSITIVITY_STEP: f32 = 0.0005;

/// FOV constraints
pub const FOV_MIN: f32 = 60.0;
pub const FOV_MAX: f32 = 110.0;
pub const FOV_DEFAULT: f32 = 70.0;
pub const FOV_STEP: f32 = 5.0;

/// Game configuration persisted to config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Mouse sensitivity multiplier (0.0001 to 0.01)
    pub sensitivity: f32,

    /// Field of view in degrees (60 to 110)
    pub fov_degrees: f32,

    /// Fullscreen mode enabled
    pub fullscreen: bool,

    /// Master audio muted
    pub audio_muted: bool,

    /// Key bindings for all rebindable actions
    pub keybinds: Keybinds,

    /// CEF UI configuration (Feature 030)
    #[serde(default)]
    pub ui: CefConfig,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            sensitivity: SENSITIVITY_DEFAULT,
            fov_degrees: FOV_DEFAULT,
            fullscreen: false,
            audio_muted: false,
            keybinds: Keybinds::default(),
            ui: CefConfig::default(),
        }
    }
}

impl GameConfig {
    /// Validate and clamp out-of-range values
    pub fn validate(&mut self) {
        self.sensitivity = self.sensitivity.clamp(SENSITIVITY_MIN, SENSITIVITY_MAX);
        self.fov_degrees = self.fov_degrees.clamp(FOV_MIN, FOV_MAX);
        self.keybinds.ensure_all_actions_bound();
        self.ui.validate();
    }
}

/// Get platform-specific config directory path
pub fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("plix")
        .join("config.toml")
}

/// Load configuration from default platform path
pub fn load_config() -> GameConfig {
    let path = config_path();
    match load_config_from(&path) {
        Ok(config) => config,
        Err(e) => {
            warn!(error = %e, path = %path.display(), "Failed to load config, using defaults");
            GameConfig::default()
        }
    }
}

/// Load configuration from specific path
pub fn load_config_from(path: &std::path::Path) -> Result<GameConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let mut config: GameConfig = toml::from_str(&content)?;
    config.validate();
    info!(path = %path.display(), "Config loaded");
    Ok(config)
}

/// Save configuration to default platform path
pub fn save_config(config: &GameConfig) -> Result<(), ConfigError> {
    let path = config_path();
    save_config_to(config, &path)
}

/// Save configuration to specific path (atomic write with temp file)
pub fn save_config_to(config: &GameConfig, path: &std::path::Path) -> Result<(), ConfigError> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let content = toml::to_string_pretty(config)?;

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("toml.tmp");
    std::fs::write(&temp_path, &content)?;
    std::fs::rename(&temp_path, path)?;

    info!(path = %path.display(), "Config saved");
    Ok(())
}

/// Configuration error
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert!((config.sensitivity - SENSITIVITY_DEFAULT).abs() < f32::EPSILON);
        assert!((config.fov_degrees - FOV_DEFAULT).abs() < f32::EPSILON);
        assert!(!config.fullscreen);
        assert!(!config.audio_muted);
    }

    #[test]
    fn test_validate_clamps_sensitivity() {
        let mut config = GameConfig::default();
        config.sensitivity = 1.0; // Way too high
        config.validate();
        assert!((config.sensitivity - SENSITIVITY_MAX).abs() < f32::EPSILON);

        config.sensitivity = 0.0; // Too low
        config.validate();
        assert!((config.sensitivity - SENSITIVITY_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_validate_clamps_fov() {
        let mut config = GameConfig::default();
        config.fov_degrees = 200.0; // Way too high
        config.validate();
        assert!((config.fov_degrees - FOV_MAX).abs() < f32::EPSILON);

        config.fov_degrees = 30.0; // Too low
        config.validate();
        assert!((config.fov_degrees - FOV_MIN).abs() < f32::EPSILON);
    }

    #[test]
    fn test_keybinds_default() {
        let keybinds = Keybinds::default();
        assert_eq!(keybinds.get(Action::Forward), Some(Key::W));
        assert_eq!(keybinds.get(Action::Backward), Some(Key::S));
        assert_eq!(keybinds.get(Action::Jump), Some(Key::Space));
    }

    #[test]
    fn test_keybinds_swap() {
        let mut keybinds = Keybinds::default();
        keybinds.swap(Action::Forward, Action::Backward);
        assert_eq!(keybinds.get(Action::Forward), Some(Key::S));
        assert_eq!(keybinds.get(Action::Backward), Some(Key::W));
    }

    #[test]
    fn test_action_for_key() {
        let keybinds = Keybinds::default();
        assert_eq!(keybinds.action_for_key(Key::W), Some(Action::Forward));
        assert_eq!(keybinds.action_for_key(Key::Space), Some(Action::Jump));
    }

    #[test]
    fn test_ui_config_defaults() {
        let config = GameConfig::default();
        assert!(config.ui.enabled);
        assert!(!config.ui.devtools);
        assert_eq!(config.ui.initial_page, "index.html");
        assert_eq!(config.ui.frame_rate, 60);
    }

    #[test]
    fn test_ui_config_validates() {
        let mut config = GameConfig::default();
        config.ui.frame_rate = 200; // Too high
        config.ui.initial_page = "../secret.html".to_string(); // Path traversal
        config.validate();
        assert_eq!(config.ui.frame_rate, 120); // Clamped to max
        assert_eq!(config.ui.initial_page, "index.html"); // Reset to default
    }

    #[test]
    fn test_ui_config_serde() {
        let config = GameConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        // Should contain [ui] section
        assert!(toml_str.contains("[ui]"));
        // Should roundtrip
        let parsed: GameConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.ui.enabled, config.ui.enabled);
        assert_eq!(parsed.ui.frame_rate, config.ui.frame_rate);
    }

    #[test]
    fn test_config_without_ui_section_uses_defaults() {
        // Config without [ui] section should use defaults (backward compatible)
        let toml_str = r#"
sensitivity = 0.003
fov_degrees = 70.0
fullscreen = false
audio_muted = false

[keybinds.bindings]
Forward = "W"
"#;
        let config: GameConfig = toml::from_str(toml_str).unwrap();
        assert!(config.ui.enabled); // Default
        assert_eq!(config.ui.frame_rate, 60); // Default
    }
}
