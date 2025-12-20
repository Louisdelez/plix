//! Bridge message handlers (Feature 031)
//!
//! Implements handlers for each bridge message type.

use super::messages::{BridgeError, BridgePush, BridgeResponse, PushType};
use super::serialize::sanitize_string_plain;
use crate::accessibility::{
    AccessibilityConfig, AudioEvent, ColorblindPreset, SubtitleConfig, SubtitleEntry, SubtitleSize,
};
use crate::config::{Action, GameConfig, Key, Keybinds};
use serde_json::{json, Value};

/// Supported bridge version (major.minor)
pub const BRIDGE_VERSION: &str = "1.0";

/// Handle Handshake message
///
/// Validates bridge version compatibility and returns player info.
pub fn handle_handshake(id: &str, payload: &Value, display_name: &str) -> BridgeResponse {
    // Extract bridge_version from payload
    let client_version = payload
        .get("bridge_version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Check version compatibility (accept 1.x)
    if !client_version.starts_with("1.") {
        return BridgeResponse::error(
            id,
            "Handshake",
            BridgeError::version_mismatch("1.x", client_version),
        );
    }

    // Sanitize display name
    let safe_name = sanitize_string_plain(display_name, 32);

    BridgeResponse::success(
        id,
        "Handshake",
        json!({
            "supported_version": BRIDGE_VERSION,
            "display_name": safe_name
        }),
    )
}

/// Handle Quit message
///
/// Returns success immediately. The game loop should check for quit flag.
pub fn handle_quit(id: &str) -> BridgeResponse {
    BridgeResponse::success_empty(id, "Quit")
}

// === Feature 042: Accessibility Handlers ===

/// Handle GetKeybinds message
///
/// Returns the current keybindings list for UI display.
pub fn handle_get_keybinds(id: &str, keybinds: &Keybinds) -> BridgeResponse {
    let bindings: Vec<Value> = Action::all()
        .iter()
        .map(|action| {
            let key = keybinds.key_for_action(*action);
            json!({
                "action": action.as_str(),
                "action_display": action.display_name(),
                "key": key.map(|k| k.as_str()),
                "key_display": key.map(|k| k.display_name())
            })
        })
        .collect();

    BridgeResponse::success(id, "GetKeybinds", json!({ "bindings": bindings }))
}

/// Handle GetAccessibilitySettings message
///
/// Returns the current accessibility configuration.
pub fn handle_get_accessibility_settings(id: &str, config: &AccessibilityConfig) -> BridgeResponse {
    BridgeResponse::success(
        id,
        "GetAccessibilitySettings",
        json!({
            "ui_scale": config.ui_scale,
            "high_contrast": config.high_contrast,
            "colorblind_preset": config.colorblind_preset.as_str(),
            "subtitles": {
                "enabled": config.subtitles.enabled,
                "size": config.subtitles.size.as_str(),
                "background_opacity": config.subtitles.background_opacity,
                "duration_ms": config.subtitles.duration_ms
            }
        }),
    )
}

/// Handle SetAccessibility message
///
/// Updates one or more accessibility settings. Returns the updated config.
pub fn handle_set_accessibility(
    id: &str,
    payload: &Value,
    config: &mut AccessibilityConfig,
) -> BridgeResponse {
    // Parse and apply settings from payload
    if let Some(ui_scale) = payload.get("ui_scale").and_then(|v| v.as_u64()) {
        config.ui_scale = (ui_scale as u8).clamp(75, 150);
    }

    if let Some(high_contrast) = payload.get("high_contrast").and_then(|v| v.as_bool()) {
        config.high_contrast = high_contrast;
    }

    if let Some(preset_str) = payload.get("colorblind_preset").and_then(|v| v.as_str()) {
        if let Some(preset) = ColorblindPreset::from_str(preset_str) {
            config.colorblind_preset = preset;
        } else {
            return BridgeResponse::error(
                id,
                "SetAccessibility",
                BridgeError::invalid_value(&format!("Unknown colorblind preset: {}", preset_str)),
            );
        }
    }

    // Handle subtitle settings
    if let Some(subtitles) = payload.get("subtitles").and_then(|v| v.as_object()) {
        if let Some(enabled) = subtitles.get("enabled").and_then(|v| v.as_bool()) {
            config.subtitles.enabled = enabled;
        }

        if let Some(size_str) = subtitles.get("size").and_then(|v| v.as_str()) {
            if let Some(size) = SubtitleSize::from_str(size_str) {
                config.subtitles.size = size;
            } else {
                return BridgeResponse::error(
                    id,
                    "SetAccessibility",
                    BridgeError::invalid_value(&format!("Unknown subtitle size: {}", size_str)),
                );
            }
        }

        if let Some(opacity) = subtitles.get("background_opacity").and_then(|v| v.as_u64()) {
            config.subtitles.background_opacity = (opacity as u8).clamp(0, 100);
        }

        if let Some(duration) = subtitles.get("duration_ms").and_then(|v| v.as_u64()) {
            config.subtitles.duration_ms = (duration as u32).clamp(1000, 10000);
        }
    }

    // Validate after all changes
    config.validate();

    // Return updated config
    handle_get_accessibility_settings(id, config)
}

/// Handle RebindAction message
///
/// Rebinds an action to a new key. Returns conflict if key is already bound.
pub fn handle_rebind_action(
    id: &str,
    payload: &Value,
    keybinds: &mut Keybinds,
) -> BridgeResponse {
    let action_str = match payload.get("action").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return BridgeResponse::error(
                id,
                "RebindAction",
                BridgeError::invalid_format("Missing 'action' field"),
            );
        }
    };

    let key_str = match payload.get("key").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return BridgeResponse::error(
                id,
                "RebindAction",
                BridgeError::invalid_format("Missing 'key' field"),
            );
        }
    };

    let action = match Action::from_str(action_str) {
        Some(a) => a,
        None => {
            return BridgeResponse::error(
                id,
                "RebindAction",
                BridgeError::invalid_action(action_str),
            );
        }
    };

    let key = match Key::from_str(key_str) {
        Some(k) => k,
        None => {
            return BridgeResponse::error(id, "RebindAction", BridgeError::invalid_key(key_str));
        }
    };

    // Check for conflicts
    if let Some(existing_action) = keybinds.action_for_key(key) {
        if existing_action != action {
            // Conflict detected - return conflict info
            return BridgeResponse::success(
                id,
                "RebindAction",
                json!({
                    "conflict": true,
                    "target_action": action.as_str(),
                    "target_action_display": action.display_name(),
                    "conflicting_action": existing_action.as_str(),
                    "conflicting_action_display": existing_action.display_name(),
                    "key": key.as_str(),
                    "key_display": key.display_name()
                }),
            );
        }
    }

    // No conflict - perform the rebind
    keybinds.rebind(action, key);

    BridgeResponse::success(
        id,
        "RebindAction",
        json!({
            "conflict": false,
            "action": action.as_str(),
            "key": key.as_str()
        }),
    )
}

/// Handle SwapKeybinds message
///
/// Swaps keybindings between two actions (conflict resolution).
pub fn handle_swap_keybinds(
    id: &str,
    payload: &Value,
    keybinds: &mut Keybinds,
) -> BridgeResponse {
    let action1_str = match payload.get("action1").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return BridgeResponse::error(
                id,
                "SwapKeybinds",
                BridgeError::invalid_format("Missing 'action1' field"),
            );
        }
    };

    let action2_str = match payload.get("action2").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => {
            return BridgeResponse::error(
                id,
                "SwapKeybinds",
                BridgeError::invalid_format("Missing 'action2' field"),
            );
        }
    };

    let action1 = match Action::from_str(action1_str) {
        Some(a) => a,
        None => {
            return BridgeResponse::error(
                id,
                "SwapKeybinds",
                BridgeError::invalid_action(action1_str),
            );
        }
    };

    let action2 = match Action::from_str(action2_str) {
        Some(a) => a,
        None => {
            return BridgeResponse::error(
                id,
                "SwapKeybinds",
                BridgeError::invalid_action(action2_str),
            );
        }
    };

    // Get current keys for both actions
    let key1 = keybinds.key_for_action(action1);
    let key2 = keybinds.key_for_action(action2);

    // Perform the swap
    if let Some(k2) = key2 {
        keybinds.rebind(action1, k2);
    }
    if let Some(k1) = key1 {
        keybinds.rebind(action2, k1);
    }

    BridgeResponse::success(
        id,
        "SwapKeybinds",
        json!({
            "action1": action1.as_str(),
            "action1_key": key2.map(|k| k.as_str()),
            "action2": action2.as_str(),
            "action2_key": key1.map(|k| k.as_str())
        }),
    )
}

/// Handle ResetKeybinds message
///
/// Resets all keybindings to defaults.
pub fn handle_reset_keybinds(id: &str, keybinds: &mut Keybinds) -> BridgeResponse {
    *keybinds = Keybinds::default();
    handle_get_keybinds(id, keybinds)
}

// === Push Event Builders (Feature 042) ===

/// Create a SubtitleShow push event
pub fn push_subtitle_show(entry: &SubtitleEntry) -> BridgePush {
    BridgePush::new(
        PushType::SubtitleShow.as_str(),
        json!({
            "id": entry.id,
            "event": entry.event.as_str(),
            "text": entry.text,
            "duration_ms": entry.duration.as_millis() as u32
        }),
    )
}

/// Create a SubtitleClear push event
pub fn push_subtitle_clear() -> BridgePush {
    BridgePush::new(PushType::SubtitleClear.as_str(), json!({}))
}

/// Create an AccessibilitySettings push event
pub fn push_accessibility_settings(config: &AccessibilityConfig) -> BridgePush {
    BridgePush::new(
        PushType::AccessibilitySettings.as_str(),
        json!({
            "ui_scale": config.ui_scale,
            "high_contrast": config.high_contrast,
            "colorblind_preset": config.colorblind_preset.as_str(),
            "subtitles": {
                "enabled": config.subtitles.enabled,
                "size": config.subtitles.size.as_str(),
                "background_opacity": config.subtitles.background_opacity,
                "duration_ms": config.subtitles.duration_ms
            }
        }),
    )
}

/// Create a KeybindConflict push event
pub fn push_keybind_conflict(
    target_action: Action,
    conflicting_action: Action,
    key: Key,
) -> BridgePush {
    BridgePush::new(
        PushType::KeybindConflict.as_str(),
        json!({
            "target_action": target_action.as_str(),
            "target_action_display": target_action.display_name(),
            "conflicting_action": conflicting_action.as_str(),
            "conflicting_action_display": conflicting_action.display_name(),
            "key": key.as_str(),
            "key_display": key.display_name()
        }),
    )
}

/// Create a KeybindCaptureTimeout push event
pub fn push_keybind_capture_timeout(action: Action) -> BridgePush {
    BridgePush::new(
        PushType::KeybindCaptureTimeout.as_str(),
        json!({
            "action": action.as_str(),
            "action_display": action.display_name()
        }),
    )
}

/// Create a KeybindsList push event
pub fn push_keybinds_list(keybinds: &Keybinds) -> BridgePush {
    let bindings: Vec<Value> = Action::all()
        .iter()
        .map(|action| {
            let key = keybinds.key_for_action(*action);
            json!({
                "action": action.as_str(),
                "action_display": action.display_name(),
                "key": key.map(|k| k.as_str()),
                "key_display": key.map(|k| k.display_name())
            })
        })
        .collect();

    BridgePush::new(PushType::KeybindsList.as_str(), json!({ "bindings": bindings }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_valid_version() {
        let payload = json!({"bridge_version": "1.0"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert_eq!(payload["supported_version"], "1.0");
        assert_eq!(payload["display_name"], "TestPlayer");
    }

    #[test]
    fn test_handshake_compatible_version() {
        let payload = json!({"bridge_version": "1.5"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(response.ok); // 1.x should be compatible
    }

    #[test]
    fn test_handshake_incompatible_version() {
        let payload = json!({"bridge_version": "2.0"});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "EBRG001");
    }

    #[test]
    fn test_handshake_missing_version() {
        let payload = json!({});
        let response = handle_handshake("req-1", &payload, "TestPlayer");

        assert!(!response.ok); // "unknown" doesn't match 1.x
    }

    #[test]
    fn test_handshake_sanitizes_display_name() {
        let payload = json!({"bridge_version": "1.0"});
        let long_name = "a".repeat(100);
        let response = handle_handshake("req-1", &payload, &long_name);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        let display_name = payload["display_name"].as_str().unwrap();
        assert!(display_name.len() <= 32);
    }

    #[test]
    fn test_quit() {
        let response = handle_quit("req-1");

        assert!(response.ok);
        assert_eq!(response.msg_type, "Quit");
    }

    // === Feature 042: Accessibility Tests ===

    #[test]
    fn test_get_keybinds() {
        let keybinds = Keybinds::default();
        let response = handle_get_keybinds("req-1", &keybinds);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        let bindings = payload["bindings"].as_array().unwrap();

        // Should have all actions
        assert_eq!(bindings.len(), Action::all().len());

        // Check that Forward is bound to W
        let forward = bindings.iter().find(|b| b["action"] == "forward").unwrap();
        assert_eq!(forward["key"], "w");
        assert_eq!(forward["action_display"], "Forward");
        assert_eq!(forward["key_display"], "W");
    }

    #[test]
    fn test_get_accessibility_settings() {
        let config = AccessibilityConfig::default();
        let response = handle_get_accessibility_settings("req-1", &config);

        assert!(response.ok);
        let payload = response.payload.unwrap();

        assert_eq!(payload["ui_scale"], 100);
        assert_eq!(payload["high_contrast"], false);
        assert_eq!(payload["colorblind_preset"], "none");
        assert_eq!(payload["subtitles"]["enabled"], false);
        assert_eq!(payload["subtitles"]["size"], "medium");
        assert_eq!(payload["subtitles"]["background_opacity"], 75);
    }

    #[test]
    fn test_set_accessibility_ui_scale() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({"ui_scale": 125});
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(response.ok);
        assert_eq!(config.ui_scale, 125);
    }

    #[test]
    fn test_set_accessibility_ui_scale_clamped() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({"ui_scale": 200}); // Over max
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(response.ok);
        assert_eq!(config.ui_scale, 150); // Clamped to max
    }

    #[test]
    fn test_set_accessibility_high_contrast() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({"high_contrast": true});
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(response.ok);
        assert!(config.high_contrast);
    }

    #[test]
    fn test_set_accessibility_colorblind_preset() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({"colorblind_preset": "deuteranopia"});
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(response.ok);
        assert_eq!(config.colorblind_preset, ColorblindPreset::Deuteranopia);
    }

    #[test]
    fn test_set_accessibility_invalid_colorblind_preset() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({"colorblind_preset": "invalid"});
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "EACC004");
    }

    #[test]
    fn test_set_accessibility_subtitles() {
        let mut config = AccessibilityConfig::default();
        let payload = json!({
            "subtitles": {
                "enabled": true,
                "size": "large",
                "background_opacity": 50,
                "duration_ms": 5000
            }
        });
        let response = handle_set_accessibility("req-1", &payload, &mut config);

        assert!(response.ok);
        assert!(config.subtitles.enabled);
        assert_eq!(config.subtitles.size, SubtitleSize::Large);
        assert_eq!(config.subtitles.background_opacity, 50);
        assert_eq!(config.subtitles.duration_ms, 5000);
    }

    #[test]
    fn test_rebind_action_no_conflict() {
        let mut keybinds = Keybinds::default();
        let payload = json!({"action": "jump", "key": "g"}); // G is unbound
        let response = handle_rebind_action("req-1", &payload, &mut keybinds);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert_eq!(payload["conflict"], false);
        assert_eq!(keybinds.key_for_action(Action::Jump), Some(Key::G));
    }

    #[test]
    fn test_rebind_action_with_conflict() {
        let mut keybinds = Keybinds::default();
        let payload = json!({"action": "jump", "key": "w"}); // W is bound to Forward
        let response = handle_rebind_action("req-1", &payload, &mut keybinds);

        assert!(response.ok);
        let payload = response.payload.unwrap();
        assert_eq!(payload["conflict"], true);
        assert_eq!(payload["target_action"], "jump");
        assert_eq!(payload["conflicting_action"], "forward");
        // Keybind should NOT have changed
        assert_eq!(keybinds.key_for_action(Action::Jump), Some(Key::Space));
    }

    #[test]
    fn test_rebind_action_invalid_action() {
        let mut keybinds = Keybinds::default();
        let payload = json!({"action": "invalid_action", "key": "g"});
        let response = handle_rebind_action("req-1", &payload, &mut keybinds);

        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "EACC001");
    }

    #[test]
    fn test_rebind_action_invalid_key() {
        let mut keybinds = Keybinds::default();
        let payload = json!({"action": "jump", "key": "invalid_key"});
        let response = handle_rebind_action("req-1", &payload, &mut keybinds);

        assert!(!response.ok);
        let error = response.error.unwrap();
        assert_eq!(error.code, "EACC002");
    }

    #[test]
    fn test_swap_keybinds() {
        let mut keybinds = Keybinds::default();
        // Forward=W, Backward=S
        let payload = json!({"action1": "forward", "action2": "backward"});
        let response = handle_swap_keybinds("req-1", &payload, &mut keybinds);

        assert!(response.ok);
        // After swap: Forward=S, Backward=W
        assert_eq!(keybinds.key_for_action(Action::Forward), Some(Key::S));
        assert_eq!(keybinds.key_for_action(Action::Backward), Some(Key::W));
    }

    #[test]
    fn test_reset_keybinds() {
        let mut keybinds = Keybinds::default();
        // Modify a binding
        keybinds.rebind(Action::Forward, Key::G);
        assert_eq!(keybinds.key_for_action(Action::Forward), Some(Key::G));

        // Reset
        let response = handle_reset_keybinds("req-1", &mut keybinds);

        assert!(response.ok);
        // Should be back to default
        assert_eq!(keybinds.key_for_action(Action::Forward), Some(Key::W));
    }

    #[test]
    fn test_push_subtitle_show() {
        use std::time::Duration;

        let entry = SubtitleEntry {
            id: 1,
            event: AudioEvent::ChatMessage,
            text: "[Chat] Test".to_string(),
            created_at: std::time::Instant::now(),
            duration: Duration::from_millis(3000),
        };

        let push = push_subtitle_show(&entry);
        assert_eq!(push.push_type, "SubtitleShow");

        let payload = push.payload;
        assert_eq!(payload["id"], 1);
        assert_eq!(payload["event"], "chat_message");
        assert_eq!(payload["text"], "[Chat] Test");
        assert_eq!(payload["duration_ms"], 3000);
    }

    #[test]
    fn test_push_accessibility_settings() {
        let config = AccessibilityConfig::default();
        let push = push_accessibility_settings(&config);

        assert_eq!(push.push_type, "AccessibilitySettings");
        let payload = push.payload;
        assert_eq!(payload["ui_scale"], 100);
        assert_eq!(payload["high_contrast"], false);
    }

    #[test]
    fn test_push_keybind_conflict() {
        let push = push_keybind_conflict(Action::Jump, Action::Forward, Key::W);

        assert_eq!(push.push_type, "KeybindConflict");
        let payload = push.payload;
        assert_eq!(payload["target_action"], "jump");
        assert_eq!(payload["conflicting_action"], "forward");
        assert_eq!(payload["key"], "w");
    }
}
