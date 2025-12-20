//! Accessibility feature tests (Feature 042)
//!
//! Unit tests for accessibility configuration, keybind capture,
//! subtitle queue, and audio events.

use plix_client::accessibility::{
    AccessibilityConfig, AudioEvent, ColorblindPreset, KeybindCaptureState, KeybindConflict,
    SubtitleConfig, SubtitleEntry, SubtitleQueue, SubtitleSize, UI_SCALE_DEFAULT, UI_SCALE_MAX,
    UI_SCALE_MIN,
};
use plix_client::config::{Action, Key, Keybinds};

// ============================================================================
// AccessibilityConfig Tests
// ============================================================================

#[test]
fn test_accessibility_config_defaults() {
    let config = AccessibilityConfig::default();
    assert_eq!(config.ui_scale, UI_SCALE_DEFAULT);
    assert!(!config.high_contrast);
    assert_eq!(config.colorblind_preset, ColorblindPreset::None);
    assert!(!config.subtitles.enabled);
    assert_eq!(config.subtitles.size, SubtitleSize::Medium);
    assert_eq!(config.subtitles.background_opacity, 75);
    assert_eq!(config.subtitles.duration_ms, 3000);
}

#[test]
fn test_accessibility_config_validate_clamps_ui_scale() {
    let mut config = AccessibilityConfig::default();

    // Test upper bound
    config.ui_scale = 200;
    config.validate();
    assert_eq!(config.ui_scale, UI_SCALE_MAX);

    // Test lower bound
    config.ui_scale = 50;
    config.validate();
    assert_eq!(config.ui_scale, UI_SCALE_MIN);

    // Test valid value unchanged
    config.ui_scale = 125;
    config.validate();
    assert_eq!(config.ui_scale, 125);
}

#[test]
fn test_accessibility_config_validate_clamps_subtitles() {
    let mut config = AccessibilityConfig::default();

    // Test background opacity clamping
    config.subtitles.background_opacity = 150;
    config.subtitles.duration_ms = 500;
    config.validate();
    assert_eq!(config.subtitles.background_opacity, 100);
    assert_eq!(config.subtitles.duration_ms, 1000);
}

// ============================================================================
// ColorblindPreset Tests
// ============================================================================

#[test]
fn test_colorblind_preset_css_class() {
    assert_eq!(ColorblindPreset::None.css_class(), None);
    assert_eq!(
        ColorblindPreset::Protanopia.css_class(),
        Some("colorblind-protanopia")
    );
    assert_eq!(
        ColorblindPreset::Deuteranopia.css_class(),
        Some("colorblind-deuteranopia")
    );
    assert_eq!(
        ColorblindPreset::Tritanopia.css_class(),
        Some("colorblind-tritanopia")
    );
}

#[test]
fn test_colorblind_preset_display_names() {
    assert_eq!(ColorblindPreset::None.display_name(), "None");
    assert_eq!(
        ColorblindPreset::Protanopia.display_name(),
        "Protanopia (Red-blind)"
    );
    assert_eq!(
        ColorblindPreset::Deuteranopia.display_name(),
        "Deuteranopia (Green-blind)"
    );
    assert_eq!(
        ColorblindPreset::Tritanopia.display_name(),
        "Tritanopia (Blue-blind)"
    );
}

#[test]
fn test_colorblind_preset_from_str() {
    assert_eq!(
        ColorblindPreset::from_str("none"),
        Some(ColorblindPreset::None)
    );
    assert_eq!(
        ColorblindPreset::from_str("Protanopia"),
        Some(ColorblindPreset::Protanopia)
    );
    assert_eq!(
        ColorblindPreset::from_str("DEUTERANOPIA"),
        Some(ColorblindPreset::Deuteranopia)
    );
    assert_eq!(
        ColorblindPreset::from_str("tritanopia"),
        Some(ColorblindPreset::Tritanopia)
    );
    assert_eq!(ColorblindPreset::from_str("invalid"), None);
}

#[test]
fn test_colorblind_preset_all() {
    let all = ColorblindPreset::all();
    assert_eq!(all.len(), 4);
    assert!(all.contains(&ColorblindPreset::None));
    assert!(all.contains(&ColorblindPreset::Protanopia));
    assert!(all.contains(&ColorblindPreset::Deuteranopia));
    assert!(all.contains(&ColorblindPreset::Tritanopia));
}

// ============================================================================
// SubtitleSize Tests
// ============================================================================

#[test]
fn test_subtitle_size_font_size() {
    assert_eq!(SubtitleSize::Small.font_size_px(), 12);
    assert_eq!(SubtitleSize::Medium.font_size_px(), 16);
    assert_eq!(SubtitleSize::Large.font_size_px(), 20);
}

#[test]
fn test_subtitle_size_from_str() {
    assert_eq!(SubtitleSize::from_str("small"), Some(SubtitleSize::Small));
    assert_eq!(SubtitleSize::from_str("MEDIUM"), Some(SubtitleSize::Medium));
    assert_eq!(SubtitleSize::from_str("Large"), Some(SubtitleSize::Large));
    assert_eq!(SubtitleSize::from_str("invalid"), None);
}

#[test]
fn test_subtitle_size_all() {
    let all = SubtitleSize::all();
    assert_eq!(all.len(), 3);
}

// ============================================================================
// KeybindCaptureState Tests
// ============================================================================

#[test]
fn test_capture_state_lifecycle() {
    let mut state = KeybindCaptureState::new();

    // Initial state
    assert!(!state.is_listening());
    assert!(!state.has_conflict());
    assert!(state.capturing_action().is_none());

    // Start capture
    state.start_capture(Action::Forward);
    assert!(state.is_listening());
    assert_eq!(state.capturing_action(), Some(Action::Forward));

    // Cancel
    state.cancel();
    assert!(!state.is_listening());
}

#[test]
fn test_capture_state_timeout_check() {
    let mut state = KeybindCaptureState::new();
    state.start_capture(Action::Jump);

    // Should not be timed out immediately
    assert!(!state.is_timed_out());

    // Remaining time should be close to 5 seconds
    let remaining = state.remaining_time().unwrap();
    assert!(remaining.as_secs() <= 5);
    assert!(remaining.as_millis() > 4900);
}

#[test]
fn test_capture_state_conflict() {
    let mut state = KeybindCaptureState::new();
    let conflict = KeybindConflict::new(Action::Jump, Action::Forward, Key::W);
    state.set_conflict(conflict);

    assert!(state.has_conflict());
    assert!(!state.is_listening());

    let pending = state.pending_conflict().unwrap();
    assert_eq!(pending.target_action, Action::Jump);
    assert_eq!(pending.conflicting_action, Action::Forward);
    assert_eq!(pending.key, Key::W);

    // Resolve conflict
    state.resolve();
    assert!(!state.has_conflict());
}

// ============================================================================
// Conflict Detection Tests
// ============================================================================

#[test]
fn test_detect_conflict_exists() {
    use plix_client::accessibility::keybind_capture::detect_conflict;

    let keybinds = Keybinds::default();

    // W is bound to Forward, trying to bind Jump to W should conflict
    let conflict = detect_conflict(&keybinds, Action::Jump, Key::W);
    assert!(conflict.is_some());

    let conflict = conflict.unwrap();
    assert_eq!(conflict.target_action, Action::Jump);
    assert_eq!(conflict.conflicting_action, Action::Forward);
    assert_eq!(conflict.key, Key::W);
}

#[test]
fn test_detect_conflict_same_action() {
    use plix_client::accessibility::keybind_capture::detect_conflict;

    let keybinds = Keybinds::default();

    // Binding Forward to W (same action) should NOT conflict
    let conflict = detect_conflict(&keybinds, Action::Forward, Key::W);
    assert!(conflict.is_none());
}

#[test]
fn test_detect_conflict_unbound_key() {
    use plix_client::accessibility::keybind_capture::detect_conflict;

    let keybinds = Keybinds::default();

    // Binding to an unbound key should NOT conflict
    let conflict = detect_conflict(&keybinds, Action::Jump, Key::G);
    assert!(conflict.is_none());
}

// ============================================================================
// SubtitleQueue Tests
// ============================================================================

#[test]
fn test_subtitle_queue_basic() {
    let mut queue = SubtitleQueue::new();
    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    let entry = queue.push(AudioEvent::ChatMessage, 3000);
    assert_eq!(entry.text, "[Chat]");
    assert_eq!(queue.len(), 1);
}

#[test]
fn test_subtitle_queue_max_capacity() {
    let mut queue = SubtitleQueue::new();

    // Add 3 entries (max)
    queue.push(AudioEvent::ChatMessage, 3000);
    queue.push(AudioEvent::PlayerJoin, 3000);
    queue.push(AudioEvent::PlayerLeave, 3000);
    assert_eq!(queue.len(), 3);

    // Collect IDs before adding 4th
    let ids_before: Vec<_> = queue.entries().map(|e| e.id).collect();
    assert_eq!(ids_before, vec![1, 2, 3]);

    // Add 4th - should drop oldest (id=1)
    queue.push(AudioEvent::PlayerHit, 3000);
    assert_eq!(queue.len(), 3);

    let ids_after: Vec<_> = queue.entries().map(|e| e.id).collect();
    assert_eq!(ids_after, vec![2, 3, 4]);
}

#[test]
fn test_subtitle_queue_drop_oldest_on_overflow() {
    let mut queue = SubtitleQueue::new();

    // Fill to capacity
    queue.push(AudioEvent::ChatMessage, 3000); // id=1
    queue.push(AudioEvent::PlayerJoin, 3000); // id=2
    queue.push(AudioEvent::PlayerLeave, 3000); // id=3

    // Add more - should drop oldest each time
    queue.push(AudioEvent::PlayerHit, 3000); // id=4, drops id=1
    queue.push(AudioEvent::BlockPlace, 3000); // id=5, drops id=2

    let ids: Vec<_> = queue.entries().map(|e| e.id).collect();
    assert_eq!(ids, vec![3, 4, 5]);
}

#[test]
fn test_subtitle_queue_clear() {
    let mut queue = SubtitleQueue::new();
    queue.push(AudioEvent::ChatMessage, 3000);
    queue.push(AudioEvent::PlayerJoin, 3000);

    queue.clear();
    assert!(queue.is_empty());
}

#[test]
fn test_subtitle_queue_remove() {
    let mut queue = SubtitleQueue::new();
    queue.push(AudioEvent::ChatMessage, 3000); // id=1
    queue.push(AudioEvent::PlayerJoin, 3000); // id=2
    queue.push(AudioEvent::PlayerLeave, 3000); // id=3

    // Remove middle entry
    assert!(queue.remove(2));
    assert_eq!(queue.len(), 2);

    let ids: Vec<_> = queue.entries().map(|e| e.id).collect();
    assert_eq!(ids, vec![1, 3]);

    // Remove non-existent
    assert!(!queue.remove(999));
}

// ============================================================================
// AudioEvent Tests
// ============================================================================

#[test]
fn test_audio_event_subtitle_text() {
    assert_eq!(AudioEvent::ChatMessage.subtitle_text(), "[Chat]");
    assert_eq!(AudioEvent::PlayerJoin.subtitle_text(), "[Player Joined]");
    assert_eq!(AudioEvent::PlayerLeave.subtitle_text(), "[Player Left]");
    assert_eq!(AudioEvent::PlayerHit.subtitle_text(), "[Hit!]");
    assert_eq!(AudioEvent::BlockPlace.subtitle_text(), "[Block Placed]");
    assert_eq!(AudioEvent::BlockBreak.subtitle_text(), "[Block Broken]");
    assert_eq!(AudioEvent::MatchStart.subtitle_text(), "[Match Starting]");
    assert_eq!(AudioEvent::MatchEnd.subtitle_text(), "[Match Ended]");
}

#[test]
fn test_audio_event_all() {
    assert_eq!(AudioEvent::all().len(), 8);
}

// ============================================================================
// Integration: Config Serialization
// ============================================================================

#[test]
fn test_accessibility_config_serde_roundtrip() {
    let config = AccessibilityConfig {
        ui_scale: 125,
        high_contrast: true,
        colorblind_preset: ColorblindPreset::Deuteranopia,
        subtitles: SubtitleConfig {
            enabled: true,
            size: SubtitleSize::Large,
            background_opacity: 80,
            duration_ms: 4000,
        },
    };

    let serialized = toml::to_string(&config).unwrap();
    let deserialized: AccessibilityConfig = toml::from_str(&serialized).unwrap();

    assert_eq!(deserialized.ui_scale, 125);
    assert!(deserialized.high_contrast);
    assert_eq!(deserialized.colorblind_preset, ColorblindPreset::Deuteranopia);
    assert!(deserialized.subtitles.enabled);
    assert_eq!(deserialized.subtitles.size, SubtitleSize::Large);
}

#[test]
fn test_accessibility_config_defaults_on_missing_fields() {
    // Config with only some fields should use defaults for others
    let toml_str = r#"
ui_scale = 110
"#;
    let config: AccessibilityConfig = toml::from_str(toml_str).unwrap();

    assert_eq!(config.ui_scale, 110);
    assert!(!config.high_contrast); // default
    assert_eq!(config.colorblind_preset, ColorblindPreset::None); // default
    assert!(!config.subtitles.enabled); // default
}
