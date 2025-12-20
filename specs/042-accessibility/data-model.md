# Data Model: Accessibility

## Type Definitions

### AccessibilityConfig

Holds all accessibility settings, nested in GameConfig.

```rust
/// Accessibility configuration
///
/// Stored in config.toml under `[accessibility]` section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// UI scale percentage (75-150, default: 100)
    #[serde(default = "default_ui_scale")]
    pub ui_scale: u8,

    /// High contrast mode enabled (default: false)
    #[serde(default)]
    pub high_contrast: bool,

    /// Colorblind preset (default: None)
    #[serde(default)]
    pub colorblind_preset: ColorblindPreset,

    /// Subtitle configuration
    #[serde(default)]
    pub subtitles: SubtitleConfig,
}

fn default_ui_scale() -> u8 { 100 }

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            ui_scale: 100,
            high_contrast: false,
            colorblind_preset: ColorblindPreset::None,
            subtitles: SubtitleConfig::default(),
        }
    }
}

impl AccessibilityConfig {
    /// Validate and clamp values to valid ranges
    pub fn validate(&mut self) {
        self.ui_scale = self.ui_scale.clamp(UI_SCALE_MIN, UI_SCALE_MAX);
        self.subtitles.validate();
    }
}

/// UI Scale constraints
pub const UI_SCALE_MIN: u8 = 75;
pub const UI_SCALE_MAX: u8 = 150;
pub const UI_SCALE_DEFAULT: u8 = 100;
pub const UI_SCALE_STEP: u8 = 5;
```

---

### ColorblindPreset

Enum of supported colorblind simulation modes.

```rust
/// Colorblind simulation preset
///
/// Implemented via CSS SVG filters on the CEF root element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorblindPreset {
    /// No colorblind simulation (default)
    #[default]
    None,

    /// Red-blind (1% of males)
    /// CSS filter: feColorMatrix with protanopia values
    Protanopia,

    /// Green-blind (6% of males, most common)
    /// CSS filter: feColorMatrix with deuteranopia values
    Deuteranopia,

    /// Blue-blind (rare, <1%)
    /// CSS filter: feColorMatrix with tritanopia values
    Tritanopia,
}

impl ColorblindPreset {
    /// Get all presets for UI display
    pub fn all() -> &'static [ColorblindPreset] {
        &[
            ColorblindPreset::None,
            ColorblindPreset::Protanopia,
            ColorblindPreset::Deuteranopia,
            ColorblindPreset::Tritanopia,
        ]
    }

    /// Get display name for preset
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorblindPreset::None => "None",
            ColorblindPreset::Protanopia => "Protanopia (Red-blind)",
            ColorblindPreset::Deuteranopia => "Deuteranopia (Green-blind)",
            ColorblindPreset::Tritanopia => "Tritanopia (Blue-blind)",
        }
    }

    /// Get CSS class name for this preset
    pub fn css_class(&self) -> Option<&'static str> {
        match self {
            ColorblindPreset::None => None,
            ColorblindPreset::Protanopia => Some("colorblind-protanopia"),
            ColorblindPreset::Deuteranopia => Some("colorblind-deuteranopia"),
            ColorblindPreset::Tritanopia => Some("colorblind-tritanopia"),
        }
    }
}
```

---

### SubtitleConfig

Configuration for subtitle display.

```rust
/// Subtitle display configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtitleConfig {
    /// Subtitles enabled (default: false)
    #[serde(default)]
    pub enabled: bool,

    /// Subtitle text size
    #[serde(default)]
    pub size: SubtitleSize,

    /// Background opacity percentage (0-100, default: 75)
    #[serde(default = "default_subtitle_bg_opacity")]
    pub background_opacity: u8,

    /// Display duration in milliseconds (default: 3000)
    #[serde(default = "default_subtitle_duration")]
    pub duration_ms: u32,
}

fn default_subtitle_bg_opacity() -> u8 { 75 }
fn default_subtitle_duration() -> u32 { 3000 }

impl Default for SubtitleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            size: SubtitleSize::default(),
            background_opacity: 75,
            duration_ms: 3000,
        }
    }
}

impl SubtitleConfig {
    pub fn validate(&mut self) {
        self.background_opacity = self.background_opacity.clamp(0, 100);
        self.duration_ms = self.duration_ms.clamp(1000, 10000);
    }
}

/// Subtitle text size options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubtitleSize {
    Small,   // 12px
    #[default]
    Medium,  // 16px
    Large,   // 20px
}

impl SubtitleSize {
    pub fn all() -> &'static [SubtitleSize] {
        &[SubtitleSize::Small, SubtitleSize::Medium, SubtitleSize::Large]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SubtitleSize::Small => "Small",
            SubtitleSize::Medium => "Medium",
            SubtitleSize::Large => "Large",
        }
    }

    pub fn font_size_px(&self) -> u8 {
        match self {
            SubtitleSize::Small => 12,
            SubtitleSize::Medium => 16,
            SubtitleSize::Large => 20,
        }
    }
}
```

---

### AudioEvent (for subtitles)

Enum of audio events that can display subtitles.

```rust
/// Audio events that can be captioned with subtitles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioEvent {
    /// Chat message received
    ChatMessage,

    /// Player joined server
    PlayerJoin,

    /// Player left server
    PlayerLeave,

    /// Local player took damage
    PlayerHit,

    /// Block placed
    BlockPlace,

    /// Block destroyed
    BlockBreak,

    /// Match started
    MatchStart,

    /// Match ended
    MatchEnd,
}

impl AudioEvent {
    /// Get subtitle text for this event
    pub fn subtitle_text(&self) -> &'static str {
        match self {
            AudioEvent::ChatMessage => "[Chat]",
            AudioEvent::PlayerJoin => "[Player Joined]",
            AudioEvent::PlayerLeave => "[Player Left]",
            AudioEvent::PlayerHit => "[Hit!]",
            AudioEvent::BlockPlace => "[Block Placed]",
            AudioEvent::BlockBreak => "[Block Broken]",
            AudioEvent::MatchStart => "[Match Starting]",
            AudioEvent::MatchEnd => "[Match Ended]",
        }
    }
}
```

---

### KeybindConflict

Represents a detected keybinding conflict.

```rust
/// A keybinding conflict between two actions
#[derive(Debug, Clone)]
pub struct KeybindConflict {
    /// The action the user is trying to rebind
    pub target_action: Action,

    /// The action that currently has the conflicting key
    pub conflicting_action: Action,

    /// The key that is causing the conflict
    pub key: Key,
}

impl KeybindConflict {
    pub fn new(target: Action, conflicting: Action, key: Key) -> Self {
        Self {
            target_action: target,
            conflicting_action: conflicting,
            key,
        }
    }
}
```

---

### Bridge Messages

Messages for CEF <-> Rust communication.

```rust
// ===== Rust -> JS (via bridge.send_to_ui) =====

/// Update accessibility settings in UI
#[derive(Debug, Clone, Serialize)]
pub struct AccessibilitySettingsMsg {
    pub ui_scale: u8,
    pub high_contrast: bool,
    pub colorblind_preset: String,  // "none", "protanopia", etc.
    pub subtitles_enabled: bool,
    pub subtitle_size: String,  // "small", "medium", "large"
    pub subtitle_bg_opacity: u8,
}

/// Display a subtitle
#[derive(Debug, Clone, Serialize)]
pub struct SubtitleShowMsg {
    pub event_id: String,
    pub text: String,
    pub duration_ms: u32,
}

/// Keybinding conflict detected
#[derive(Debug, Clone, Serialize)]
pub struct KeybindConflictMsg {
    pub target_action: String,
    pub conflicting_action: String,
    pub key: String,
}

// ===== JS -> Rust (via bridge message handler) =====

/// Request to rebind an action
#[derive(Debug, Clone, Deserialize)]
pub struct RebindActionMsg {
    pub action: String,   // Action variant name
    pub key: String,      // Key variant name
}

/// Request to reset keybindings to defaults
#[derive(Debug, Clone, Deserialize)]
pub struct ResetKeybindsMsg {}

/// Request to swap conflicting keybindings
#[derive(Debug, Clone, Deserialize)]
pub struct SwapKeybindsMsg {
    pub action1: String,
    pub action2: String,
}

/// Update accessibility setting
#[derive(Debug, Clone, Deserialize)]
pub struct SetAccessibilityMsg {
    pub setting: String,  // "ui_scale", "colorblind", "high_contrast", "subtitles"
    pub value: serde_json::Value,  // u8, string, or bool depending on setting
}
```

---

## TOML Configuration Schema

Extended config.toml structure:

```toml
# Existing fields
sensitivity = 0.003
fov_degrees = 70.0
fullscreen = false
audio_muted = false

# Existing keybinds section (no changes)
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
ToggleDebugOverlay = "F3"

# Existing UI section
[ui]
enabled = true
# ... other ui fields

# NEW: Accessibility section
[accessibility]
ui_scale = 100
high_contrast = false
colorblind_preset = "none"

[accessibility.subtitles]
enabled = false
size = "medium"
background_opacity = 75
duration_ms = 3000
```

---

## Updated GameConfig

```rust
/// Game configuration persisted to config.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub sensitivity: f32,
    pub fov_degrees: f32,
    pub fullscreen: bool,
    pub audio_muted: bool,
    pub keybinds: Keybinds,
    #[serde(default)]
    pub ui: CefConfig,

    // NEW: Accessibility settings
    #[serde(default)]
    pub accessibility: AccessibilityConfig,
}
```
