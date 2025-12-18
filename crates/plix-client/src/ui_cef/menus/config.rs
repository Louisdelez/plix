//! Config handlers for CEF UI (Feature 031)
//!
//! Handles GetConfig and SetConfig messages for the settings page.

use crate::config::{
    Action, GameConfig, Key, Keybinds, FOV_MAX, FOV_MIN, SENSITIVITY_MAX, SENSITIVITY_MIN,
};
use crate::ui_cef::bridge::messages::{BridgeError, BridgeResponse};
use crate::ui_cef::bridge::serialize::sanitize_string_plain;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// GameConfig subset exposed to UI (matches data-model.md)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfigUI {
    /// Mouse sensitivity (0.0001 - 0.01)
    pub sensitivity: f32,

    /// Field of view in degrees (60 - 110)
    pub fov_degrees: f32,

    /// Fullscreen mode enabled
    pub fullscreen: bool,

    /// Master audio muted
    pub audio_muted: bool,

    /// Key bindings map
    pub keybinds: KeybindMapUI,
}

/// Keybind map for UI (string keys and values)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeybindMapUI {
    pub forward: String,
    pub backward: String,
    pub left: String,
    pub right: String,
    pub jump: String,
    pub attack: String,
    pub place_block: String,
    pub remove_block: String,
    pub pause: String,
    pub toggle_debug: String,
}

impl From<&Keybinds> for KeybindMapUI {
    fn from(keybinds: &Keybinds) -> Self {
        Self {
            forward: keybinds
                .get(Action::Forward)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            backward: keybinds
                .get(Action::Backward)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            left: keybinds
                .get(Action::Left)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            right: keybinds
                .get(Action::Right)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            jump: keybinds
                .get(Action::Jump)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            attack: keybinds
                .get(Action::Attack)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            place_block: keybinds
                .get(Action::PlaceBlock)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            remove_block: keybinds
                .get(Action::RemoveBlock)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            pause: keybinds
                .get(Action::Pause)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
            toggle_debug: keybinds
                .get(Action::ToggleDebugOverlay)
                .map(|k| k.display_name().to_string())
                .unwrap_or_default(),
        }
    }
}

/// Convert GameConfig to UI-friendly format
pub fn to_ui_config(config: &GameConfig) -> GameConfigUI {
    GameConfigUI {
        sensitivity: config.sensitivity,
        fov_degrees: config.fov_degrees,
        fullscreen: config.fullscreen,
        audio_muted: config.audio_muted,
        keybinds: KeybindMapUI::from(&config.keybinds),
    }
}

/// Parse key name from UI string to Key enum
fn parse_key(name: &str) -> Option<Key> {
    let name = name.trim();
    match name {
        // Letters
        "A" => Some(Key::A),
        "B" => Some(Key::B),
        "C" => Some(Key::C),
        "D" => Some(Key::D),
        "E" => Some(Key::E),
        "F" => Some(Key::F),
        "G" => Some(Key::G),
        "H" => Some(Key::H),
        "I" => Some(Key::I),
        "J" => Some(Key::J),
        "K" => Some(Key::K),
        "L" => Some(Key::L),
        "M" => Some(Key::M),
        "N" => Some(Key::N),
        "O" => Some(Key::O),
        "P" => Some(Key::P),
        "Q" => Some(Key::Q),
        "R" => Some(Key::R),
        "S" => Some(Key::S),
        "T" => Some(Key::T),
        "U" => Some(Key::U),
        "V" => Some(Key::V),
        "W" => Some(Key::W),
        "X" => Some(Key::X),
        "Y" => Some(Key::Y),
        "Z" => Some(Key::Z),
        // Numbers
        "0" => Some(Key::Key0),
        "1" => Some(Key::Key1),
        "2" => Some(Key::Key2),
        "3" => Some(Key::Key3),
        "4" => Some(Key::Key4),
        "5" => Some(Key::Key5),
        "6" => Some(Key::Key6),
        "7" => Some(Key::Key7),
        "8" => Some(Key::Key8),
        "9" => Some(Key::Key9),
        // Special
        "Space" => Some(Key::Space),
        "Escape" => Some(Key::Escape),
        "Enter" => Some(Key::Enter),
        "Tab" => Some(Key::Tab),
        "Backspace" => Some(Key::Backspace),
        "Ctrl" => Some(Key::Ctrl),
        "Shift" => Some(Key::Shift),
        "Alt" => Some(Key::Alt),
        "Up" => Some(Key::Up),
        "Down" => Some(Key::Down),
        "Left Arrow" => Some(Key::Left),
        "Right Arrow" => Some(Key::Right),
        // Function keys
        "F1" => Some(Key::F1),
        "F2" => Some(Key::F2),
        "F3" => Some(Key::F3),
        "F4" => Some(Key::F4),
        "F5" => Some(Key::F5),
        "F6" => Some(Key::F6),
        "F7" => Some(Key::F7),
        "F8" => Some(Key::F8),
        "F9" => Some(Key::F9),
        "F10" => Some(Key::F10),
        "F11" => Some(Key::F11),
        "F12" => Some(Key::F12),
        // Mouse
        "LMB" => Some(Key::LeftClick),
        "RMB" => Some(Key::RightClick),
        "MMB" => Some(Key::MiddleClick),
        _ => None,
    }
}

/// Update GameConfig from UI format
pub fn from_ui_config(
    ui_config: &GameConfigUI,
    config: &mut GameConfig,
) -> Result<(), BridgeError> {
    // Validate sensitivity
    if ui_config.sensitivity < SENSITIVITY_MIN || ui_config.sensitivity > SENSITIVITY_MAX {
        return Err(BridgeError::validation_failed(&format!(
            "Sensitivity must be between {} and {}",
            SENSITIVITY_MIN, SENSITIVITY_MAX
        )));
    }

    // Validate FOV
    if ui_config.fov_degrees < FOV_MIN || ui_config.fov_degrees > FOV_MAX {
        return Err(BridgeError::validation_failed(&format!(
            "FOV must be between {} and {}",
            FOV_MIN as i32, FOV_MAX as i32
        )));
    }

    // Parse keybinds
    let keybind_entries = [
        (Action::Forward, &ui_config.keybinds.forward),
        (Action::Backward, &ui_config.keybinds.backward),
        (Action::Left, &ui_config.keybinds.left),
        (Action::Right, &ui_config.keybinds.right),
        (Action::Jump, &ui_config.keybinds.jump),
        (Action::Attack, &ui_config.keybinds.attack),
        (Action::PlaceBlock, &ui_config.keybinds.place_block),
        (Action::RemoveBlock, &ui_config.keybinds.remove_block),
        (Action::Pause, &ui_config.keybinds.pause),
        (Action::ToggleDebugOverlay, &ui_config.keybinds.toggle_debug),
    ];

    let mut new_bindings: HashMap<Action, Key> = HashMap::new();

    for (action, key_name) in keybind_entries {
        if key_name.is_empty() {
            continue;
        }

        match parse_key(key_name) {
            Some(key) => {
                new_bindings.insert(action, key);
            }
            None => {
                return Err(BridgeError::validation_failed(&format!(
                    "Invalid key '{}' for action {:?}",
                    sanitize_string_plain(key_name, 32),
                    action
                )));
            }
        }
    }

    // Apply changes
    config.sensitivity = ui_config.sensitivity;
    config.fov_degrees = ui_config.fov_degrees;
    config.fullscreen = ui_config.fullscreen;
    config.audio_muted = ui_config.audio_muted;

    // Update keybinds
    for (action, key) in new_bindings {
        config.keybinds.set(action, key);
    }

    Ok(())
}

/// Handle GetConfig message
pub fn handle_get_config(id: &str, config: &GameConfig) -> BridgeResponse {
    let ui_config = to_ui_config(config);

    match serde_json::to_value(&ui_config) {
        Ok(payload) => BridgeResponse::success(id, "GetConfig", payload),
        Err(e) => BridgeResponse::error(id, "GetConfig", BridgeError::save_failed(&e.to_string())),
    }
}

/// Handle SetConfig message
pub fn handle_set_config(
    id: &str,
    payload: &Value,
    config: &mut GameConfig,
) -> Result<BridgeResponse, BridgeError> {
    // Parse UI config from payload
    let ui_config: GameConfigUI = serde_json::from_value(payload.clone())
        .map_err(|e| BridgeError::invalid_format(&e.to_string()))?;

    // Apply and validate
    from_ui_config(&ui_config, config)?;

    // Save to disk
    if let Err(e) = crate::config::save_config(config) {
        return Err(BridgeError::save_failed(&e.to_string()));
    }

    Ok(BridgeResponse::success_empty(id, "SetConfig"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_ui_config() {
        let config = GameConfig::default();
        let ui_config = to_ui_config(&config);

        assert!((ui_config.sensitivity - config.sensitivity).abs() < f32::EPSILON);
        assert!((ui_config.fov_degrees - config.fov_degrees).abs() < f32::EPSILON);
        assert_eq!(ui_config.fullscreen, config.fullscreen);
        assert_eq!(ui_config.keybinds.forward, "W");
        assert_eq!(ui_config.keybinds.jump, "Space");
    }

    #[test]
    fn test_from_ui_config_valid() {
        let mut config = GameConfig::default();
        let ui_config = GameConfigUI {
            sensitivity: 0.005,
            fov_degrees: 90.0,
            fullscreen: true,
            audio_muted: true,
            keybinds: KeybindMapUI {
                forward: "W".to_string(),
                backward: "S".to_string(),
                left: "A".to_string(),
                right: "D".to_string(),
                jump: "Space".to_string(),
                attack: "LMB".to_string(),
                place_block: "RMB".to_string(),
                remove_block: "LMB".to_string(),
                pause: "Escape".to_string(),
                toggle_debug: "F3".to_string(),
            },
        };

        let result = from_ui_config(&ui_config, &mut config);
        assert!(result.is_ok());

        assert!((config.sensitivity - 0.005).abs() < f32::EPSILON);
        assert!((config.fov_degrees - 90.0).abs() < f32::EPSILON);
        assert!(config.fullscreen);
        assert!(config.audio_muted);
    }

    #[test]
    fn test_from_ui_config_invalid_sensitivity() {
        let mut config = GameConfig::default();
        let ui_config = GameConfigUI {
            sensitivity: 1.0, // Way out of range
            fov_degrees: 70.0,
            fullscreen: false,
            audio_muted: false,
            keybinds: KeybindMapUI::default(),
        };

        let result = from_ui_config(&ui_config, &mut config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code, "ECFG001");
    }

    #[test]
    fn test_from_ui_config_invalid_fov() {
        let mut config = GameConfig::default();
        let ui_config = GameConfigUI {
            sensitivity: 0.003,
            fov_degrees: 200.0, // Out of range
            fullscreen: false,
            audio_muted: false,
            keybinds: KeybindMapUI::default(),
        };

        let result = from_ui_config(&ui_config, &mut config);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_ui_config_invalid_key() {
        let mut config = GameConfig::default();
        let ui_config = GameConfigUI {
            sensitivity: 0.003,
            fov_degrees: 70.0,
            fullscreen: false,
            audio_muted: false,
            keybinds: KeybindMapUI {
                forward: "InvalidKey".to_string(),
                ..Default::default()
            },
        };

        let result = from_ui_config(&ui_config, &mut config);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_key() {
        assert_eq!(parse_key("W"), Some(Key::W));
        assert_eq!(parse_key("Space"), Some(Key::Space));
        assert_eq!(parse_key("LMB"), Some(Key::LeftClick));
        assert_eq!(parse_key("F3"), Some(Key::F3));
        assert_eq!(parse_key("Left Arrow"), Some(Key::Left));
        assert_eq!(parse_key("Invalid"), None);
    }

    #[test]
    fn test_handle_get_config() {
        let config = GameConfig::default();
        let response = handle_get_config("req-1", &config);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert!(payload.get("sensitivity").is_some());
        assert!(payload.get("keybinds").is_some());
    }
}
