//! Accessibility module for the Plix client
//!
//! This module provides accessibility features:
//! - Keybinding remapping with conflict detection
//! - Visual accessibility (UI scale, colorblind presets, high contrast)
//! - Subtitles for audio events

pub mod audio_events;
pub mod config;
pub mod keybind_capture;
pub mod subtitle_queue;

// Re-export main types
pub use audio_events::AudioEvent;
pub use config::{
    AccessibilityConfig, ColorblindPreset, SubtitleConfig, SubtitleSize, UI_SCALE_DEFAULT,
    UI_SCALE_MAX, UI_SCALE_MIN, UI_SCALE_STEP,
};
pub use keybind_capture::{detect_conflict, KeybindCaptureState, KeybindConflict, CAPTURE_TIMEOUT};
pub use subtitle_queue::{SubtitleEntry, SubtitleQueue, MAX_SUBTITLES};
