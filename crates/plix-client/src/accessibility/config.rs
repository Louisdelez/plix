//! Accessibility configuration types
//!
//! Defines AccessibilityConfig, ColorblindPreset, and SubtitleConfig
//! for visual and audio accessibility settings.

use serde::{Deserialize, Serialize};

/// UI Scale constraints
pub const UI_SCALE_MIN: u8 = 75;
pub const UI_SCALE_MAX: u8 = 150;
pub const UI_SCALE_DEFAULT: u8 = 100;
pub const UI_SCALE_STEP: u8 = 5;

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

fn default_ui_scale() -> u8 {
    UI_SCALE_DEFAULT
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            ui_scale: UI_SCALE_DEFAULT,
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

    /// Parse from string (for bridge messages)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "none" => Some(ColorblindPreset::None),
            "protanopia" => Some(ColorblindPreset::Protanopia),
            "deuteranopia" => Some(ColorblindPreset::Deuteranopia),
            "tritanopia" => Some(ColorblindPreset::Tritanopia),
            _ => None,
        }
    }

    /// Convert to string for serialization
    pub fn as_str(&self) -> &'static str {
        match self {
            ColorblindPreset::None => "none",
            ColorblindPreset::Protanopia => "protanopia",
            ColorblindPreset::Deuteranopia => "deuteranopia",
            ColorblindPreset::Tritanopia => "tritanopia",
        }
    }
}

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

fn default_subtitle_bg_opacity() -> u8 {
    75
}

fn default_subtitle_duration() -> u32 {
    3000
}

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
    /// Validate and clamp values to valid ranges
    pub fn validate(&mut self) {
        self.background_opacity = self.background_opacity.clamp(0, 100);
        self.duration_ms = self.duration_ms.clamp(1000, 10000);
    }
}

/// Subtitle text size options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubtitleSize {
    /// Small text (12px)
    Small,
    /// Medium text (16px, default)
    #[default]
    Medium,
    /// Large text (20px)
    Large,
}

impl SubtitleSize {
    /// Get all sizes for UI display
    pub fn all() -> &'static [SubtitleSize] {
        &[SubtitleSize::Small, SubtitleSize::Medium, SubtitleSize::Large]
    }

    /// Get display name for size
    pub fn display_name(&self) -> &'static str {
        match self {
            SubtitleSize::Small => "Small",
            SubtitleSize::Medium => "Medium",
            SubtitleSize::Large => "Large",
        }
    }

    /// Get font size in pixels
    pub fn font_size_px(&self) -> u8 {
        match self {
            SubtitleSize::Small => 12,
            SubtitleSize::Medium => 16,
            SubtitleSize::Large => 20,
        }
    }

    /// Parse from string (for bridge messages)
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "small" => Some(SubtitleSize::Small),
            "medium" => Some(SubtitleSize::Medium),
            "large" => Some(SubtitleSize::Large),
            _ => None,
        }
    }

    /// Convert to string for serialization
    pub fn as_str(&self) -> &'static str {
        match self {
            SubtitleSize::Small => "small",
            SubtitleSize::Medium => "medium",
            SubtitleSize::Large => "large",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accessibility_config_defaults() {
        let config = AccessibilityConfig::default();
        assert_eq!(config.ui_scale, UI_SCALE_DEFAULT);
        assert!(!config.high_contrast);
        assert_eq!(config.colorblind_preset, ColorblindPreset::None);
        assert!(!config.subtitles.enabled);
    }

    #[test]
    fn test_accessibility_config_validate_clamps_ui_scale() {
        let mut config = AccessibilityConfig::default();
        config.ui_scale = 200; // Too high
        config.validate();
        assert_eq!(config.ui_scale, UI_SCALE_MAX);

        config.ui_scale = 50; // Too low
        config.validate();
        assert_eq!(config.ui_scale, UI_SCALE_MIN);
    }

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
        assert_eq!(ColorblindPreset::from_str("invalid"), None);
    }

    #[test]
    fn test_subtitle_config_defaults() {
        let config = SubtitleConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.size, SubtitleSize::Medium);
        assert_eq!(config.background_opacity, 75);
        assert_eq!(config.duration_ms, 3000);
    }

    #[test]
    fn test_subtitle_config_validate() {
        let mut config = SubtitleConfig::default();
        config.background_opacity = 150; // Too high
        config.duration_ms = 500; // Too low
        config.validate();
        assert_eq!(config.background_opacity, 100);
        assert_eq!(config.duration_ms, 1000);
    }

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
}
